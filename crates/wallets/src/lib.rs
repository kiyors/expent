use db::AppError;
use db::entities;
use db::entities::enums::WalletType;
use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, DatabaseConnection};
use std::sync::Arc;

pub mod ops;

#[derive(Debug, Clone)]
pub struct WalletsManager {
    db: Arc<DatabaseConnection>,
}

impl WalletsManager {
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: &str,
        name: &str,
        wallet_type: WalletType,
        initial_balance: Decimal,
    ) -> Result<entities::wallets::Model, AppError> {
        ops::create_wallet(&*self.db, user_id, name, wallet_type, initial_balance).await
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
        ops::update_wallet(&*self.db, user_id, wallet_id, name, balance).await
    }

    pub async fn delete(&self, user_id: &str, wallet_id: &str) -> Result<u64, AppError> {
        ops::delete_wallet(&*self.db, user_id, wallet_id).await
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
        ops::resolve_wallet(db, user_id, params).await
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
