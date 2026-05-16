use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use db::entities::background_jobs;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

impl ToString for JobStatus {
    fn to_string(&self) -> String {
        match self {
            Self::Queued => "QUEUED".to_string(),
            Self::Running => "RUNNING".to_string(),
            Self::Completed => "COMPLETED".to_string(),
            Self::Failed => "FAILED".to_string(),
        }
    }
}

pub trait JobArgs: Serialize + DeserializeOwned + Send + Sync + 'static {
    const JOB_TYPE: &'static str;
}

#[async_trait]
pub trait JobHandler<Args: JobArgs>: Send + Sync + 'static {
    async fn handle(&self, args: Args) -> Result<()>;
}

#[async_trait]
pub trait JobQueue: Send + Sync {
    async fn enqueue_erased(
        &self,
        job_type: String,
        payload: serde_json::Value,
        user_id: Option<String>,
        run_at: Option<DateTime<Utc>>,
    ) -> Result<String>;
}

#[async_trait]
pub trait JobQueueExt: JobQueue {
    async fn enqueue<Args: JobArgs>(
        &self,
        args: Args,
        user_id: Option<String>,
        run_at: Option<DateTime<Utc>>,
    ) -> Result<String> {
        let job_type = Args::JOB_TYPE.to_string();
        let payload = serde_json::to_value(args)?;
        self.enqueue_erased(job_type, payload, user_id, run_at)
            .await
    }
}

impl<T: JobQueue + ?Sized> JobQueueExt for T {}

pub struct DbJobQueue {
    db: Arc<DatabaseConnection>,
}

impl DbJobQueue {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl JobQueue for DbJobQueue {
    async fn enqueue_erased(
        &self,
        job_type: String,
        payload: serde_json::Value,
        user_id: Option<String>,
        run_at: Option<DateTime<Utc>>,
    ) -> Result<String> {
        let id = Uuid::now_v7().to_string();
        let now = Utc::now();
        let run_at = run_at.unwrap_or(now);

        let active_model = background_jobs::ActiveModel {
            id: Set(id.clone()),
            job_type: Set(job_type),
            payload: Set(payload),
            status: Set(JobStatus::Queued.to_string()),
            attempts: Set(0),
            max_attempts: Set(3),
            run_at: Set(run_at.naive_utc()),
            created_at: Set(now.naive_utc()),
            user_id: Set(user_id),
            ..Default::default()
        };

        active_model.insert(self.db.as_ref()).await?;
        Ok(id)
    }
}

#[async_trait]
trait ErasedJobHandler: Send + Sync {
    async fn handle(&self, payload: serde_json::Value) -> Result<()>;
}

struct JobHandlerWrapper<Args, H> {
    _phantom: std::marker::PhantomData<Args>,
    handler: H,
}

#[async_trait]
impl<Args, H> ErasedJobHandler for JobHandlerWrapper<Args, H>
where
    Args: JobArgs,
    H: JobHandler<Args>,
{
    async fn handle(&self, payload: serde_json::Value) -> Result<()> {
        let args: Args = serde_json::from_value(payload)?;
        self.handler.handle(args).await
    }
}

pub struct WorkerPool {
    db: Arc<DatabaseConnection>,
    handlers: std::collections::HashMap<String, Arc<dyn ErasedJobHandler>>,
    semaphore: Arc<Semaphore>,
}

impl WorkerPool {
    pub fn new(db: Arc<DatabaseConnection>, concurrency: usize) -> Self {
        Self {
            db,
            handlers: std::collections::HashMap::new(),
            semaphore: Arc::new(Semaphore::new(concurrency)),
        }
    }

    pub fn register_handler<Args, H>(&mut self, handler: H)
    where
        Args: JobArgs,
        H: JobHandler<Args>,
    {
        self.handlers.insert(
            Args::JOB_TYPE.to_string(),
            Arc::new(JobHandlerWrapper {
                _phantom: std::marker::PhantomData,
                handler,
            }),
        );
    }

    pub async fn run(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            if let Err(e) = self.process_queued_jobs().await {
                tracing::error!("❌ Background worker pool failed: {}", e);
            }
        }
    }

    async fn process_queued_jobs(&self) -> Result<()> {
        let now = Utc::now().naive_utc();

        // Use a transaction and "SELECT ... FOR UPDATE SKIP LOCKED" logic if possible.
        // For SeaORM, we'll fetch then try to mark as running.

        let queued_jobs = background_jobs::Entity::find()
            .filter(background_jobs::Column::Status.eq(JobStatus::Queued.to_string()))
            .filter(background_jobs::Column::RunAt.lte(now))
            .all(self.db.as_ref())
            .await?;

        for job in queued_jobs {
            let handler = match self.handlers.get(&job.job_type) {
                Some(h) => h.clone(),
                None => {
                    tracing::warn!("⚠️ No handler registered for job type: {}", job.job_type);
                    continue;
                }
            };

            let permit = match self.semaphore.clone().try_acquire_owned() {
                Ok(p) => p,
                Err(_) => break, // No more capacity
            };

            let db = self.db.clone();
            let job_id = job.id.clone();
            let payload = job.payload.clone();

            tokio::spawn(async move {
                let _permit = permit;

                // Panic recovery
                let result = std::panic::AssertUnwindSafe(Self::execute_job(
                    db.clone(),
                    job_id.clone(),
                    payload,
                    handler,
                ))
                .catch_unwind()
                .await;

                match result {
                    Ok(Ok(_)) => {}
                    Ok(Err(e)) => {
                        tracing::error!("❌ Job {} failed: {}", job_id, e);
                    }
                    Err(_) => {
                        tracing::error!("🔥 Job {} panicked!", job_id);
                        let _ = Self::mark_failed(db, job_id, "Panic occurred".to_string()).await;
                    }
                }
            });
        }

        Ok(())
    }

    async fn execute_job(
        db: Arc<DatabaseConnection>,
        job_id: String,
        payload: serde_json::Value,
        handler: Arc<dyn ErasedJobHandler>,
    ) -> Result<()> {
        // 1. Mark as Running
        let active_model = background_jobs::ActiveModel {
            id: Set(job_id.clone()),
            status: Set(JobStatus::Running.to_string()),
            ..Default::default()
        };
        active_model.update(db.as_ref()).await?;

        // 2. Execute
        match handler.handle(payload).await {
            Ok(_) => {
                let active_model = background_jobs::ActiveModel {
                    id: Set(job_id),
                    status: Set(JobStatus::Completed.to_string()),
                    completed_at: Set(Some(Utc::now().naive_utc())),
                    ..Default::default()
                };
                active_model.update(db.as_ref()).await?;
            }
            Err(e) => {
                Self::handle_job_error(db, job_id, e).await?;
            }
        }

        Ok(())
    }

    async fn handle_job_error(
        db: Arc<DatabaseConnection>,
        job_id: String,
        error: anyhow::Error,
    ) -> Result<()> {
        let job = background_jobs::Entity::find_by_id(job_id.clone())
            .one(db.as_ref())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Job not found"))?;

        let next_attempts = job.attempts + 1;
        let status = if next_attempts >= job.max_attempts {
            JobStatus::Failed
        } else {
            JobStatus::Queued
        };

        let active_model = background_jobs::ActiveModel {
            id: Set(job_id),
            status: Set(status.to_string()),
            attempts: Set(next_attempts),
            error: Set(Some(error.to_string())),
            ..Default::default()
        };
        active_model.update(db.as_ref()).await?;
        Ok(())
    }

    async fn mark_failed(db: Arc<DatabaseConnection>, job_id: String, error: String) -> Result<()> {
        let active_model = background_jobs::ActiveModel {
            id: Set(job_id),
            status: Set(JobStatus::Failed.to_string()),
            error: Set(Some(error)),
            ..Default::default()
        };
        active_model.update(db.as_ref()).await?;
        Ok(())
    }
}

use futures::FutureExt;
