use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::entities::enums::{
    BudgetPeriod, IdentifierType, TransactionDirection, TransactionStatus, WalletType,
};
use crate::{ProcessedOcr, SplitDetail};

// --- OCR DTOs ---

#[derive(Serialize, Deserialize, TS)]
#[ts(export, rename = "OcrJobResponse")]
pub struct OcrJobResponse {
    pub job_id: String,
    pub status: String,
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "ProcessImageOcrRequest")]
pub struct ProcessImageOcrRequest {
    pub key: String,
    #[ts(optional)]
    pub raw_key: Option<String>,
    #[ts(optional)]
    pub p_hash: Option<String>,
    #[ts(optional)]
    pub auto_confirm: Option<bool>,
    #[ts(optional)]
    pub wallet_id: Option<String>,
    #[ts(optional)]
    pub category_id: Option<String>,
    #[ts(optional)]
    pub batch_id: Option<String>,
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "ConfirmOcrRequest")]
pub struct ConfirmOcrRequest {
    #[ts(optional)]
    pub manual_data: Option<ProcessedOcr>,
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "BulkConfirmOcrRequest")]
pub struct BulkConfirmOcrRequest {
    pub job_ids: Vec<String>,
}

#[derive(Serialize, TS)]
#[ts(export, rename = "BulkConfirmOcrResponse")]
pub struct BulkConfirmOcrResponse {
    pub succeeded: Vec<String>,
    pub failed: Vec<(String, String)>,
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "ResolveContactRequest")]
pub struct ResolveContactRequest {
    pub contact_id: String,
}

// --- Transaction DTOs ---

#[derive(Deserialize, TS, Validate)]
#[ts(export, rename = "CreateManualTransactionRequest")]
pub struct CreateManualTransactionRequest {
    #[validate(custom(function = "validate_positive_decimal"))]
    #[ts(type = "string")]
    pub amount: Decimal,
    pub date: DateTime<FixedOffset>,
    #[validate(length(min = 1, max = 255))]
    pub purpose_tag: String,
    #[ts(optional)]
    pub category_id: Option<String>,
    pub direction: TransactionDirection,
    #[ts(optional)]
    pub source_wallet_id: Option<String>,
    #[ts(optional)]
    pub destination_wallet_id: Option<String>,
    #[ts(optional)]
    pub contact_id: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "UpdateTransactionRequest")]
pub struct UpdateTransactionRequest {
    #[ts(optional, type = "string | null")]
    pub amount: Option<Decimal>,
    #[ts(optional)]
    pub date: Option<DateTime<FixedOffset>>,
    #[ts(optional)]
    pub purpose_tag: Option<String>,
    #[ts(optional)]
    pub category_id: Option<String>,
    #[ts(optional)]
    pub status: Option<TransactionStatus>,
    #[ts(optional)]
    pub notes: Option<String>,
    #[ts(optional)]
    pub source_wallet_id: Option<String>,
    #[ts(optional)]
    pub destination_wallet_id: Option<String>,
    #[ts(optional)]
    pub contact_id: Option<String>,
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "PaginationParams")]
pub struct PaginationParams {
    #[ts(optional)]
    pub limit: Option<u64>,
    #[ts(optional)]
    pub offset: Option<u64>,
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "SplitTransactionRequest")]
pub struct SplitTransactionRequest {
    pub transaction_id: String,
    pub splits: Vec<SplitDetail>,
}

// --- Wallet DTOs ---

#[derive(Deserialize, TS)]
#[ts(export, rename = "CreateWalletRequest")]
pub struct CreateWalletRequest {
    pub name: String,
    pub r#type: WalletType,
    #[ts(type = "string")]
    pub initial_balance: Decimal,
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "UpdateWalletRequest")]
pub struct UpdateWalletRequest {
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional, type = "string | null")]
    pub balance: Option<Decimal>,
}

// --- Budget DTOs ---

#[derive(Deserialize, TS, Validate)]
#[ts(export, rename = "CreateBudgetRequest")]
pub struct CreateBudgetRequest {
    #[ts(optional)]
    pub category_id: Option<String>,
    #[validate(custom(function = "validate_positive_decimal"))]
    #[ts(type = "string")]
    pub amount: Decimal,
    pub period: BudgetPeriod,
}

#[derive(Deserialize, TS, Validate)]
#[ts(export, rename = "UpdateBudgetRequest")]
pub struct UpdateBudgetRequest {
    #[validate(custom(function = "validate_positive_decimal"))]
    #[ts(optional, type = "string | null")]
    pub amount: Option<Decimal>,
    #[ts(optional)]
    pub period: Option<BudgetPeriod>,
}

// --- Category DTOs ---

#[derive(Deserialize, TS)]
#[ts(export, rename = "CreateCategoryRequest")]
pub struct CreateCategoryRequest {
    pub name: String,
    #[ts(optional)]
    pub icon: Option<String>,
    #[ts(optional)]
    pub color: Option<String>,
}

// --- Contact DTOs ---

#[derive(Deserialize, TS, Validate)]
#[ts(export, rename = "CreateContactRequest")]
pub struct CreateContactRequest {
    pub name: String,
    #[ts(optional)]
    pub phone: Option<String>,
}

#[derive(Deserialize, TS, Validate)]
#[ts(export, rename = "UpdateContactRequest")]
pub struct UpdateContactRequest {
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub phone: Option<String>,
    #[ts(optional)]
    pub is_pinned: Option<bool>,
}

#[derive(Deserialize, TS, Validate)]
#[ts(export, rename = "AddIdentifierRequest")]
pub struct AddIdentifierRequest {
    pub r#type: IdentifierType,
    #[validate(length(min = 1, max = 255))]
    pub value: String,
}

#[derive(Deserialize, TS, Validate)]
#[ts(export, rename = "MergeContactsRequest")]
pub struct MergeContactsRequest {
    #[validate(length(min = 1, max = 255))]
    pub primary_id: String,
    #[validate(length(min = 1, max = 255))]
    pub secondary_id: String,
}

// --- Validation Helpers ---

fn validate_positive_decimal(amount: &Decimal) -> Result<(), validator::ValidationError> {
    if amount <= &Decimal::ZERO {
        return Err(validator::ValidationError::new("amount_must_be_positive"));
    }
    Ok(())
}
