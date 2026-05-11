use db::entities;
use db::AppError;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;
use upload::UploadClient;

pub async fn confirm_ocr_job(
    db: &DatabaseConnection,
    upload_client: &UploadClient,
    processor: Arc<dyn crate::OcrProcessor>,
    user_id: &str,
    job_id: &str,
    manual_data: Option<db::ProcessedOcr>,
) -> Result<db::OcrTransactionResponse, AppError> {
    db.transaction::<_, db::OcrTransactionResponse, AppError>(|txn_db| {
        let upload_client = upload_client.clone();
        let processor = processor.clone();
        let user_id = user_id.to_string();
        let job_id = job_id.to_string();
        Box::pin(async move {
            let job = super::lifecycle::get_ocr_job(txn_db, &job_id)
                .await?
                .ok_or_else(|| AppError::NotFound)?;

            let mut processed_ocr = if let Some(data) = manual_data {
                data
            } else {
                job.processed_data
                    .map(|v| serde_json::from_value(v.0).ok())
                    .flatten()
                    .ok_or_else(|| AppError::Generic("Processed data not found".to_string()))?
            };

            // Log edits if any
            // TODO: Compare manual_data with original processed_data and log diffs

            let transaction_response = processor
                .process_ocr(txn_db, &user_id, processed_ocr.clone())
                .await?;

            super::lifecycle::update_ocr_job(
                txn_db,
                &job.id,
                super::lifecycle::OcrJobUpdateParams {
                    status: "COMPLETED".to_string(),
                    processed_data: Some(serde_json::to_value(processed_ocr)?),
                    error_message: None,
                    transaction_id: Some(transaction_response.transaction.id.clone()),
                    started_at: None,
                    retry_count: None,
                    last_error: None,
                    scheduled_at: None,
                    resolution_candidates: None,
                },
            )
            .await?;

            Ok(transaction_response)
        })
    })
    .await
}

pub async fn resolve_contact_collision(
    db: &DatabaseConnection,
    upload_client: &UploadClient,
    processor: Arc<dyn crate::OcrProcessor>,
    user_id: &str,
    job_id: &str,
    contact_id: &str,
) -> Result<db::OcrTransactionResponse, AppError> {
    db.transaction::<_, db::OcrTransactionResponse, AppError>(|txn_db| {
        let upload_client = upload_client.clone();
        let processor = processor.clone();
        let user_id = user_id.to_string();
        let job_id = job_id.to_string();
        let contact_id = contact_id.to_string();
        Box::pin(async move {
            let job = super::lifecycle::get_ocr_job(txn_db, &job_id)
                .await?
                .ok_or_else(|| AppError::NotFound)?;

            let mut processed_ocr: db::ProcessedOcr = job
                .processed_data
                .map(|v| serde_json::from_value(v.0).ok())
                .flatten()
                .ok_or_else(|| AppError::Generic("Processed data not found".to_string()))?;

            // Apply resolved contact_id to the OCR result
            match processed_ocr.doc_type.as_str() {
                "GPAY" => {
                    if let Ok(mut gpay) =
                        serde_json::from_value::<db::GPayExtraction>(processed_ocr.data.0.clone())
                    {
                        gpay.contact_id = Some(contact_id.clone());
                        processed_ocr.data.0 = serde_json::to_value(gpay)?;
                    }
                }
                "BANK_STATEMENT" => {
                    if let Ok(mut bank_extraction) = serde_json::from_value::<
                        db::BankExtractionResult,
                    >(processed_ocr.data.0.clone())
                    {
                        // Apply resolved contact_id to all transactions if needed
                        for tx in &mut bank_extraction.bank_data.transactions {
                            // This might need more sophisticated logic to apply to the correct transactions
                            // For simplicity, we assume the resolution applies to the whole statement for now
                            if tx.contact_name.is_some() {
                                tx.metadata
                                    .get_or_insert(serde_json::json!({}))
                                    .as_object_mut()
                                    .map(|obj| obj.insert("resolved_contact_id".to_string(), serde_json::json!(contact_id.clone())));
                            }
                        }
                        processed_ocr.data.0 = serde_json::to_value(bank_extraction)?;
                    }
                }
                "GENERIC" => {
                    if let Ok(mut generic) =
                        serde_json::from_value::<db::OcrResult>(processed_ocr.data.0.clone())
                    {
                        generic.contact_id = Some(contact_id.clone());
                        processed_ocr.data.0 = serde_json::to_value(generic)?;
                    }
                }
                _ => {}
            }

            let transaction_response = processor
                .process_ocr(txn_db, &user_id, processed_ocr.clone())
                .await?;

            super::lifecycle::update_ocr_job(
                txn_db,
                &job.id,
                super::lifecycle::OcrJobUpdateParams {
                    status: "COMPLETED".to_string(),
                    processed_data: Some(serde_json::to_value(processed_ocr)?),
                    error_message: None,
                    transaction_id: Some(transaction_response.transaction.id.clone()),
                    started_at: None,
                    retry_count: None,
                    last_error: None,
                    scheduled_at: None,
                    resolution_candidates: None,
                },
            )
            .await?;

            Ok(transaction_response)
        })
    })
    .await
}
