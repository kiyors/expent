use db::AppError;
use db::ContactDetail;
use db::entities;
use db::entities::enums::IdentifierType;
use sea_orm::{ConnectionTrait, DatabaseConnection};
use std::sync::Arc;

pub mod ops;

#[derive(Debug, Clone)]
pub struct ContactsManager {
    db: Arc<DatabaseConnection>,
}

impl ContactsManager {
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Creates a new contact for the given user.
    ///
    /// # Errors
    /// Propagates [`AppError`] from [`ops::create_contact`]; typically `AppError::Db`
    /// if the insert or user-link fails.
    pub async fn create(
        &self,
        user_id: &str,
        name: &str,
        phone: Option<String>,
    ) -> Result<entities::contacts::Model, AppError> {
        ops::create_contact(&*self.db, user_id, name.to_string(), phone).await
    }

    /// Lists contacts linked to the given user, optionally paginated. See
    /// [`ops::list_contacts`] for the default limit.
    ///
    /// # Errors
    /// Propagates [`AppError::Db`] from [`ops::list_contacts`] if the query fails.
    pub async fn list(
        &self,
        user_id: &str,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<entities::contacts::Model>, AppError> {
        ops::list_contacts(&self.db, user_id, limit, offset).await
    }

    /// Removes the link between the user and the contact.
    ///
    /// # Errors
    /// Propagates [`AppError::Db`] from [`ops::delete_contact`] if the delete fails.
    pub async fn delete(&self, user_id: &str, contact_id: &str) -> Result<(), AppError> {
        ops::delete_contact(&self.db, user_id, contact_id).await
    }

    /// Updates mutable fields on a contact owned by the user.
    ///
    /// # Errors
    /// Propagates [`AppError`] from [`ops::update_contact`]: `AppError::NotFound`
    /// if the user-contact link or contact is missing, or `AppError::Db` on query failure.
    pub async fn update(
        &self,
        user_id: &str,
        contact_id: &str,
        name: Option<String>,
        phone: Option<String>,
        is_pinned: Option<bool>,
    ) -> Result<entities::contacts::Model, AppError> {
        ops::update_contact(&self.db, user_id, contact_id, name, phone, is_pinned).await
    }

    /// Loads a contact along with its identifiers and related transactions.
    ///
    /// # Errors
    /// Propagates [`AppError`] from [`ops::get_contact_detail`]: `AppError::NotFound`
    /// when the contact link or contact does not exist, or `AppError::Db` on query failure.
    pub async fn get_detail(
        &self,
        user_id: &str,
        contact_id: &str,
    ) -> Result<ContactDetail, AppError> {
        ops::get_contact_detail(&self.db, user_id, contact_id).await
    }

    /// Adds an identifier (UPI, email, phone, etc.) to a contact owned by the user.
    ///
    /// # Errors
    /// Propagates [`AppError`] from [`ops::add_contact_identifier`]: `AppError::NotFound`
    /// if the user does not own the contact, or `AppError::Db` if the insert fails.
    pub async fn add_identifier(
        &self,
        user_id: &str,
        contact_id: &str,
        r#type: IdentifierType,
        value: String,
    ) -> Result<entities::contact_identifiers::Model, AppError> {
        ops::add_contact_identifier(&*self.db, user_id, contact_id, r#type, value).await
    }

    /// Merges `source_id` into `target_id`, transferring identifiers and transactions.
    ///
    /// # Errors
    /// Propagates [`AppError`] from [`ops::merge_contacts`]: `AppError::Validation` when the
    /// two ids are equal, `AppError::NotFound` when a contact link or row is missing, or
    /// `AppError::Db` if any step of the merge transaction fails.
    pub async fn merge(
        &self,
        user_id: &str,
        source_id: &str,
        target_id: &str,
    ) -> Result<entities::contacts::Model, AppError> {
        ops::merge_contacts(&self.db, user_id, source_id, target_id).await
    }

    /// Returns pairs of contacts that look like duplicates for the user.
    ///
    /// # Errors
    /// Propagates [`AppError::Db`] from [`ops::get_merge_suggestions`] on query failure.
    pub async fn get_merge_suggestions(
        &self,
        user_id: &str,
    ) -> Result<Vec<ops::MergeSuggestion>, AppError> {
        ops::get_merge_suggestions(&self.db, user_id).await
    }

    /// Resolves a single set of identifiers/name to an existing contact.
    ///
    /// # Errors
    /// Propagates [`AppError::Db`] from [`ops::resolve_contact`] if any lookup query fails.
    pub async fn resolve<C>(
        &self,
        conn: &C,
        user_id: &str,
        params: ops::ResolveParams,
    ) -> Result<ops::ContactResolution, AppError>
    where
        C: ConnectionTrait,
    {
        ops::resolve_contact(conn, user_id, params).await
    }

    /// Resolves a batch of [`ops::ResolveParams`] in one optimized pass.
    ///
    /// # Errors
    /// Propagates [`AppError::Db`] from [`ops::resolve_contacts_bulk`] if any of the
    /// shared bulk lookups fail.
    pub async fn resolve_bulk<C>(
        &self,
        conn: &C,
        user_id: &str,
        batch: Vec<ops::ResolveParams>,
    ) -> Result<Vec<ops::ContactResolution>, AppError>
    where
        C: ConnectionTrait,
    {
        ops::resolve_contacts_bulk(conn, user_id, batch).await
    }
}
