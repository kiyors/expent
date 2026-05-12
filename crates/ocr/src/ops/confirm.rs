use crate::OcrUpdate;
use crate::ops::lifecycle::{OcrJobUpdateParams, update_ocr_job};
use crate::ops::process::OcrProcessor;
use chrono::Utc;
use db::AppError;
use db::entities;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::sync::Arc;

pub async fn log_ocr_edits(
    db: &DatabaseConnection,
    user_id: &str,
    job_id: &str,
    original: &serde_json::Value,
    corrected: &serde_json::Value,
) -> Result<(), AppError> {
    if let (Some(orig_obj), Some(corr_obj)) = (original.as_object(), corrected.as_object()) {
        for (key, corr_val) in corr_obj {
            let orig_val = orig_obj.get(key).unwrap_or(&serde_json::Value::Null);
            if orig_val != corr_val && !corr_val.is_object() && !corr_val.is_array() {
                let edit = entities::ocr_job_edits::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    ocr_job_id: Set(job_id.to_string()),
                    user_id: Set(user_id.to_string()),
                    field_name: Set(key.clone()),
                    original_value: Set(Some(orig_val.to_string())),
                    corrected_value: Set(Some(corr_val.to_string())),
                    created_at: Set(Utc::now().naive_utc()),
                };
                edit.insert(db).await?;
            }
        }
    }
    Ok(())
}

pub async fn confirm_ocr_job(
    db: &DatabaseConnection,
    ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
    processor: Arc<dyn OcrProcessor>,
    user_id: &str,
    job_id: &str,
    corrected_data: serde_json::Value,
) -> Result<db::OcrTransactionResponse, AppError> {
    let job = entities::ocr_jobs::Entity::find_by_id(job_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("OCR Job not found"))?;

    if job.user_id != user_id {
        return Err(AppError::unauthorized("You don't own this job"));
    }

    // 1. Log edits if original data exists
    if let Some(original) = job.processed_data.clone() {
        let _ = log_ocr_edits(db, user_id, job_id, &original, &corrected_data).await;
    }

    // 2. Process the transaction
    let processed: db::ProcessedOcr = serde_json::from_value(corrected_data.clone())
        .map_err(|e| AppError::Ocr(format!("Invalid corrected data: {}", e)))?;

    let res = processor.process_ocr(db, user_id, processed).await?;

    // 3. Update job status
    update_ocr_job(
        db,
        job_id,
        OcrJobUpdateParams {
            status: "COMPLETED".to_string(),
            processed_data: Some(corrected_data),
            error_message: None,
            transaction_id: Some(res.transaction.id.clone()),
            started_at: None,
            retry_count: None,
            last_error: None,
            scheduled_at: None,
            resolution_candidates: None,
        },
    )
    .await?;

    let _ = ocr_tx.send(OcrUpdate {
        user_id: user_id.to_string(),
        job_id: job_id.to_string(),
        status: "COMPLETED".to_string(),
        trace_id: job.trace_id,
    });

    Ok(res)
}

pub async fn resolve_contact_collision(
    db: &DatabaseConnection,
    ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
    processor: Arc<dyn OcrProcessor>,
    user_id: &str,
    job_id: &str,
    contact_id: &str,
) -> Result<db::OcrTransactionResponse, AppError> {
    let job = entities::ocr_jobs::Entity::find_by_id(job_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("OCR Job not found"))?;

    if job.user_id != user_id {
        return Err(AppError::unauthorized("You don't own this job"));
    }

    let mut processed: db::ProcessedOcr = job
        .processed_data
        .clone()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| AppError::Ocr(format!("Corrupt job data: {}", e)))?
        .ok_or_else(|| AppError::not_found("No processed data found"))?;

    // Inject the selected contact_id into the extraction data
    match processed.doc_type.as_str() {
        "GPAY" => {
            let mut gpay: db::GPayExtraction = serde_json::from_value(processed.data.0.clone())
                .map_err(|e| AppError::Ocr(format!("Invalid GPAY data: {}", e)))?;
            gpay.contact_id = Some(contact_id.to_string());
            processed.data.0 = serde_json::to_value(gpay).unwrap();
        }
        "GENERIC" => {
            let mut generic: db::OcrResult = serde_json::from_value(processed.data.0.clone())
                .map_err(|e| AppError::Ocr(format!("Invalid GENERIC data: {}", e)))?;
            generic.contact_id = Some(contact_id.to_string());
            processed.data.0 = serde_json::to_value(generic).unwrap();
        }
        _ => {}
    }

    let res = processor
        .process_ocr(db, user_id, processed.clone())
        .await?;

    // Update job status
    update_ocr_job(
        db,
        job_id,
        OcrJobUpdateParams {
            status: "COMPLETED".to_string(),
            processed_data: Some(serde_json::to_value(processed).unwrap()),
            error_message: None,
            transaction_id: Some(res.transaction.id.clone()),
            started_at: None,
            retry_count: None,
            last_error: None,
            scheduled_at: None,
            resolution_candidates: None,
        },
    )
    .await?;

    let _ = ocr_tx.send(OcrUpdate {
        user_id: user_id.to_string(),
        job_id: job_id.to_string(),
        status: "COMPLETED".to_string(),
        trace_id: job.trace_id,
    });

    Ok(res)
}
