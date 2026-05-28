use db::AppError;
use db::entities;
use sea_orm::{DatabaseConnection, EntityTrait};

/// Removes the link between a user and a contact, effectively deleting the contact for the user.
///
/// # Errors
/// Returns `AppError::Db` if the underlying delete query fails.
pub async fn delete_contact(
    db: &DatabaseConnection,
    user_id: &str,
    contact_id: &str,
) -> Result<(), AppError> {
    entities::contact_links::Entity::delete_by_id((user_id.to_string(), contact_id.to_string()))
        .exec(db)
        .await?;
    Ok(())
}
