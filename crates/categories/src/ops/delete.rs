use db::AppError;
use db::entities;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

#[allow(clippy::missing_errors_doc)]
pub async fn delete_category(
    db: &DatabaseConnection,
    user_id: &str,
    category_id: &str,
) -> Result<(), AppError> {
    // 1. Fetch category to check ownership and system status
    let category = entities::categories::Entity::find_by_id(category_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Category not found"))?;

    if category.user_id == "system" {
        return Err(AppError::unauthorized(
            "System categories cannot be deleted",
        ));
    }

    if category.user_id != user_id {
        return Err(AppError::unauthorized(
            "You don't have permission to delete this category",
        ));
    }

    // 2. Perform deletion
    entities::categories::Entity::delete_many()
        .filter(entities::categories::Column::Id.eq(category_id))
        .exec(db)
        .await?;

    Ok(())
}
