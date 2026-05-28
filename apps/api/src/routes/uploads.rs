use axum::extract::{Multipart, State};
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/presigned", post(get_presigned_url_handler))
        .route("/", post(direct_upload_handler))
}

#[derive(Deserialize)]
pub struct PresignedUrlRequest {
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
}

#[derive(Serialize)]
pub struct PresignedUrlResponse {
    pub url: String,
    pub key: String,
    pub raw_key: Option<String>,
}

pub async fn get_presigned_url_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<PresignedUrlRequest>,
) -> Result<Json<PresignedUrlResponse>, ApiError> {
    let (url, key) = state
        .core
        .upload_client
        .get_presigned_url(
            &session.user.id,
            &payload.file_name,
            &payload.content_type,
            Duration::from_secs(3600),
        )
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get presigned URL: {:?}", e)))?;

    Ok(Json(PresignedUrlResponse {
        url,
        key,
        raw_key: None,
    }))
}

pub async fn direct_upload_handler(
    State(state): State<AppState>,
    session: AuthSession,
    mut multipart: Multipart,
) -> Result<Json<PresignedUrlResponse>, ApiError> {
    // Per-user rate limiting — proper 429 with Retry-After (not a 400) so
    // clients can back off intelligently and distinguish quota from input errors.
    if !state.ocr_limiter.check(&session.user.id) {
        return Err(ApiError::RateLimited(
            "Rate limit exceeded for upload requests. Please wait a moment.".to_string(),
        ));
    }

    tracing::debug!("📁 Received upload request for user: {}", session.user.id);
    let mut file_data = None;
    let mut file_name = String::new();
    let mut content_type = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Multipart next_field error: {:?}", e)))?
    {
        let name = field.name().unwrap_or_default().to_string();
        tracing::debug!("📦 Processing multipart field: {}", name);
        if name == "file" {
            file_name = field.file_name().unwrap_or("unnamed").to_string();
            content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            tracing::debug!(
                "📎 Extracting bytes for file: {} ({})",
                file_name,
                content_type
            );
            file_data = Some(field.bytes().await.map_err(|e| {
                ApiError::Internal(format!("Multipart bytes extraction error: {:?}", e))
            })?);
            break;
        }
    }

    let data = file_data.ok_or_else(|| {
        tracing::warn!("⚠️ No file data found in multipart request");
        ApiError::BadRequest("No file uploaded".to_string())
    })?;

    tracing::info!(
        "🚀 Starting direct upload for file: {} ({} bytes)",
        file_name,
        data.len()
    );

    // Use the new optimized upload_direct which handles resizing, pHash, and JPEG conversion
    let processed = state
        .core
        .upload_client
        .upload_direct(
            &session.user.id,
            data,
            Some(file_name),
            Some(content_type),
            true, // Enable optimization
        )
        .await
        .map_err(|e| ApiError::Internal(format!("UploadClient upload failed: {:?}", e)))?;

    tracing::info!(
        "✅ Upload successful, key: {}, pHash: {:?}",
        processed.key,
        processed.p_hash
    );
    let r2_public_url = std::env::var("R2_PUBLIC_URL")
        .unwrap_or_else(|_| "https://pub-3e637dff099d43faa282edc2702dbf2c.r2.dev".to_string());

    Ok(Json(PresignedUrlResponse {
        url: format!("{}/{}", r2_public_url, processed.key),
        key: processed.key,
        raw_key: processed.raw_key,
    }))
}
