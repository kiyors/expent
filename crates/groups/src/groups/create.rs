use chrono::Utc;
use db::AppError;
use db::entities;
use db::entities::enums::GroupRole;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};

#[allow(clippy::missing_errors_doc)]
pub async fn create_group(
    db: &DatabaseConnection,
    user_id: &str,
    name: &str,
    description: Option<String>,
) -> Result<entities::groups::Model, AppError> {
    let group = entities::groups::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        name: Set(name.to_string()),
        description: Set(description),
        created_at: Set(Utc::now().into()),
    };
    let result = group.insert(db).await?;

    let user_group = entities::user_groups::ActiveModel {
        user_id: Set(user_id.to_string()),
        group_id: Set(result.id.clone()),
        role: Set(GroupRole::Admin),
    };
    user_group.insert(db).await?;

    Ok(result)
}
