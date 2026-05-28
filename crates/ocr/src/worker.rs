use crate::{OcrService, OcrUpdate};
use chrono::Utc;
use db::entities;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use upload::UploadClient;

pub async fn start_recovery_worker(db: Arc<DatabaseConnection>, token: CancellationToken) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = recover_stale_jobs(&db).await {
                    tracing::error!("❌ Recovery worker failed: {}", e);
                }
            }
            _ = token.cancelled() => break,
        }
    }
}

// Worker bootstrap intentionally wires together all collaborators required for
// the OCR pipeline (DB, service, uploader, broadcast tx, processor, semaphore,
// shutdown token, task tracker). A struct wrapper would just move the arity
// without improving the call site.
#[allow(clippy::too_many_arguments)]
pub async fn start_processor_worker(
    db: Arc<DatabaseConnection>,
    ocr_service: Arc<OcrService>,
    upload_client: UploadClient,
    ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
    processor: Arc<dyn crate::OcrProcessor>,
    semaphore: Arc<Semaphore>,
    token: CancellationToken,
    tracker: TaskTracker,
) {
    // Setup Postgres LISTEN if possible
    let mut listener = if let Ok(url) = std::env::var("DATABASE_URL") {
        if url.starts_with("postgres") {
            match sqlx::postgres::PgListener::connect(&url).await {
                Ok(mut l) => {
                    if let Err(e) = l.listen("ocr_jobs_channel").await {
                        tracing::warn!("⚠️ Failed to listen on ocr_jobs_channel: {}", e);
                        None
                    } else {
                        tracing::info!("📡 Listening for job notifications on ocr_jobs_channel");
                        Some(l)
                    }
                }
                Err(e) => {
                    tracing::warn!("⚠️ Failed to connect PgListener (OCR): {}", e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    let mut interval = tokio::time::interval(Duration::from_secs(30)); // Poll every 30 seconds

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = process_queued_jobs(
                    db.clone(),
                    ocr_service.clone(),
                    &upload_client,
                    ocr_tx.clone(),
                    processor.clone(),
                    semaphore.clone(),
                    &tracker,
                )
                .await
                {
                    tracing::error!("❌ Processor worker failed: {}", e);
                }
            }
            notification = async {
                if let Some(l) = listener.as_mut() {
                    l.recv().await.ok()
                } else {
                    std::future::pending::<Option<sqlx::postgres::PgNotification>>().await
                }
            } => {
                if notification.is_some()
                    && let Err(e) = process_queued_jobs(
                        db.clone(),
                        ocr_service.clone(),
                        &upload_client,
                        ocr_tx.clone(),
                        processor.clone(),
                        semaphore.clone(),
                        &tracker,
                    )
                    .await
                    {
                        tracing::error!("❌ Processor worker failed (notify): {}", e);
                    }
            }
            _ = token.cancelled() => {
                tracing::info!("🛑 OCR processor worker received shutdown signal...");
                break;
            }
        }
    }

    tracker.close();
    tracker.wait().await;
}

async fn process_queued_jobs(
    db: Arc<DatabaseConnection>,
    ocr_service: Arc<OcrService>,
    upload_client: &UploadClient,
    ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
    processor: Arc<dyn crate::OcrProcessor>,
    semaphore: Arc<Semaphore>,
    tracker: &TaskTracker,
) -> Result<(), anyhow::Error> {
    let now = Utc::now();

    // Find jobs in QUEUED status that are scheduled for now or in the past.
    // Bounded batch + FIFO ordering: under a large backlog (e.g. after an outage)
    // we would otherwise load every queued job into memory at once, while
    // concurrency is already capped by the semaphore. The next poll picks up
    // the next batch.
    const QUEUE_BATCH_SIZE: u64 = 100;
    let queued_jobs = entities::ocr_jobs::Entity::find()
        .filter(entities::ocr_jobs::Column::Status.eq("QUEUED"))
        .filter(
            entities::ocr_jobs::Column::ScheduledAt
                .is_null()
                .or(entities::ocr_jobs::Column::ScheduledAt.lte(now)),
        )
        .order_by_asc(entities::ocr_jobs::Column::ScheduledAt)
        .order_by_asc(entities::ocr_jobs::Column::CreatedAt)
        .limit(QUEUE_BATCH_SIZE)
        .all(&*db)
        .await?;

    for job in queued_jobs {
        let job_id = job.id.clone();

        // Wait for an available permit before spawning
        let permit = semaphore.clone().acquire_owned().await;

        if let Ok(permit) = permit {
            tracing::info!("👷 Background worker picking up job: {}", job_id);

            let db_clone = db.clone();
            let ocr_service_clone = ocr_service.clone();
            let upload_client_clone = upload_client.clone();
            let ocr_tx_clone = ocr_tx.clone();
            let processor_clone = processor.clone();

            tracker.spawn(async move {
                // The permit is held until this task finishes
                let _permit = permit;

                if let Err(e) = crate::process_job(
                    &db_clone,
                    ocr_service_clone,
                    &upload_client_clone,
                    ocr_tx_clone,
                    processor_clone,
                    job_id,
                )
                .await
                {
                    tracing::error!("❌ Background job processing failed: {}", e);
                }
            });
        }
    }

    Ok(())
}

async fn recover_stale_jobs(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
    use crate::ops::lifecycle::MAX_OCR_RETRIES;
    use sea_orm::sea_query::Expr;

    let ten_minutes_ago = Utc::now() - chrono::Duration::minutes(10);

    // 1. Dead-letter stale jobs that have already exhausted their retry budget,
    //    so a perpetually-hanging job cannot bounce PROCESSING -> QUEUED forever.
    let dead = entities::ocr_jobs::Entity::update_many()
        .col_expr(
            entities::ocr_jobs::Column::Status,
            Expr::value("DEAD_LETTER".to_string()),
        )
        .col_expr(
            entities::ocr_jobs::Column::LastError,
            Expr::value("Stale processing timeout".to_string()),
        )
        .filter(entities::ocr_jobs::Column::Status.eq("PROCESSING"))
        .filter(entities::ocr_jobs::Column::StartedAt.lt(ten_minutes_ago))
        .filter(entities::ocr_jobs::Column::RetryCount.gte(MAX_OCR_RETRIES))
        .exec(db)
        .await?;

    // 2. Re-queue the remaining stale jobs, charging one attempt against the budget.
    let requeued = entities::ocr_jobs::Entity::update_many()
        .col_expr(
            entities::ocr_jobs::Column::Status,
            Expr::value("QUEUED".to_string()),
        )
        .col_expr(
            entities::ocr_jobs::Column::StartedAt,
            Expr::value(Option::<chrono::DateTime<chrono::Utc>>::None),
        )
        .col_expr(
            entities::ocr_jobs::Column::RetryCount,
            Expr::col(entities::ocr_jobs::Column::RetryCount).add(1),
        )
        .filter(entities::ocr_jobs::Column::Status.eq("PROCESSING"))
        .filter(entities::ocr_jobs::Column::StartedAt.lt(ten_minutes_ago))
        .exec(db)
        .await?;

    if dead.rows_affected > 0 {
        tracing::warn!(
            "💀 Dead-lettered {} stale OCR jobs (retry budget exhausted)",
            dead.rows_affected
        );
    }
    if requeued.rows_affected > 0 {
        tracing::warn!("🔄 Re-queued {} stale OCR jobs", requeued.rows_affected);
    }

    Ok(())
}
