use ::contacts::ContactsManager;
use ::wallets::WalletsManager;
use async_trait::async_trait;
use db::{AppError, OcrTransactionResponse, ProcessedOcr};
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use std::sync::Arc;

#[async_trait]
pub trait OcrExtractionStrategy: Send + Sync {
    /// Enrich the OCR data before user review
    async fn enrich(
        &self,
        db: &DatabaseConnection,
        contacts: Arc<ContactsManager>,
        wallets: Arc<WalletsManager>,
        user_id: &str,
        processed: ProcessedOcr,
    ) -> Result<ProcessedOcr, AppError>;

    /// Extract and save transactions to the database after user confirmation
    async fn extract_and_save(
        &self,
        txn_db: &DatabaseTransaction,
        contacts: Arc<ContactsManager>,
        wallets: Arc<WalletsManager>,
        user_id: &str,
        processed: ProcessedOcr,
    ) -> Result<OcrTransactionResponse, AppError>;
}

pub mod bank_statement;
pub mod generic;
pub mod gpay;

pub fn get_strategy(doc_type: &str) -> Box<dyn OcrExtractionStrategy> {
    match doc_type {
        "BANK_STATEMENT" => Box::new(bank_statement::BankStatementStrategy),
        "GPAY" => Box::new(gpay::GPayStrategy),
        _ => Box::new(generic::GenericStrategy),
    }
}
