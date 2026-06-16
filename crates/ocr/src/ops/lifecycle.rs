use db::AppError;
use db::entities;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub const CURRENT_SCHEMA_VERSION: i32 = 1;

/// Maximum number of processing attempts before a job is moved to DEAD_LETTER.
/// Shared by the failure-retry path and the stale-job recovery worker so both
/// honour the same retry budget.
pub const MAX_OCR_RETRIES: i32 = 5;

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
    /// When `Some`, escalates/de-escalates the job's high-res processing mode.
    pub is_high_res: Option<bool>,
}

pub struct OcrJobCreateParams {
    pub user_id: String,
    pub trace_id: Option<String>,
    pub key: String,
    pub raw_key: Option<String>,
    pub p_hash: Option<String>,
    pub auto_confirm: bool,
    pub wallet_id: Option<String>,
    pub category_id: Option<String>,
    pub batch_id: Option<String>,
    pub idempotency_key: Option<String>,
}

pub async fn create_ocr_job(
    db: &DatabaseConnection,
    params: OcrJobCreateParams,
) -> Result<entities::ocr_jobs::Model, AppError> {
    let now = chrono::Utc::now().naive_utc();
    let job = entities::ocr_jobs::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(params.user_id),
        status: Set("QUEUED".to_string()),
        r2_key: Set(params.key),
        raw_key: Set(params.raw_key),
        p_hash: Set(params.p_hash),
        auto_confirm: Set(params.auto_confirm),
        wallet_id: Set(params.wallet_id),
        category_id: Set(params.category_id),
        schema_version: Set(CURRENT_SCHEMA_VERSION),
        trace_id: Set(params.trace_id),
        batch_id: Set(params.batch_id),
        idempotency_key: Set(params.idempotency_key),
        created_at: Set(now),
        updated_at: Set(now),
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

pub async fn get_ocr_job_by_idempotency_key(
    db: &DatabaseConnection,
    user_id: &str,
    idempotency_key: &str,
) -> Result<Option<entities::ocr_jobs::Model>, AppError> {
    Ok(entities::ocr_jobs::Entity::find()
        .filter(entities::ocr_jobs::Column::UserId.eq(user_id.to_string()))
        .filter(entities::ocr_jobs::Column::IdempotencyKey.eq(idempotency_key.to_string()))
        .one(db)
        .await?)
}

pub async fn list_pending_ocr_jobs(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::ocr_jobs::Model>, AppError> {
    Ok(entities::ocr_jobs::Entity::find()
        .filter(entities::ocr_jobs::Column::UserId.eq(user_id.to_string()))
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
    job.updated_at = Set(chrono::Utc::now().naive_utc());
    if let Some(data) = params.processed_data {
        job.processed_data = Set(Some(data));
    }
    job.error = Set(params.error_message);
    job.transaction_id = Set(params.transaction_id);
    job.started_at = Set(params.started_at.map(|dt| dt.naive_utc()));
    // Only touch retry_count when explicitly provided; otherwise preserve the
    // existing value so intermediate updates don't clobber the retry budget.
    if let Some(retry_count) = params.retry_count {
        job.retry_count = Set(retry_count);
    }
    job.last_error = Set(params.last_error);
    job.scheduled_at = Set(params.scheduled_at.map(|dt| dt.naive_utc()));
    if let Some(candidates) = params.resolution_candidates {
        job.resolution_candidates = Set(Some(candidates));
    }
    if let Some(is_high_res) = params.is_high_res {
        job.is_high_res = Set(is_high_res);
    }

    Ok(job.update(db).await?)
}
