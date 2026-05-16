use db::AppError;
use db::entities;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[allow(clippy::missing_errors_doc)]
pub async fn create_category(
    db: &DatabaseConnection,
    user_id: &str,
    name: String,
    icon: Option<String>,
    color: Option<String>,
) -> Result<entities::categories::Model, AppError> {
    // 1. Case-insensitive duplicate check
    let normalized_name = name.trim().to_lowercase();

    let existing = entities::categories::Entity::find()
        .filter(
            sea_orm::Condition::any()
                .add(entities::categories::Column::UserId.eq(user_id))
                .add(entities::categories::Column::UserId.eq("system")),
        )
        .all(db)
        .await?;

    if existing
        .iter()
        .any(|c| c.name.to_lowercase() == normalized_name)
    {
        return Err(AppError::Generic(format!(
            "Category '{name}' already exists"
        )));
    }

    let category = entities::categories::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        name: Set(name),
        icon: Set(icon),
        color: Set(color),
    };
    category.insert(db).await.map_err(AppError::from)
}
