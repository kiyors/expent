use db::AppError;
use db::entities;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

#[allow(clippy::missing_errors_doc)]
pub async fn get_group(
    db: &DatabaseConnection,
    user_id: &str,
    group_id: &str,
) -> Result<entities::groups::Model, AppError> {
    entities::groups::Entity::find_by_id(group_id.to_string())
        .inner_join(entities::user_groups::Entity)
        .filter(entities::user_groups::Column::UserId.eq(user_id))
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Group not found"))
}
