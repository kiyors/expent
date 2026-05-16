use db::AppError;
use db::entities;
use db::entities::enums::WalletType;
use moka::future::Cache;
use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, DatabaseConnection};
use std::sync::Arc;
use std::time::Duration;

pub mod ops;

#[derive(Clone)]
pub struct WalletsManager {
    db: Arc<DatabaseConnection>,
    resolve_cache: Cache<(String, ops::ResolveWalletParams), entities::wallets::Model>,
}

impl std::fmt::Debug for WalletsManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletsManager")
            .field("db", &self.db)
            .finish()
    }
}

impl WalletsManager {
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        let resolve_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_idle(Duration::from_secs(300)) // 5 minutes
            .build();

        Self { db, resolve_cache }
    }

    pub async fn create(
        &self,
        user_id: &str,
        name: &str,
        wallet_type: WalletType,
        initial_balance: Decimal,
    ) -> Result<entities::wallets::Model, AppError> {
        let result =
            ops::create_wallet(&*self.db, user_id, name, wallet_type, initial_balance).await?;
        self.resolve_cache.invalidate_all(); // Simple invalidation for now
        Ok(result)
    }

    pub async fn list(&self, user_id: &str) -> Result<Vec<entities::wallets::Model>, AppError> {
        ops::list_wallets(&*self.db, user_id).await
    }

    pub async fn get(
        &self,
        user_id: &str,
        wallet_id: &str,
    ) -> Result<entities::wallets::Model, AppError> {
        ops::get_wallet(&*self.db, user_id, wallet_id).await
    }

    pub async fn update(
        &self,
        user_id: &str,
        wallet_id: &str,
        name: Option<String>,
        balance: Option<Decimal>,
    ) -> Result<entities::wallets::Model, AppError> {
        let result = ops::update_wallet(&*self.db, user_id, wallet_id, name, balance).await?;
        self.resolve_cache.invalidate_all();
        Ok(result)
    }

    pub async fn delete(&self, user_id: &str, wallet_id: &str) -> Result<u64, AppError> {
        let res = ops::delete_wallet(&*self.db, user_id, wallet_id).await?;
        if res > 0 {
            self.resolve_cache.invalidate_all();
        }
        Ok(res)
    }

    pub async fn resolve<C>(
        &self,
        db: &C,
        user_id: &str,
        params: ops::ResolveWalletParams,
    ) -> Result<entities::wallets::Model, AppError>
    where
        C: ConnectionTrait,
    {
        let key = (user_id.to_string(), params.clone());
        if let Some(cached) = self.resolve_cache.get(&key).await {
            return Ok(cached);
        }

        let wallet = ops::resolve_wallet(db, user_id, params).await?;
        self.resolve_cache.insert(key, wallet.clone()).await;
        Ok(wallet)
    }

    pub async fn adjust_balance<C>(
        &self,
        db: &C,
        wallet_id: &str,
        amount: Decimal,
        allow_negative: bool,
    ) -> Result<(), AppError>
    where
        C: ConnectionTrait,
    {
        ops::adjust_balance(db, wallet_id, amount, allow_negative).await
    }
}
