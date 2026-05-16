use db::AppError;
use db::entities;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

#[allow(clippy::missing_errors_doc)]
pub async fn list_categories(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::categories::Model>, AppError> {
    // We want System categories first, then User categories.
    // Within those groups, sort alphabetically by name.

    let mut categories = entities::categories::Entity::find()
        .filter(
            sea_orm::Condition::any()
                .add(entities::categories::Column::UserId.eq(user_id))
                .add(entities::categories::Column::UserId.eq("system")),
        )
        .order_by_asc(entities::categories::Column::Name)
        .all(db)
        .await?;

    // Manual sort to ensure "system" is always first
    categories.sort_by(|a, b| {
        let a_is_system = a.user_id == "system";
        let b_is_system = b.user_id == "system";

        match (a_is_system, b_is_system) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Ok(categories)
}
