use axum::Router;
use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post};
use db::dto::PaginationParams;
use serde::Deserialize;
use validator::Validate;

use crate::extractors::ValidatedJson;
use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_groups_handler))
        .route("/create", post(create_group_handler))
        .route("/invite", post(invite_to_group_handler))
        .route("/{id}/transactions", get(list_group_transactions_handler))
        .route("/{id}/members", get(list_group_members_handler))
        .route(
            "/{group_id}/members/{user_id}",
            delete(remove_group_member_handler),
        )
        .route(
            "/{group_id}/members/{user_id}/role",
            patch(update_member_role_handler),
        )
}

pub async fn list_groups_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::groups::Model>>, ApiError> {
    let result = state.core.groups.list_groups(&session.user.id).await?;
    Ok(Json(result))
}

#[derive(Deserialize, Validate)]
pub struct CreateGroupRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

pub async fn create_group_handler(
    State(state): State<AppState>,
    session: AuthSession,
    ValidatedJson(payload): ValidatedJson<CreateGroupRequest>,
) -> Result<Json<db::entities::groups::Model>, ApiError> {
    let result = state
        .core
        .groups
        .create_group(&session.user.id, &payload.name, payload.description)
        .await?;

    Ok(Json(result))
}

#[derive(Deserialize, Validate)]
pub struct InviteToGroupRequest {
    #[validate(email)]
    pub receiver_email: String,
    pub group_id: String,
}

pub async fn invite_to_group_handler(
    State(state): State<AppState>,
    session: AuthSession,
    ValidatedJson(payload): ValidatedJson<InviteToGroupRequest>,
) -> Result<Json<db::entities::p2p_requests::Model>, ApiError> {
    let result = state
        .core
        .groups
        .invite_to_group(&session.user.id, &payload.receiver_email, &payload.group_id)
        .await?;

    Ok(Json(result))
}

pub async fn list_group_transactions_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<db::entities::transactions::Model>>, ApiError> {
    let result = state
        .core
        .groups
        .list_group_transactions(&session.user.id, &id, params.limit, params.offset)
        .await?;

    Ok(Json(result))
}

pub async fn list_group_members_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<Json<Vec<db::GroupMemberDetail>>, ApiError> {
    let result = state
        .core
        .groups
        .list_group_members(&session.user.id, &id)
        .await?;
    Ok(Json(result))
}

pub async fn remove_group_member_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path((group_id, user_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    state
        .core
        .groups
        .remove_group_member(&session.user.id, &group_id, &user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: db::entities::enums::GroupRole,
}

pub async fn update_member_role_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path((group_id, user_id)): Path<(String, String)>,
    Json(payload): Json<UpdateMemberRoleRequest>,
) -> Result<StatusCode, ApiError> {
    state
        .core
        .groups
        .update_member_role(&session.user.id, &group_id, &user_id, payload.role)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
