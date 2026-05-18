use axum::{
    Router,
    extract::FromRef,
    http::{HeaderValue, Method},
    routing::get,
};
pub use expent_core::auth::AuthSession;
use expent_core::better_auth::AxumIntegration;
use expent_core::{Core, CoreConfig, sea_orm::DatabaseConnection};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod extractors;
pub mod middleware;
pub mod routes;

use crate::middleware::rate_limit::UserRateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub core: Core,
    pub ocr_limiter: UserRateLimiter,
}

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

impl FromRef<AppState> for Arc<DatabaseConnection> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.core.db)
    }
}

impl FromRef<AppState>
    for Arc<expent_core::better_auth::BetterAuth<expent_core::auth::adapter::PostgresAdapter>>
{
    fn from_ref(state: &AppState) -> Self {
        state.core.auth.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();

    let rust_log =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info,api=debug,better_auth=info".into());

    let filter_string = if rust_log.contains("sqlx=") {
        rust_log
    } else {
        format!("{},sqlx=error,sea_orm=warn,tower_http=debug", rust_log)
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter_string))
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false),
        )
        .init();

    let (ocr_tx, _) = tokio::sync::broadcast::channel(100);

    let core_config = CoreConfig {
        database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        s3_endpoint: std::env::var("S3_ENDPOINT").expect("S3_ENDPOINT must be set"),
        s3_access_key_id: std::env::var("S3_ACCESS_KEY_ID").expect("S3_ACCESS_KEY_ID must be set"),
        s3_secret_access_key: std::env::var("S3_SECRET_ACCESS_KEY")
            .expect("S3_SECRET_ACCESS_KEY must be set"),
        s3_bucket_name: std::env::var("S3_BUCKET_NAME").expect("S3_BUCKET_NAME must be set"),
        ocr_worker_url: std::env::var("OCR_WORKER_URL").ok(),
        better_auth_secret: std::env::var("BETTER_AUTH_SECRET")
            .or_else(|_| std::env::var("BETTERAUTH_SECRET"))
            .expect("BETTER_AUTH_SECRET must be set"),
        better_auth_base_url: std::env::var("BETTER_AUTH_BASE_URL")
            .or_else(|_| std::env::var("BASE_URL"))
            .unwrap_or_else(|_| "http://localhost:7878".into()),
    };

    let core = Core::init(core_config, ocr_tx).await?;

    // Start background workers from OCR manager
    core.ocr_manager.spawn_workers(Arc::new(core.clone()));

    let state = AppState {
        core: core.clone(),
        ocr_limiter: UserRateLimiter::new(10, 20),
    };

    let auth_router = core.auth.clone().axum_router();

    // Rate limiting config: ~1 request per second per IP, burst of 10
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(10)
            .finish()
            .ok_or("Failed to build governor configuration")?,
    );

    let api_router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest("/transactions", routes::transactions::router())
        .nest("/budgets", routes::budgets::router())
        .nest("/p2p", routes::p2p::router())
        .nest("/groups", routes::groups::router())
        .nest("/contacts", routes::contacts::router())
        .nest("/wallets", routes::wallets::router())
        .nest("/users", routes::users::router())
        .nest("/categories", routes::categories::router())
        .nest("/subscriptions", routes::subscriptions::router())
        .nest("/reconciliation", routes::reconciliation::router())
        .nest("/upload", routes::uploads::router())
        .nest("/ocr", routes::ocr::router())
        .layer(GovernorLayer::new(governor_conf));

    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://127.0.0.1:3000".to_string())
        .split(',')
        .map(|s| s.parse::<HeaderValue>())
        .collect::<Result<Vec<_>, _>>()?;

    let app = Router::new()
        .nest("/api/auth", auth_router.with_state(core.auth.clone()))
        .nest("/api", api_router)
        .layer(TraceLayer::new_for_http())
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origins)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::PATCH,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                ])
                .allow_credentials(true),
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 7878));
    tracing::info!("🚀 API starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
