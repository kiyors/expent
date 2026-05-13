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

    pub async fn create(
        &self,
        user_id: &str,
        name: &str,
        phone: Option<String>,
    ) -> Result<entities::contacts::Model, AppError> {
        ops::create_contact(&*self.db, user_id, name.to_string(), phone).await
    }

    pub async fn list(&self, user_id: &str) -> Result<Vec<entities::contacts::Model>, AppError> {
        ops::list_contacts(&*self.db, user_id).await
    }

    pub async fn delete(&self, user_id: &str, contact_id: &str) -> Result<(), AppError> {
        ops::delete_contact(&*self.db, user_id, contact_id).await
    }

    pub async fn update(
        &self,
        user_id: &str,
        contact_id: &str,
        name: Option<String>,
        phone: Option<String>,
        is_pinned: Option<bool>,
    ) -> Result<entities::contacts::Model, AppError> {
        ops::update_contact(&*self.db, user_id, contact_id, name, phone, is_pinned).await
    }

    pub async fn get_detail(
        &self,
        user_id: &str,
        contact_id: &str,
    ) -> Result<ContactDetail, AppError> {
        ops::get_contact_detail(&*self.db, user_id, contact_id).await
    }

    pub async fn add_identifier(
        &self,
        user_id: &str,
        contact_id: &str,
        r#type: IdentifierType,
        value: String,
    ) -> Result<entities::contact_identifiers::Model, AppError> {
        ops::add_contact_identifier(&*self.db, user_id, contact_id, r#type, value).await
    }

    pub async fn merge(
        &self,
        user_id: &str,
        source_id: &str,
        target_id: &str,
    ) -> Result<entities::contacts::Model, AppError> {
        ops::merge_contacts(&*self.db, user_id, source_id, target_id).await
    }

    pub async fn get_merge_suggestions(
        &self,
        user_id: &str,
    ) -> Result<Vec<ops::MergeSuggestion>, AppError> {
        ops::get_merge_suggestions(&*self.db, user_id).await
    }

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
}
