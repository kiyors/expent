use crate::OcrService;
use crate::OcrUpdate;
use crate::ops::lifecycle::{OcrJobUpdateParams, update_ocr_job};
use db::AppError;
use db::entities;
use rand::Rng;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::sync::Arc;
use upload::UploadClient;

/// Helper function to bridge with expent_core for processing transactions.
/// This will be provided by a trait or callback to keep the ocr crate decoupled.
pub trait OcrProcessor: Send + Sync {
    fn process_ocr(
        &self,
        db: &DatabaseConnection,
        user_id: &str,
        processed: db::ProcessedOcr,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<db::OcrTransactionResponse, AppError>> + Send>,
    >;
}

pub async fn process_job(
    db: &DatabaseConnection,
    ocr_service: Arc<OcrService>,
    upload_client: &UploadClient,
    ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
    processor: Arc<dyn OcrProcessor>,
    job_id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let job = entities::ocr_jobs::Entity::find_by_id(job_id.clone())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("OCR Job not found"))?;

    if job.status != "QUEUED" && job.status != "PENDING" {
        return Ok(());
    }

    let user_id = job.user_id.clone();
    let trace_id = job.trace_id.clone();

    // Use raw_key if it's already a high-res attempt or if we want to try high-res
    let key = if job.is_high_res {
        job.raw_key.as_ref().unwrap_or(&job.r2_key).clone()
    } else {
        job.r2_key.clone()
    };

    // 1. Update status to PROCESSING
    update_ocr_job(
        db,
        &job_id,
        OcrJobUpdateParams {
            status: "PROCESSING".to_string(),
            processed_data: None,
            error_message: None,
            transaction_id: None,
            started_at: Some(chrono::Utc::now()),
            retry_count: None,
            last_error: None,
            scheduled_at: None,
            resolution_candidates: None,
        },
    )
    .await?;

    let _ = ocr_tx.send(OcrUpdate {
        user_id: user_id.clone(),
        job_id: job_id.clone(),
        status: "PROCESSING".to_string(),
        trace_id: trace_id.clone(),
    });

    let process_res = async {
        let bytes = upload_client.get_file(&key).await?;

        // Determine filename and mime type from the key
        let filename = key.split("/").last().unwrap_or("upload");
        let mime_type = if filename.ends_with(".pdf") {
            "application/pdf"
        } else if filename.ends_with(".csv") {
            "text/csv"
        } else if filename.ends_with(".webp") {
            "image/webp"
        } else {
            "image/png"
        };

        let ocr_json = ocr_service
            .process_file(&bytes, filename, mime_type)
            .await?;

        let mut processed_ocr: db::ProcessedOcr = serde_json::from_value(ocr_json.clone())?;
        processed_ocr.r2_key = Some(job.r2_key.clone());
        processed_ocr.is_high_res = job.is_high_res;

        // Extract confidence from the inner data
        let confidence_score = match processed_ocr.doc_type.as_str() {
            "GPAY" => {
                let gpay: db::GPayExtraction =
                    serde_json::from_value(processed_ocr.data.0.clone())?;
                gpay.confidence_score
            }
            "GENERIC" => {
                let generic: db::OcrResult = serde_json::from_value(processed_ocr.data.0.clone())?;
                generic.confidence_score
            }
            "BANK_STATEMENT" => {
                let bank: db::BankExtractionResult =
                    serde_json::from_value(processed_ocr.data.0.clone())?;
                bank.confidence_score
            }
            _ => 1.0,
        };

        // 3. Progressive Quality Fallback
        if confidence_score < 0.8 && !job.is_high_res && job.raw_key.is_some() {
            tracing::info!(
                "⚠️ Low confidence ({}) for job {}, triggering high-res retry",
                confidence_score,
                job_id
            );
            return Ok::<
                (
                    db::ProcessedOcr,
                    String,
                    Option<String>,
                    Option<serde_json::Value>,
                ),
                Box<dyn std::error::Error + Send + Sync>,
            >((processed_ocr, "RETRY_HIGH_RES".to_string(), None, None));
        }

        let mut transaction_id = None;
        let mut final_status = "COMPLETED";
        let mut collision_data = None;

        if job.auto_confirm {
            if let Some(w_id) = job.wallet_id {
                match processed_ocr.doc_type.as_str() {
                    "GPAY" => {
                        if let Ok(mut gpay) = serde_json::from_value::<db::GPayExtraction>(
                            processed_ocr.data.0.clone(),
                        ) {
                            gpay.wallet_id = Some(w_id);
                            if let Some(c_id) = job.category_id.clone() {
                                gpay.category_id = Some(c_id);
                            }
                            processed_ocr.data.0 = serde_json::to_value(gpay).map_err(|e| {
                                AppError::Ocr(format!("Failed to serialize GPAY data: {}", e))
                            })?;
                        }
                    }
                    "GENERIC" => {
                        if let Ok(mut generic) =
                            serde_json::from_value::<db::OcrResult>(processed_ocr.data.0.clone())
                        {
                            generic.wallet_id = Some(w_id);
                            if let Some(c_id) = job.category_id.clone() {
                                generic.category_id = Some(c_id);
                            }
                            processed_ocr.data.0 = serde_json::to_value(generic).map_err(|e| {
                                AppError::Ocr(format!("Failed to serialize GENERIC data: {}", e))
                            })?;
                        }
                    }
                    "BANK_STATEMENT" => {
                        if let Ok(mut bank) = serde_json::from_value::<db::BankExtractionResult>(
                            processed_ocr.data.0.clone(),
                        ) {
                            for tx in &mut bank.bank_data.transactions {
                                tx.wallet_id = Some(w_id.clone());
                                if let Some(c_id) = job.category_id.clone() {
                                    tx.category_id = Some(c_id);
                                }
                            }
                            processed_ocr.data.0 = serde_json::to_value(bank).map_err(|e| {
                                AppError::Ocr(format!("Failed to serialize bank data: {}", e))
                            })?;
                        }
                    }
                    _ => {}
                }
            }

            match processor
                .process_ocr(db, &user_id, processed_ocr.clone())
                .await
            {
                Ok(res) => {
                    transaction_id = Some(res.transaction.id);
                }
                Err(e) => {
                    if let db::AppError::ContactCollision(candidates) = e {
                        tracing::warn!(
                            "⚠️ Contact collision for job {}, needs manual review",
                            job_id
                        );
                        final_status = "CONTACT_COLLISION";
                        collision_data = Some(candidates);
                    } else {
                        tracing::error!("❌ Auto-confirmation failed for job {}: {}", job_id, e);
                        final_status = "PENDING_REVIEW";
                    }
                }
            }
        } else {
            final_status = "PENDING_REVIEW";
        }

        Ok::<
            (
                db::ProcessedOcr,
                String,
                Option<String>,
                Option<serde_json::Value>,
            ),
            Box<dyn std::error::Error + Send + Sync>,
        >((
            processed_ocr,
            final_status.to_string(),
            transaction_id,
            collision_data,
        ))
    }
    .await;

    match process_res {
        Ok((processed, status, tx_id, candidates)) => {
            if status == "RETRY_HIGH_RES" {
                update_ocr_job(
                    db,
                    &job_id,
                    OcrJobUpdateParams {
                        status: "QUEUED".to_string(),
                        processed_data: None,
                        error_message: None,
                        transaction_id: None,
                        started_at: None,
                        retry_count: None,
                        last_error: None,
                        scheduled_at: Some(chrono::Utc::now()),
                        resolution_candidates: None,
                    },
                )
                .await?;

                let _ = ocr_tx.send(OcrUpdate {
                    user_id: user_id.clone(),
                    job_id: job_id.clone(),
                    status: "QUEUED".to_string(),
                    trace_id: trace_id.clone(),
                });
            } else {
                update_ocr_job(
                    db,
                    &job_id,
                    OcrJobUpdateParams {
                        status: status.to_string(),
                        processed_data: Some(serde_json::to_value(processed).map_err(|e| {
                            AppError::Ocr(format!("Failed to serialize processed OCR data: {}", e))
                        })?),
                        error_message: None,
                        transaction_id: tx_id,
                        started_at: None,
                        retry_count: None,
                        last_error: None,
                        scheduled_at: None,
                        resolution_candidates: candidates,
                    },
                )
                .await?;

                let _ = ocr_tx.send(OcrUpdate {
                    user_id,
                    job_id: job_id.clone(),
                    status: status.to_string(),
                    trace_id,
                });
            }
        }
        Err(e) => {
            tracing::error!("❌ OCR Background Job {} failed: {}", job_id, e);

            let new_retry_count = job.retry_count + 1;
            let max_retries = 5;

            if new_retry_count < max_retries {
                let base_delay = 10;
                let backoff_secs = base_delay * (2_i64.pow(new_retry_count as u32));
                let jitter = rand::thread_rng().gen_range(0..5);
                let next_run =
                    chrono::Utc::now() + chrono::Duration::seconds(backoff_secs + jitter);

                update_ocr_job(
                    db,
                    &job_id,
                    OcrJobUpdateParams {
                        status: "QUEUED".to_string(),
                        processed_data: None,
                        error_message: None,
                        transaction_id: None,
                        started_at: None,
                        retry_count: Some(new_retry_count),
                        last_error: Some(e.to_string()),
                        scheduled_at: Some(next_run),
                        resolution_candidates: None,
                    },
                )
                .await?;

                let _ = ocr_tx.send(OcrUpdate {
                    user_id: user_id.clone(),
                    job_id: job_id.clone(),
                    status: "QUEUED".to_string(),
                    trace_id,
                });
            } else {
                update_ocr_job(
                    db,
                    &job_id,
                    OcrJobUpdateParams {
                        status: "DEAD_LETTER".to_string(),
                        processed_data: None,
                        error_message: Some(e.to_string()),
                        transaction_id: None,
                        started_at: None,
                        retry_count: Some(new_retry_count),
                        last_error: Some(e.to_string()),
                        scheduled_at: None,
                        resolution_candidates: None,
                    },
                )
                .await?;

                let _ = ocr_tx.send(OcrUpdate {
                    user_id,
                    job_id: job_id.clone(),
                    status: "DEAD_LETTER".to_string(),
                    trace_id,
                });
            }
        }
    }

    Ok(())
}
