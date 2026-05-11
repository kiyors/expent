use crate::ops::resolve::{normalize_name, phonetic_encode};
use db::AppError;
use db::entities;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

pub async fn update_contact(
    db: &DatabaseConnection,
    user_id: &str,
    contact_id: &str,
    name: Option<String>,
    phone: Option<String>,
    is_pinned: Option<bool>,
) -> Result<entities::contacts::Model, AppError> {
    let _link =
        entities::contact_links::Entity::find_by_id((user_id.to_string(), contact_id.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("Contact link not found"))?;

    let mut contact: entities::contacts::ActiveModel =
        entities::contacts::Entity::find_by_id(contact_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("Contact not found"))?
            .into();

    if let Some(n) = name {
        let normalized_name = normalize_name(&n);
        let phonetic_name = phonetic_encode(&n);
        contact.name = Set(n);
        contact.normalized_name = Set(Some(normalized_name));
        contact.phonetic_name = Set(Some(phonetic_name));
    }
    if let Some(p) = phone {
        contact.phone = Set(Some(p));
    }
    if let Some(ip) = is_pinned {
        contact.is_pinned = Set(ip);
    }

    contact.update(db).await.map_err(AppError::from)
}
