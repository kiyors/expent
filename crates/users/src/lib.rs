use db::AppError;
use db::entities;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub mod ops;

#[derive(Debug, Clone)]
pub struct UsersManager {
    db: Arc<DatabaseConnection>,
}

impl UsersManager {
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn update_profile(
        &self,
        user_id: &str,
        name: Option<String>,
        username: Option<String>,
        image: Option<String>,
    ) -> Result<entities::users::Model, AppError> {
        ops::update_profile(&self.db, user_id, name, username, image).await
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn list_upi(
        &self,
        user_id: &str,
    ) -> Result<Vec<entities::user_upi_ids::Model>, AppError> {
        ops::list_user_upi(&self.db, user_id).await
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn add_upi(
        &self,
        user_id: &str,
        upi_id: String,
        label: Option<String>,
    ) -> Result<entities::user_upi_ids::Model, AppError> {
        ops::add_user_upi(&self.db, user_id, upi_id, label).await
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn make_primary_upi(&self, user_id: &str, upi_id: &str) -> Result<(), AppError> {
        ops::make_primary_upi(&self.db, user_id, upi_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase};

    #[tokio::test]
    async fn test_users_manager_new() {
        let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        let manager = UsersManager::new(Arc::new(db));
        // Verify the manager retains the shared DB handle (refcount > 0) and is
        // cheaply cloneable, which is the contract callers rely on.
        let cloned = manager.clone();
        assert!(Arc::strong_count(&manager.db) >= 2);
        drop(cloned);
        assert_eq!(Arc::strong_count(&manager.db), 1);
    }
}
