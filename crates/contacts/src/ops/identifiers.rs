use db::AppError;
use db::entities;
use db::entities::enums::IdentifierType;
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set};

/// Adds a new identifier (UPI, email, phone, etc.) to a contact belonging to the given user.
///
/// # Errors
/// Returns `AppError::NotFound` if the user does not have access to the contact, or
/// `AppError::Db` if the identifier insert fails.
pub async fn add_contact_identifier<C>(
    db: &C,
    user_id: &str,
    contact_id: &str,
    r#type: IdentifierType,
    value: String,
) -> Result<entities::contact_identifiers::Model, AppError>
where
    C: ConnectionTrait,
{
    let _link =
        entities::contact_links::Entity::find_by_id((user_id.to_string(), contact_id.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("Contact link not found"))?;

    let identifier = entities::contact_identifiers::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        contact_id: Set(contact_id.to_string()),
        r#type: Set(r#type),
        value: Set(value),
        linked_user_id: Set(None),
    };

    identifier.insert(db).await.map_err(AppError::from)
}
