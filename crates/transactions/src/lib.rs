use db::AppError;
use db::entities;
use db::entities::enums::{TransactionDirection, TransactionSource, TransactionStatus};
use db::{DashboardSummary, PaginatedTransactions, SplitDetail};
use rust_decimal::Decimal;
use sea_orm::DatabaseConnection;
use sea_orm::prelude::DateTimeWithTimeZone;
use std::sync::Arc;
use wallets::WalletsManager;

pub mod ops;
pub mod summary;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct TransactionsManager {
    db: Arc<DatabaseConnection>,
    wallets: Arc<WalletsManager>,
}

impl TransactionsManager {
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>, wallets: Arc<WalletsManager>) -> Self {
        Self { db, wallets }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        user_id: &str,
        amount: Decimal,
        direction: TransactionDirection,
        date: chrono::DateTime<chrono::FixedOffset>,
        source: TransactionSource,
        purpose_tag: Option<String>,
        category_id: Option<String>,
        source_wallet_id: Option<String>,
        destination_wallet_id: Option<String>,
        contact_id: Option<String>,
        notes: Option<String>,
    ) -> Result<entities::transactions::Model, AppError> {
        ops::create_transaction(
            &*self.db,
            Arc::clone(&self.wallets),
            user_id,
            amount,
            direction,
            date,
            source,
            purpose_tag,
            category_id,
            source_wallet_id,
            destination_wallet_id,
            contact_id,
            notes,
        )
        .await
    }

    pub async fn list(
        &self,
        user_id: &str,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<PaginatedTransactions, AppError> {
        ops::list_transactions(&*self.db, user_id, limit, offset).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        user_id: &str,
        txn_id: &str,
        amount: Option<Decimal>,
        date: Option<DateTimeWithTimeZone>,
        purpose_tag: Option<String>,
        category_id: Option<String>,
        status: Option<TransactionStatus>,
        notes: Option<String>,
        source_wallet_id: Option<String>,
        destination_wallet_id: Option<String>,
        contact_id: Option<String>,
    ) -> Result<entities::transactions::Model, AppError> {
        ops::update_transaction(
            &*self.db,
            Arc::clone(&self.wallets),
            user_id,
            txn_id,
            amount,
            date,
            purpose_tag,
            category_id,
            status,
            notes,
            source_wallet_id,
            destination_wallet_id,
            contact_id,
        )
        .await
    }

    pub async fn delete(&self, user_id: &str, txn_id: &str) -> Result<u64, AppError> {
        ops::delete_transaction(&*self.db, Arc::clone(&self.wallets), user_id, txn_id).await
    }

    pub async fn split(
        &self,
        user_id: &str,
        txn_id: &str,
        splits: Vec<SplitDetail>,
    ) -> Result<Vec<entities::p2p_requests::Model>, AppError> {
        ops::split_transaction(&*self.db, user_id, txn_id, splits).await
    }

    pub async fn get_summary(&self, user_id: &str) -> Result<DashboardSummary, AppError> {
        summary::get_dashboard_summary(&*self.db, user_id).await
    }
}
