use db::AppError;
use db::ContactDetail;
use db::entities;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
};

/// Loads a contact along with its identifiers and related transactions for a given user.
///
/// # Errors
/// Returns `AppError::NotFound` if the user-contact link or the contact itself does not exist,
/// or `AppError::Db` if any of the underlying database queries fail.
pub async fn get_contact_detail(
    db: &DatabaseConnection,
    user_id: &str,
    contact_id: &str,
) -> Result<ContactDetail, AppError> {
    let _link =
        entities::contact_links::Entity::find_by_id((user_id.to_string(), contact_id.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("Contact link not found"))?;

    let contact = entities::contacts::Entity::find_by_id(contact_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Contact not found"))?;

    let identifiers = entities::contact_identifiers::Entity::find()
        .filter(entities::contact_identifiers::Column::ContactId.eq(contact_id))
        .all(db)
        .await?;

    let transactions = entities::transactions::Entity::find()
        .join(
            JoinType::InnerJoin,
            entities::transactions::Relation::TxnParties.def(),
        )
        .filter(entities::txn_parties::Column::ContactId.eq(contact_id))
        .order_by_desc(entities::transactions::Column::Date)
        .all(db)
        .await?;

    Ok(ContactDetail {
        contact,
        identifiers,
        transactions,
    })
}
