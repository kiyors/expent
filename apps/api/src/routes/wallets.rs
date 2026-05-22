use axum::Router;
use axum::extract::{Json, Path, State};
use axum::routing::{get, put};
use db::dto::{CreateWalletRequest, UpdateWalletRequest};

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_wallets_handler).post(create_wallet_handler))
        .route(
            "/{id}",
            put(update_wallet_handler).delete(delete_wallet_handler),
        )
}

pub async fn list_wallets_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::wallets::Model>>, ApiError> {
    let result = state.core.wallets.list(&session.user.id).await?;
    Ok(Json(result))
}

pub async fn create_wallet_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<CreateWalletRequest>,
) -> Result<Json<db::entities::wallets::Model>, ApiError> {
    let result = state
        .core
        .wallets
        .create(
            &session.user.id,
            &payload.name,
            payload.r#type,
            payload.initial_balance,
        )
        .await?;
    Ok(Json(result))
}

pub async fn update_wallet_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
    Json(payload): Json<UpdateWalletRequest>,
) -> Result<Json<db::entities::wallets::Model>, ApiError> {
    let result = state
        .core
        .wallets
        .update(&session.user.id, &id, payload.name, payload.balance)
        .await?;
    Ok(Json(result))
}

pub async fn delete_wallet_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<Json<u64>, ApiError> {
    let result = state.core.wallets.delete(&session.user.id, &id).await?;
    Ok(Json(result))
}
