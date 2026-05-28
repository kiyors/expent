use db::AppError;
use db::entities;
use moka::future::Cache;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Duration;

pub mod ops;

#[derive(Clone)]
pub struct CategoriesManager {
    db: Arc<DatabaseConnection>,
    list_cache: Cache<String, Vec<entities::categories::Model>>,
}

impl std::fmt::Debug for CategoriesManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CategoriesManager")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl CategoriesManager {
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        let list_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_idle(Duration::from_mins(10)) // 10 minutes
            .build();

        Self { db, list_cache }
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn create(
        &self,
        user_id: &str,
        name: String,
        icon: Option<String>,
        color: Option<String>,
    ) -> Result<entities::categories::Model, AppError> {
        let result = ops::create_category(&self.db, user_id, name, icon, color).await?;
        self.list_cache.invalidate(user_id).await;
        Ok(result)
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn list(&self, user_id: &str) -> Result<Vec<entities::categories::Model>, AppError> {
        if let Some(cached) = self.list_cache.get(user_id).await {
            return Ok(cached);
        }

        let categories = ops::list_categories(&self.db, user_id).await?;
        self.list_cache
            .insert(user_id.to_string(), categories.clone())
            .await;
        Ok(categories)
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn delete(&self, user_id: &str, category_id: &str) -> Result<(), AppError> {
        ops::delete_category(&self.db, user_id, category_id).await?;
        self.list_cache.invalidate(user_id).await;
        Ok(())
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn ensure_system_categories(&self) -> Result<(), AppError> {
        ops::ensure_system_categories(&self.db).await
    }
}
