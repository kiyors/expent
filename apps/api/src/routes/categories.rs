use axum::Router;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use db::dto::CreateCategoryRequest;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_categories_handler).post(create_category_handler),
        )
        .route("/{id}", delete(delete_category_handler))
}

pub async fn list_categories_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::categories::Model>>, ApiError> {
    let result = state.core.categories.list(&session.user.id).await?;
    Ok(Json(result))
}

pub async fn create_category_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<Json<db::entities::categories::Model>, ApiError> {
    let result = state
        .core
        .categories
        .create(&session.user.id, payload.name, payload.icon, payload.color)
        .await?;
    Ok(Json(result))
}

pub async fn delete_category_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.core.categories.delete(&session.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
