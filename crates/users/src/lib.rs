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

    pub async fn update_profile(
        &self,
        user_id: &str,
        name: Option<String>,
        username: Option<String>,
        image: Option<String>,
    ) -> Result<entities::users::Model, AppError> {
        ops::update_profile(&*self.db, user_id, name, username, image).await
    }

    pub async fn list_upi(
        &self,
        user_id: &str,
    ) -> Result<Vec<entities::user_upi_ids::Model>, AppError> {
        ops::list_user_upi(&*self.db, user_id).await
    }

    pub async fn add_upi(
        &self,
        user_id: &str,
        upi_id: String,
        label: Option<String>,
    ) -> Result<entities::user_upi_ids::Model, AppError> {
        ops::add_user_upi(&*self.db, user_id, upi_id, label).await
    }

    pub async fn make_primary_upi(&self, user_id: &str, upi_id: &str) -> Result<(), AppError> {
        ops::make_primary_upi(&*self.db, user_id, upi_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase};

    #[tokio::test]
    async fn test_users_manager_new() {
        let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        let _manager = UsersManager::new(Arc::new(db));
        // We just ensure we can instantiate the manager correctly.
        // It's a simple wrapper struct around a db connection.
        assert!(true); // We just assert it doesn't panic on creation
    }
}
