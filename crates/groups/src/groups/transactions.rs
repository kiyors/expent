use db::AppError;
use db::entities;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

#[allow(clippy::missing_errors_doc)]
pub async fn list_group_transactions(
    db: &DatabaseConnection,
    user_id: &str,
    group_id: &str,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<entities::transactions::Model>, AppError> {
    // Verify membership to prevent IDOR
    let is_member =
        entities::user_groups::Entity::find_by_id((user_id.to_string(), group_id.to_string()))
            .one(db)
            .await?;

    if is_member.is_none() {
        return Err(AppError::unauthorized("User is not a member of this group"));
    }

    let mut query = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::GroupId.eq(group_id))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .order_by_desc(entities::transactions::Column::Date);

    if let Some(l) = limit {
        query = query.limit(l);
    }
    if let Some(o) = offset {
        query = query.offset(o);
    }

    query.all(db).await.map_err(AppError::from)
}
