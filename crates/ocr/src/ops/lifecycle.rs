use db::AppError;
use db::entities;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub const CURRENT_SCHEMA_VERSION: i32 = 1;

pub struct OcrJobUpdateParams {
    pub status: String,
    pub processed_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub transaction_id: Option<String>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub retry_count: Option<i32>,
    pub last_error: Option<String>,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub resolution_candidates: Option<serde_json::Value>,
}

pub async fn create_ocr_job(
    db: &DatabaseConnection,
    user_id: &str,
    trace_id: Option<String>,
    key: &str,
    raw_key: Option<String>,
    p_hash: Option<String>,
    auto_confirm: bool,
    wallet_id: Option<String>,
    category_id: Option<String>,
) -> Result<entities::ocr_jobs::Model, AppError> {
    let job = entities::ocr_jobs::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        status: Set("QUEUED".to_string()),
        r2_key: Set(key.to_string()),
        raw_key: Set(raw_key),
        p_hash: Set(p_hash),
        auto_confirm: Set(auto_confirm),
        wallet_id: Set(wallet_id),
        category_id: Set(category_id),
        schema_version: Set(CURRENT_SCHEMA_VERSION),
        trace_id: Set(trace_id),
        ..Default::default()
    };
    Ok(job.insert(db).await?)
}

pub async fn get_ocr_job(
    db: &DatabaseConnection,
    job_id: &str,
) -> Result<Option<entities::ocr_jobs::Model>, AppError> {
    Ok(entities::ocr_jobs::Entity::find_by_id(job_id.to_string())
        .one(db)
        .await?)
}

pub async fn list_pending_ocr_jobs(
    db: &DatabaseConnection,
) -> Result<Vec<entities::ocr_jobs::Model>, AppError> {
    Ok(entities::ocr_jobs::Entity::find()
        .filter(entities::ocr_jobs::Column::Status.eq("QUEUED"))
        .all(db)
        .await?)
}

pub async fn update_ocr_job(
    db: &DatabaseConnection,
    job_id: &str,
    params: OcrJobUpdateParams,
) -> Result<entities::ocr_jobs::Model, AppError> {
    let mut job: entities::ocr_jobs::ActiveModel = get_ocr_job(db, job_id)
        .await?
        .ok_or_else(|| AppError::not_found("OCR Job not found"))?
        .into();

    job.status = Set(params.status.to_string());
    if let Some(data) = params.processed_data {
        job.processed_data = Set(Some(data.into()));
    }
    job.error = Set(params.error_message);
    job.transaction_id = Set(params.transaction_id);
    job.started_at = Set(params.started_at.map(|dt| dt.naive_utc()));
    job.retry_count = Set(params.retry_count.unwrap_or(0));
    job.last_error = Set(params.last_error);
    job.scheduled_at = Set(params.scheduled_at.map(|dt| dt.naive_utc()));
    if let Some(candidates) = params.resolution_candidates {
        job.resolution_candidates = Set(Some(candidates.into()));
    }

    Ok(job.update(db).await?)
}
