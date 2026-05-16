use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use db::entities::background_jobs;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[async_trait]
pub trait JobHandler: Send + Sync {
    async fn handle(&self, payload: serde_json::Value) -> Result<()>;
}

#[async_trait]
pub trait JobQueue: Send + Sync {
    async fn enqueue(
        &self,
        job_type: &str,
        payload: serde_json::Value,
        user_id: Option<String>,
        run_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<String>;
}

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
    async fn enqueue(
        &self,
        job_type: &str,
        payload: serde_json::Value,
        user_id: Option<String>,
        run_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<String> {
        let id = Uuid::now_v7().to_string();
        let now = Utc::now();
        let run_at = run_at.unwrap_or(now);

        let active_model = background_jobs::ActiveModel {
            id: Set(id.clone()),
            job_type: Set(job_type.to_string()),
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

pub struct WorkerPool {
    db: Arc<DatabaseConnection>,
    handlers: std::collections::HashMap<String, Arc<dyn JobHandler>>,
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

    pub fn register_handler(&mut self, job_type: &str, handler: Arc<dyn JobHandler>) {
        self.handlers.insert(job_type.to_string(), handler);
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
                Err(_) => break, // No more concurrency capacity for now
            };

            let db = self.db.clone();
            let job_id = job.id.clone();
            let payload = job.payload.clone();

            tokio::spawn(async move {
                let _permit = permit;
                if let Err(e) = Self::execute_job(db, job_id, payload, handler).await {
                    tracing::error!("❌ Job execution failed: {}", e);
                }
            });
        }

        Ok(())
    }

    async fn execute_job(
        db: Arc<DatabaseConnection>,
        job_id: String,
        payload: serde_json::Value,
        handler: Arc<dyn JobHandler>,
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
                tracing::error!("❌ Job {} failed: {}", job_id, e);

                // Get current job to check attempts
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
                    error: Set(Some(e.to_string())),
                    ..Default::default()
                };
                active_model.update(db.as_ref()).await?;
            }
        }

        Ok(())
    }
}
