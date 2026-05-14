use db::AppError;
use db::entities;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub mod ops;

#[derive(Debug, Clone)]
pub struct CategoriesManager {
    db: Arc<DatabaseConnection>,
}

impl CategoriesManager {
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: &str,
        name: String,
        icon: Option<String>,
        color: Option<String>,
    ) -> Result<entities::categories::Model, AppError> {
        ops::create_category(&*self.db, user_id, name, icon, color).await
    }

    pub async fn list(&self, user_id: &str) -> Result<Vec<entities::categories::Model>, AppError> {
        ops::list_categories(&*self.db, user_id).await
    }

    pub async fn delete(&self, user_id: &str, category_id: &str) -> Result<(), AppError> {
        ops::delete_category(&*self.db, user_id, category_id).await
    }

    pub async fn ensure_system_categories(&self) -> Result<(), AppError> {
        ops::ensure_system_categories(&*self.db).await
    }
}
