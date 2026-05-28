use db::AppError;
use db::entities;
use sea_orm::prelude::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use std::collections::HashSet;

/// Merges the secondary contact into the primary contact (transactions, identifiers, phone)
/// and deletes the secondary contact. The operation runs inside a database transaction.
///
/// # Errors
/// Returns `AppError::Validation` if `primary_id == secondary_id`,
/// `AppError::NotFound` if either contact link does not belong to the user or a contact
/// row is missing, or `AppError::Db` if any database operation in the transaction fails.
pub async fn merge_contacts(
    db: &DatabaseConnection,
    user_id: &str,
    primary_id: &str,
    secondary_id: &str,
) -> Result<entities::contacts::Model, AppError> {
    if primary_id == secondary_id {
        return Err(AppError::validation("Cannot merge a contact into itself"));
    }

    let user_id_owned = user_id.to_string();
    let primary_id_owned = primary_id.to_string();
    let secondary_id_owned = secondary_id.to_string();

    // Verify both contacts belong to the user in a single query
    let links = entities::contact_links::Entity::find()
        .filter(entities::contact_links::Column::UserId.eq(user_id))
        .filter(entities::contact_links::Column::ContactId.is_in(vec![primary_id, secondary_id]))
        .all(db)
        .await?;

    if links.len() != 2 {
        return Err(AppError::not_found(
            "One or both contact links not found or access denied",
        ));
    }

    // Transaction for safety
    let txn = db.begin().await?;

    // 1. Update txn_parties
    entities::txn_parties::Entity::update_many()
        .col_expr(
            entities::txn_parties::Column::ContactId,
            Expr::value(&primary_id_owned),
        )
        .filter(entities::txn_parties::Column::ContactId.eq(secondary_id))
        .exec(&txn)
        .await?;

    // 2. Update contact_identifiers
    // Get existing primary and secondary identifiers in a single query
    let all_identifiers = entities::contact_identifiers::Entity::find()
        .filter(
            entities::contact_identifiers::Column::ContactId.is_in(vec![primary_id, secondary_id]),
        )
        .all(&txn)
        .await?;

    let mut primary_identifier_set = HashSet::with_capacity(all_identifiers.len());
    let mut secondary_identifiers = Vec::with_capacity(all_identifiers.len());

    for ident in all_identifiers {
        if ident.contact_id == primary_id {
            primary_identifier_set.insert((ident.r#type, ident.value));
        } else if ident.contact_id == secondary_id {
            secondary_identifiers.push(ident);
        }
    }

    let mut to_delete = Vec::new();
    let mut to_move = Vec::new();

    for sec_id in secondary_identifiers {
        if primary_identifier_set.contains(&(sec_id.r#type, sec_id.value)) {
            to_delete.push(sec_id.id);
        } else {
            to_move.push(sec_id.id);
        }
    }

    // Batch delete duplicates
    if !to_delete.is_empty() {
        entities::contact_identifiers::Entity::delete_many()
            .filter(entities::contact_identifiers::Column::Id.is_in(to_delete))
            .exec(&txn)
            .await?;
    }

    // Batch move unique identifiers
    if !to_move.is_empty() {
        entities::contact_identifiers::Entity::update_many()
            .filter(entities::contact_identifiers::Column::Id.is_in(to_move))
            .col_expr(
                entities::contact_identifiers::Column::ContactId,
                Expr::value(&primary_id_owned),
            )
            .exec(&txn)
            .await?;
    }

    // 3. Move the phone number if primary doesn't have one and secondary does
    let mut primary_contact: entities::contacts::ActiveModel =
        entities::contacts::Entity::find_by_id(&primary_id_owned)
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::not_found("Primary contact for merge not found"))?
            .into();

    let secondary_contact = entities::contacts::Entity::find_by_id(&secondary_id_owned)
        .one(&txn)
        .await?
        .ok_or_else(|| AppError::not_found("Secondary contact for merge not found"))?;

    let mut updated_primary = false;
    if primary_contact.phone.as_ref().is_none() && secondary_contact.phone.is_some() {
        primary_contact.phone = Set(secondary_contact.phone);
        updated_primary = true;
    }

    let final_primary = if updated_primary {
        primary_contact.update(&txn).await?
    } else {
        entities::contacts::Entity::find_by_id(primary_id_owned)
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::not_found("Primary contact not found after update attempt"))?
    };

    // 4. Delete secondary contact_links
    entities::contact_links::Entity::delete_by_id((user_id_owned, secondary_id_owned.clone()))
        .exec(&txn)
        .await?;

    // 5. Delete secondary contact
    entities::contacts::Entity::delete_by_id(secondary_id_owned)
        .exec(&txn)
        .await?;

    txn.commit().await?;

    Ok(final_primary)
}
