use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Timelike, Utc};
use db::entities::background_jobs;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Queued => write!(f, "QUEUED"),
            Self::Running => write!(f, "RUNNING"),
            Self::Completed => write!(f, "COMPLETED"),
            Self::Failed => write!(f, "FAILED"),
        }
    }
}

pub trait Job: Serialize + DeserializeOwned + Send + Sync + 'static {
    const NAME: &'static str;

    fn max_attempts(&self) -> i32 {
        3
    }
}

#[async_trait]
pub trait Handler<J: Job, Ctx>: Send + Sync + 'static {
    async fn handle(&self, ctx: &Ctx, job: J) -> Result<()>;
}

#[async_trait]
pub trait JobQueue: Send + Sync {
    async fn enqueue_erased(
        &self,
        job_type: String,
        payload: serde_json::Value,
        user_id: Option<String>,
        run_at: Option<DateTime<Utc>>,
        max_attempts: i32,
    ) -> Result<String>;
}

#[async_trait]
pub trait JobQueueExt: JobQueue {
    async fn enqueue<J: Job>(
        &self,
        job: J,
        user_id: Option<String>,
        run_at: Option<DateTime<Utc>>,
    ) -> Result<String> {
        let job_type = J::NAME.to_string();
        let max_attempts = job.max_attempts();
        let payload = serde_json::to_value(job)?;
        self.enqueue_erased(job_type, payload, user_id, run_at, max_attempts)
            .await
    }
}

impl<T: JobQueue + ?Sized> JobQueueExt for T {}

pub struct DbJobQueue {
    db: Arc<DatabaseConnection>,
}

impl DbJobQueue {
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Enqueue a job using a specific database connection (e.g. a transaction)
    pub async fn enqueue_in_conn<J: Job>(
        conn: &impl ConnectionTrait,
        job: J,
        user_id: Option<String>,
        run_at: Option<DateTime<Utc>>,
    ) -> Result<String> {
        let job_type = J::NAME.to_string();
        let max_attempts = job.max_attempts();
        let payload = serde_json::to_value(job)?;
        enqueue_job_internal(conn, job_type, payload, user_id, run_at, max_attempts).await
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
        max_attempts: i32,
    ) -> Result<String> {
        enqueue_job_internal(
            self.db.as_ref(),
            job_type,
            payload,
            user_id,
            run_at,
            max_attempts,
        )
        .await
    }
}

async fn enqueue_job_internal(
    conn: &impl ConnectionTrait,
    job_type: String,
    payload: serde_json::Value,
    user_id: Option<String>,
    run_at: Option<DateTime<Utc>>,
    max_attempts: i32,
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
        max_attempts: Set(max_attempts),
        run_at: Set(run_at.naive_utc()),
        created_at: Set(now.naive_utc()),
        updated_at: Set(now.naive_utc()),
        user_id: Set(user_id),
        ..Default::default()
    };

    active_model.insert(conn).await?;
    Ok(id)
}

#[async_trait]
trait ErasedHandler<Ctx>: Send + Sync {
    async fn handle(&self, ctx: &Ctx, payload: serde_json::Value) -> Result<()>;
}

struct HandlerWrapper<J, H> {
    _phantom: std::marker::PhantomData<J>,
    handler: H,
}

#[async_trait]
impl<J, H, Ctx> ErasedHandler<Ctx> for HandlerWrapper<J, H>
where
    J: Job,
    H: Handler<J, Ctx>,
    Ctx: Send + Sync + 'static,
{
    async fn handle(&self, ctx: &Ctx, payload: serde_json::Value) -> Result<()> {
        let job: J = serde_json::from_value(payload)?;
        self.handler.handle(ctx, job).await
    }
}

pub struct WorkerPool<Ctx> {
    db: Arc<DatabaseConnection>,
    context: Ctx,
    handlers: HashMap<String, Arc<dyn ErasedHandler<Ctx>>>,
    semaphore: Arc<Semaphore>,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
}

impl<Ctx> WorkerPool<Ctx>
where
    Ctx: Clone + Send + Sync + 'static,
{
    pub fn new(db: Arc<DatabaseConnection>, context: Ctx, concurrency: usize) -> Self {
        Self {
            db,
            context,
            handlers: HashMap::new(),
            semaphore: Arc::new(Semaphore::new(concurrency)),
            cancellation_token: CancellationToken::new(),
            task_tracker: TaskTracker::new(),
        }
    }

    pub fn with_cancellation_token(mut self, token: CancellationToken) -> Self {
        self.cancellation_token = token;
        self
    }

    pub fn register_handler<J, H>(&mut self, handler: H)
    where
        J: Job,
        H: Handler<J, Ctx>,
    {
        self.handlers.insert(
            J::NAME.to_string(),
            Arc::new(HandlerWrapper {
                _phantom: std::marker::PhantomData,
                handler,
            }),
        );
    }

    pub async fn run(&self) {
        // Initial recovery on startup
        if let Err(e) = self.recover_stuck_jobs().await {
            tracing::error!("❌ Failed to recover stuck jobs on startup: {}", e);
        }

        // Setup Postgres LISTEN if possible
        let mut listener = if let Ok(url) = std::env::var("DATABASE_URL") {
            if url.starts_with("postgres") {
                match sqlx::postgres::PgListener::connect(&url).await {
                    Ok(mut l) => {
                        if let Err(e) = l.listen("background_jobs_channel").await {
                            tracing::warn!("⚠️ Failed to listen on background_jobs_channel: {}", e);
                            None
                        } else {
                            tracing::info!(
                                "📡 Listening for job notifications on background_jobs_channel"
                            );
                            Some(l)
                        }
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ Failed to connect PgListener: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let mut interval = tokio::time::interval(Duration::from_secs(30)); // Poll less frequently when LISTEN is active
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.process_queued_jobs().await {
                        tracing::error!("❌ Background worker pool failed: {}", e);
                    }

                    // Periodic recovery check
                    if Utc::now().second() % 60 == 0 {
                        if let Err(e) = self.recover_stuck_jobs().await {
                            tracing::error!("❌ Background recovery failed: {}", e);
                        }
                    }
                }
                notification = async {
                    if let Some(l) = listener.as_mut() {
                        l.recv().await.ok()
                    } else {
                        std::future::pending::<Option<sqlx::postgres::PgNotification>>().await
                    }
                } => {
                    if notification.is_some() {
                        if let Err(e) = self.process_queued_jobs().await {
                            tracing::error!("❌ Background worker pool failed (notify): {}", e);
                        }
                    }
                }
                _ = self.cancellation_token.cancelled() => {
                    tracing::info!("🛑 Worker pool received shutdown signal, waiting for jobs to finish...");
                    break;
                }
            }
        }

        self.task_tracker.close();
        self.task_tracker.wait().await;
        tracing::info!("✅ Worker pool shutdown complete.");
    }

    async fn recover_stuck_jobs(&self) -> Result<()> {
        let ten_minutes_ago = (Utc::now() - Duration::from_secs(600)).naive_utc();

        // Reset RUNNING jobs that haven't been updated for 10 minutes
        let result = background_jobs::Entity::update_many()
            .col_expr(
                background_jobs::Column::Status,
                sea_orm::sea_query::Expr::value(JobStatus::Queued.to_string()),
            )
            .col_expr(
                background_jobs::Column::StartedAt,
                sea_orm::sea_query::Expr::value(Option::<chrono::NaiveDateTime>::None),
            )
            .col_expr(
                background_jobs::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(Utc::now().naive_utc()),
            )
            .filter(background_jobs::Column::Status.eq(JobStatus::Running.to_string()))
            .filter(background_jobs::Column::UpdatedAt.lt(ten_minutes_ago))
            .exec(self.db.as_ref())
            .await?;

        if result.rows_affected > 0 {
            tracing::warn!(
                "🔄 Re-queued {} stale background jobs",
                result.rows_affected
            );
        }

        Ok(())
    }

    async fn process_queued_jobs(&self) -> Result<()> {
        let now = Utc::now().naive_utc();

        // 1. Fetch available jobs in batches to avoid memory issues and long locks
        let queued_jobs = background_jobs::Entity::find()
            .filter(background_jobs::Column::Status.eq(JobStatus::Queued.to_string()))
            .filter(background_jobs::Column::RunAt.lte(now))
            .limit(100)
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
            let context = self.context.clone();
            let job_id = job.id.clone();
            let payload = job.payload.clone();

            self.task_tracker.spawn(async move {
                let _permit = permit;

                // Panic recovery
                let result = std::panic::AssertUnwindSafe(execute_job(
                    db.clone(),
                    context,
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
                        let _ = mark_failed(db, job_id, "Panic occurred".to_string()).await;
                    }
                }
            });
        }

        Ok(())
    }
}

async fn execute_job<Ctx>(
    db: Arc<DatabaseConnection>,
    context: Ctx,
    job_id: String,
    payload: serde_json::Value,
    handler: Arc<dyn ErasedHandler<Ctx>>,
) -> Result<()> {
    // 1. Mark as Running using a transaction to avoid race conditions
    let txn = db.begin().await?;

    let job = background_jobs::Entity::find_by_id(job_id.clone())
        .one(&txn)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Job not found"))?;

    if job.status != JobStatus::Queued.to_string() {
        txn.rollback().await?;
        return Ok(());
    }

    let mut active_model: background_jobs::ActiveModel = job.into();
    active_model.status = Set(JobStatus::Running.to_string());
    active_model.started_at = Set(Some(Utc::now().naive_utc()));
    active_model.updated_at = Set(Utc::now().naive_utc());
    active_model.update(&txn).await?;
    txn.commit().await?;

    // 2. Execute
    match handler.handle(&context, payload).await {
        Ok(_) => {
            let active_model = background_jobs::ActiveModel {
                id: Set(job_id),
                status: Set(JobStatus::Completed.to_string()),
                completed_at: Set(Some(Utc::now().naive_utc())),
                updated_at: Set(Utc::now().naive_utc()),
                ..Default::default()
            };
            active_model.update(db.as_ref()).await?;
        }
        Err(e) => {
            handle_job_error(db, job_id, e).await?;
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
    let (status, run_at) = if next_attempts >= job.max_attempts {
        (JobStatus::Failed, job.run_at)
    } else {
        // Exponential backoff: 10s * 2^(attempts-1) -> 10s, 20s, 40s...
        let delay = 10 * 2u64.pow(next_attempts as u32 - 1);
        let next_run = Utc::now() + Duration::from_secs(delay);
        (JobStatus::Queued, next_run.naive_utc())
    };

    let active_model = background_jobs::ActiveModel {
        id: Set(job_id),
        status: Set(status.to_string()),
        attempts: Set(next_attempts),
        run_at: Set(run_at),
        error: Set(Some(error.to_string())),
        updated_at: Set(Utc::now().naive_utc()),
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
        updated_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    active_model.update(db.as_ref()).await?;
    Ok(())
}

use futures::FutureExt;
