use chrono::Utc;
use db::AppError;
use db::entities;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[allow(clippy::missing_errors_doc)]
pub async fn update_profile(
    db: &DatabaseConnection,
    user_id: &str,
    name: Option<String>,
    username: Option<String>,
    image: Option<String>,
) -> Result<entities::users::Model, AppError> {
    let mut user: entities::users::ActiveModel =
        entities::users::Entity::find_by_id(user_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?
            .into();

    if let Some(n) = name {
        user.name = Set(n);
    }
    if let Some(u) = username {
        let normalized = u.trim().to_lowercase();

        // 1. Uniqueness check for username
        let existing = entities::users::Entity::find()
            .filter(entities::users::Column::Username.eq(normalized.clone()))
            .filter(entities::users::Column::Id.ne(user_id))
            .one(db)
            .await?;

        if existing.is_some() {
            return Err(AppError::Generic(format!(
                "Username '{normalized}' is already taken"
            )));
        }

        user.username = Set(Some(normalized));
    }
    if let Some(i) = image {
        user.image = Set(Some(i));
    }
    user.updated_at = Set(Utc::now().into());

    user.update(db).await.map_err(AppError::from)
}
