use chrono::{DateTime, FixedOffset};
use db::AppError;
use db::entities;
use rust_decimal::Decimal;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub mod matching;
pub mod statement;

#[derive(Clone)]
pub struct ReconciliationManager {
    db: Arc<DatabaseConnection>,
}

impl ReconciliationManager {
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn list_unmatched_rows(
        &self,
        user_id: &str,
    ) -> Result<Vec<entities::bank_statement_rows::Model>, AppError> {
        matching::list_unmatched_rows(&*self.db, user_id).await
    }

    pub async fn get_row_matches(
        &self,
        user_id: &str,
        row_id: &str,
    ) -> Result<Vec<(entities::transactions::Model, i32)>, AppError> {
        matching::get_row_matches(&*self.db, user_id, row_id).await
    }

    pub async fn confirm_match(
        &self,
        user_id: &str,
        row_id: &str,
        txn_id: &str,
        confidence: i32,
    ) -> Result<(), AppError> {
        matching::confirm_match(&*self.db, user_id, row_id, txn_id, confidence).await
    }

    pub async fn upload_statement(
        &self,
        user_id: &str,
        date: DateTime<FixedOffset>,
        description: String,
        amount: Decimal,
        raw_data: Option<serde_json::Value>,
    ) -> Result<entities::bank_statement_rows::Model, AppError> {
        statement::upload_statement(&*self.db, user_id, date, description, amount, raw_data).await
    }
}
