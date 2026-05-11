use db::entities;
use db::AppError;
use sea_orm::{DatabaseConnection, EntityTrait, Set};
use std::sync::Arc;
use upload::UploadClient;

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
    ocr_tx: tokio::sync::broadcast::Sender<crate::OcrUpdate>,
    processor: Arc<dyn OcrProcessor>,
    job_id: String,
) -> Result<(), anyhow::Error> {
    let mut job = super::lifecycle::get_ocr_job(db, &job_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("OCR Job not found"))?;

    super::lifecycle::update_ocr_job(
        db,
        &job.id,
        "PROCESSING",
        None,
        None,
        None,
        Some(chrono::Utc::now()),
        None,
        None,
        None,
        None,
    )
    .await?;

    let bytes = upload_client.get_file(&job.r2_key).await?;

    let filename = job.r2_key.split('/').last().unwrap_or("upload");
    let mime_type = super::process::determine_mime_type(filename);

    let ocr_json = ocr_service
        .process_file(bytes, filename, mime_type)
        .await?;

    let mut processed_ocr: db::ProcessedOcr = serde_json::from_value(ocr_json)?;
    processed_ocr.r2_key = Some(job.r2_key.clone());
    processed_ocr.is_high_res = job.is_high_res;

    let confidence_score = super::process::extract_confidence(&processed_ocr);

    if confidence_score < 0.5 {
        super::lifecycle::update_ocr_job(
            db,
            &job.id,
            super::lifecycle::OcrJobUpdateParams {
                status: "FAILED".to_string(),
                processed_data: None,
                error_message: Some("Low confidence score".to_string()),
                transaction_id: None,
                started_at: None,
                retry_count: None,
                last_error: None,
                scheduled_at: None,
                resolution_candidates: None,
            },
        )
        .await?;        return Ok(());
    }

    if job.auto_confirm && confidence_score > 0.8 {
        // Apply wallet and category if auto-confirm is enabled and confident enough
        super::process::prepare_auto_confirm(&mut processed_ocr, &job);

        // Try to process immediately and create transaction
        let transaction_response = processor
            .process_ocr(db, &job.user_id, processed_ocr.clone())
            .await?;

        super::lifecycle::update_ocr_job(
            db,
            &job.id,
            super::lifecycle::OcrJobUpdateParams {
                status: "COMPLETED".to_string(),
                processed_data: Some(serde_json::to_value(processed_ocr)?),
                error_message: None,
                transaction_id: Some(transaction_response.transaction.id),
                started_at: None,
                retry_count: None,
                last_error: None,
                scheduled_at: None,
                resolution_candidates: None,
            },
        )
        .await?;
    } else {
        super::lifecycle::update_ocr_job(
            db,
            &job.id,
            "PROCESSED",
            Some(serde_json::to_value(processed_ocr)?),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;
    }

    ocr_tx
        .send(crate::OcrUpdate {
            user_id: job.user_id.clone(),
            job_id: job.id.clone(),
            status: "PROCESSED".to_string(),
            trace_id: job.trace_id,
        })
        .ok();

    Ok(())
}

fn determine_mime_type(filename: &str) -> &str {
    match filename.split('.').last().unwrap_or("").to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "pdf" => "application/pdf",
        "csv" => "text/csv",
        "xls" | "xlsx" => "application/vnd.ms-excel",
        _ => "application/octet-stream",
    }
}

fn extract_confidence(processed: &db::ProcessedOcr) -> f32 {
    let json_value = &processed.data.0;
    if processed.doc_type == "BANK_STATEMENT" {
        json_value
            .get("bank_data")
            .and_then(|bd| bd.get("confidence_score"))
            .and_then(|cs| cs.as_f64())
            .map(|cs| cs as f32)
            .unwrap_or(1.0)
    } else {
        json_value
            .get("confidence_score")
            .and_then(|cs| cs.as_f64())
            .map(|cs| cs as f32)
            .unwrap_or(1.0)
    }
}

fn prepare_auto_confirm(processed: &mut db::ProcessedOcr, job: &entities::ocr_jobs::Model) {
    if let Some(w_id) = job.wallet_id.clone() {
        let val = std::mem::take(&mut processed.data.0);
        match processed.doc_type.as_str() {
            "GPAY" => {
                if let Ok(mut gpay) = serde_json::from_value::<db::GPayExtraction>(val) {
                    gpay.wallet_id = Some(w_id);
                    if let Some(c_id) = job.category_id.clone() {
                        gpay.category_id = Some(c_id);
                    }
                    if let Ok(new_val) = serde_json::to_value(gpay) {
                        processed.data.0 = new_val;
                    }
                }
            }
            "GENERIC" => {
                if let Ok(mut generic) = serde_json::from_value::<db::OcrResult>(val) {
                    generic.wallet_id = Some(w_id);
                    if let Some(c_id) = job.category_id.clone() {
                        generic.category_id = Some(c_id);
                    }
                    if let Ok(new_val) = serde_json::to_value(generic) {
                        processed.data.0 = new_val;
                    }
                }
            }
            "BANK_STATEMENT" => {
                if let Ok(mut bank_extraction) =
                    serde_json::from_value::<db::BankExtractionResult>(val)
                {
                    // Apply wallet_id and category_id to all transactions in the batch
                    for tx in &mut bank_extraction.bank_data.transactions {
                        if tx.wallet_id.is_none() {
                            tx.wallet_id = Some(w_id.clone());
                        }
                        if tx.category_id.is_none() {
                            if let Some(c_id) = job.category_id.clone() {
                                tx.category_id = Some(c_id);
                            }
                        }
                    }
                    if let Ok(new_val) = serde_json::to_value(bank_extraction) {
                        processed.data.0 = new_val;
                    }
                }
            }
            _ => {
                processed.data.0 = val;
            }
        }
    }
}
            }
        }
    }
}
