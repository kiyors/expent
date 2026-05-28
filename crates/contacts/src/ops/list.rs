use db::AppError;
use db::entities;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
};

/// Lists contacts linked to the given user, optionally paginated.
///
/// When `limit` is `None` no `LIMIT` is applied (matches the existing
/// transactions list semantics). Results are ordered by `name` so paginated
/// callers see a stable cursor.
///
/// # Errors
/// Returns `AppError::Db` if the underlying query fails.
pub async fn list_contacts(
    db: &DatabaseConnection,
    user_id: &str,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<entities::contacts::Model>, AppError> {
    let mut query = entities::contacts::Entity::find()
        .join(
            JoinType::InnerJoin,
            entities::contacts::Relation::ContactLinks.def(),
        )
        .filter(entities::contact_links::Column::UserId.eq(user_id))
        .order_by_asc(entities::contacts::Column::Name);

    if let Some(l) = limit {
        query = query.limit(l);
    }
    if let Some(o) = offset {
        query = query.offset(o);
    }

    query.all(db).await.map_err(AppError::from)
}
