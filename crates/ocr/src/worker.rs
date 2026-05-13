use crate::{OcrService, OcrUpdate};
use chrono::Utc;
use db::entities;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use upload::UploadClient;

pub async fn start_recovery_worker(db: Arc<DatabaseConnection>) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
    loop {
        interval.tick().await;
        if let Err(e) = recover_stale_jobs(&*db).await {
            tracing::error!("❌ Recovery worker failed: {}", e);
        }
    }
}

pub async fn start_processor_worker(
    db: Arc<DatabaseConnection>,
    ocr_service: Arc<OcrService>,
    upload_client: UploadClient,
    ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
    processor: Arc<dyn crate::OcrProcessor>,
    semaphore: Arc<Semaphore>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(10)); // Poll every 10 seconds

    loop {
        interval.tick().await;
        if let Err(e) = process_queued_jobs(
            db.clone(),
            ocr_service.clone(),
            &upload_client,
            ocr_tx.clone(),
            processor.clone(),
            semaphore.clone(),
        )
        .await
        {
            tracing::error!("❌ Processor worker failed: {}", e);
        }
    }
}

async fn process_queued_jobs(
    db: Arc<DatabaseConnection>,
    ocr_service: Arc<OcrService>,
    upload_client: &UploadClient,
    ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
    processor: Arc<dyn crate::OcrProcessor>,
    semaphore: Arc<Semaphore>,
) -> Result<(), anyhow::Error> {
    let now = Utc::now();

    // Find jobs in QUEUED status that are scheduled for now or in the past
    let queued_jobs = entities::ocr_jobs::Entity::find()
        .filter(entities::ocr_jobs::Column::Status.eq("QUEUED"))
        .filter(
            entities::ocr_jobs::Column::ScheduledAt
                .is_null()
                .or(entities::ocr_jobs::Column::ScheduledAt.lte(now)),
        )
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

            tokio::spawn(async move {
                // The permit is held until this task finishes
                let _permit = permit;

                if let Err(e) = crate::process_job(
                    &*db_clone,
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
    let ten_minutes_ago = Utc::now() - chrono::Duration::minutes(10);

    // Re-queue jobs that have been PROCESSING for more than 10 minutes
    let result = entities::ocr_jobs::Entity::update_many()
        .col_expr(
            entities::ocr_jobs::Column::Status,
            sea_orm::sea_query::Expr::value("QUEUED".to_string()),
        )
        .col_expr(
            entities::ocr_jobs::Column::StartedAt,
            sea_orm::sea_query::Expr::value(Option::<chrono::DateTime<chrono::Utc>>::None),
        )
        .filter(entities::ocr_jobs::Column::Status.eq("PROCESSING"))
        .filter(entities::ocr_jobs::Column::StartedAt.lt(ten_minutes_ago))
        .exec(db)
        .await?;

    if result.rows_affected > 0 {
        tracing::warn!("🔄 Re-queued {} stale OCR jobs", result.rows_affected);
    }

    Ok(())
}
