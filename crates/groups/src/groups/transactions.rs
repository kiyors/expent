use db::AppError;
use db::entities;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

#[allow(clippy::missing_errors_doc)]
pub async fn list_group_transactions(
    db: &DatabaseConnection,
    user_id: &str,
    group_id: &str,
) -> Result<Vec<entities::transactions::Model>, AppError> {
    // Verify membership to prevent IDOR
    let is_member =
        entities::user_groups::Entity::find_by_id((user_id.to_string(), group_id.to_string()))
            .one(db)
            .await?;

    if is_member.is_none() {
        return Err(AppError::unauthorized("User is not a member of this group"));
    }

    entities::transactions::Entity::find()
        .filter(entities::transactions::Column::GroupId.eq(group_id))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .order_by_desc(entities::transactions::Column::Date)
        .all(db)
        .await
        .map_err(AppError::from)
}
