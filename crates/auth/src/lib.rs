use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use better_auth::plugins::{EmailPasswordPlugin, SessionManagementPlugin};
use better_auth::{AuthBuilder, AuthConfig, AuthRequest, BetterAuth, HttpMethod};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

pub mod adapter;
use crate::adapter::PostgresAdapter;

pub struct AuthSession {
    pub user: better_auth::types_mod::User,
}

impl<S> FromRequestParts<S> for AuthSession
where
    S: Send + Sync,
    Arc<BetterAuth<PostgresAdapter>>: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = Arc::from_ref(state);

        let mut mapped_headers = HashMap::with_capacity(parts.headers.len());
        for (name, value) in parts.headers.iter() {
            if let Ok(val_str) = value.to_str() {
                mapped_headers.insert(name.as_str().to_string(), val_str.to_string());
            }
        }

        let auth_req = AuthRequest::from_parts(
            HttpMethod::Get,
            "/get-session".to_string(),
            mapped_headers,
            None,
            HashMap::new(),
        );

        let response = auth.handle_request(auth_req).await.map_err(|e| {
            tracing::error!("Better-Auth handle_request error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

        if response.status != 200 {
            tracing::debug!(
                "Better-Auth session check failed with status: {}",
                response.status
            );
            return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
        }

        let full_body: serde_json::Value = serde_json::from_slice(&response.body).map_err(|e| {
            tracing::error!(
                "Failed to parse body as Value: {}. Body: {:?}",
                e,
                String::from_utf8_lossy(&response.body)
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid JSON from auth".to_string(),
            )
        })?;

        #[derive(serde::Deserialize)]
        struct SessionResponse {
            user: better_auth::types_mod::User,
        }

        let session_data: SessionResponse = serde_json::from_value(full_body).map_err(|e| {
            tracing::error!("Failed to parse session from Value: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse session: {}", e),
            )
        })?;

        tracing::info!(
            "🔑 Auth Session extracted for user: {}",
            session_data.user.email.as_deref().unwrap_or("unknown")
        );

        Ok(AuthSession {
            user: session_data.user,
        })
    }
}

pub async fn init_auth(
    db: Arc<DatabaseConnection>,
    auth_secret: String,
    base_url: String,
) -> Result<Arc<BetterAuth<PostgresAdapter>>, Box<dyn std::error::Error>> {
    let cors_origin = env::var("CORS_ORIGIN").unwrap_or_default();

    let is_production = env::var("APP_ENV").unwrap_or_default() == "production"
        || env::var("NODE_ENV").unwrap_or_default() == "production";

    let mut trusted_origins = Vec::new();

    if !is_production {
        trusted_origins.extend(vec![
            "http://localhost:3000".to_string(),
            "http://127.0.0.1:3000".to_string(),
            "http://localhost:8080".to_string(),
            "http://127.0.0.1:8080".to_string(),
            "http://localhost:8081".to_string(),
            "http://127.0.0.1:8081".to_string(),
        ]);
    }

    trusted_origins.push(base_url.clone());

    if !cors_origin.is_empty() {
        trusted_origins.extend(cors_origin.split(',').map(|s| s.trim().to_string()));
    }

    trusted_origins.sort();
    trusted_origins.dedup();

    let adapter = PostgresAdapter::new(db);

    let enable_signup = env::var("ENABLE_SIGNUP")
        .map(|v| v != "false")
        .unwrap_or(true);

    let require_email_verification = env::var("REQUIRE_EMAIL_VERIFICATION")
        .map(|v| v != "false")
        .unwrap_or(false);

    let auth_instance = AuthBuilder::new(
        AuthConfig::new(auth_secret)
            .base_url(base_url)
            .trusted_origins(trusted_origins),
    )
    .database(adapter)
    .plugin(
        EmailPasswordPlugin::new()
            .enable_signup(enable_signup)
            .require_email_verification(require_email_verification),
    )
    .plugin(SessionManagementPlugin::new())
    .build()
    .await?;

    Ok(Arc::new(auth_instance))
}
