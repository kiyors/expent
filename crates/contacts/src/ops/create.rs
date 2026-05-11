use crate::ops::resolve::{normalize_name, phonetic_encode};
use db::AppError;
use db::entities;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};

pub async fn create_contact(
    db: &DatabaseConnection,
    user_id: &str,
    name: String,
    phone: Option<String>,
) -> Result<entities::contacts::Model, AppError> {
    let normalized_name = normalize_name(&name);
    let phonetic_name = phonetic_encode(&name);

    let contact = entities::contacts::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        name: Set(name),
        phone: Set(phone),
        is_pinned: Set(false),
        normalized_name: Set(Some(normalized_name)),
        phonetic_name: Set(Some(phonetic_name)),
    };
    let result = contact.insert(db).await?;

    let link = entities::contact_links::ActiveModel {
        user_id: Set(user_id.to_string()),
        contact_id: Set(result.id.clone()),
    };
    link.insert(db).await?;

    Ok(result)
}
