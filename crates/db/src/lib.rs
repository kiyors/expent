use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod dto;
pub mod entities;
pub mod error;

pub use error::AppError;

/// Represents a single line item in a purchase, typically extracted via OCR.
#[derive(Debug, Serialize, Deserialize, Clone, TS, JsonSchema)]
#[ts(export, rename = "LineItem")]
pub struct LineItem {
    pub name: String,
    pub quantity: i32,
    #[ts(type = "string")]
    #[serde(with = "rust_decimal::serde::str")]
    #[schemars(with = "String")]
    pub price: Decimal,
}

/// The result of an OCR process, containing raw text and extracted transaction details.
#[derive(Debug, Serialize, Deserialize, Clone, TS, JsonSchema)]
#[ts(export, rename = "OcrResult")]
pub struct OcrResult {
    pub raw_text: String,
    pub vendor: Option<String>,
    #[ts(type = "string | null")]
    #[schemars(with = "Option<String>")]
    pub amount: Option<Decimal>,
    #[schemars(with = "Option<String>")]
    pub date: Option<DateTime<FixedOffset>>,
    pub upi_id: Option<String>,
    pub category_id: Option<String>,
    pub wallet_id: Option<String>,
    pub contact_id: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence_score: f32,
    #[serde(default)]
    pub items: Vec<LineItem>,
}

fn default_confidence() -> f32 {
    1.0
}

/// Specialized extraction for Google Pay screenshots.
#[derive(Debug, Serialize, Deserialize, Clone, TS, JsonSchema)]
#[ts(export, rename = "GPayExtraction")]
pub struct GPayExtraction {
    #[ts(type = "string")]
    #[serde(with = "rust_decimal::serde::str")]
    #[schemars(with = "String")]
    pub amount: Decimal,
    pub direction: String, // "IN" | "OUT"
    pub datetime_str: Option<String>,
    pub status: Option<String>,
    pub counterparty_name: String,
    pub counterparty_phone: Option<String>,
    pub counterparty_upi_id: Option<String>,
    pub is_merchant: bool,
    pub upi_transaction_id: Option<String>,
    pub google_transaction_id: Option<String>,
    pub source_bank_account: Option<String>,
    pub category_id: Option<String>,
    pub wallet_id: Option<String>,
    pub contact_id: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence_score: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS, JsonSchema)]
#[ts(export, rename = "BankTransaction")]
pub struct BankTransaction {
    pub transaction_date: String,
    pub description: String,
    pub mode: Option<String>,
    #[ts(type = "string | null")]
    #[serde(with = "rust_decimal::serde::str_option")]
    #[schemars(with = "Option<String>")]
    pub debit_amount: Option<Decimal>,
    #[ts(type = "string | null")]
    #[serde(with = "rust_decimal::serde::str_option")]
    #[schemars(with = "Option<String>")]
    pub credit_amount: Option<Decimal>,
    #[ts(type = "string | null")]
    #[serde(with = "rust_decimal::serde::str_option")]
    #[schemars(with = "Option<String>")]
    pub balance: Option<Decimal>,
    pub contact_name: Option<String>,
    pub upi_id: Option<String>,
    pub reference_number: Option<String>,
    pub category_id: Option<String>,
    pub wallet_id: Option<String>,
    pub contact_id: Option<String>,
    pub metadata: Option<ExportedJsonValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS, JsonSchema)]
#[ts(export, rename = "BankStatementResponse")]
pub struct BankStatementResponse {
    pub transactions: Vec<BankTransaction>,
    pub bank_name: String,
    pub account_number: Option<String>,
    pub statement_period: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS, JsonSchema)]
#[ts(export, rename = "BankExtractionResult")]
pub struct BankExtractionResult {
    pub raw_text: String,
    pub doc_type: String,
    pub confidence_score: f32,
    pub bank_data: BankStatementResponse,
}

/// Unified OCR data from the Python worker.
#[derive(Debug, Serialize, Deserialize, Clone, TS, JsonSchema)]
#[ts(export, rename = "ProcessedOcr")]
pub struct ProcessedOcr {
    pub doc_type: String,        // "GPAY" or "GENERIC"
    pub data: ExportedJsonValue, // Use ExportedJsonValue instead of serde_json::Value
    pub r2_key: Option<String>,
    #[serde(default)]
    pub is_high_res: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS, JsonSchema)]
#[serde(tag = "doc_type", rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum TypedProcessedOcr {
    #[serde(rename = "GPAY")]
    Gpay {
        data: GPayExtraction,
        r2_key: Option<String>,
    },
    #[serde(rename = "BANK_STATEMENT")]
    BankStatement {
        data: BankExtractionResult,
        r2_key: Option<String>,
    },
    #[serde(rename = "GENERIC")]
    Generic {
        data: OcrResult,
        r2_key: Option<String>,
    },
}

/// A type alias for `serde_json::Value` to control its TypeScript export location.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, rename = "JsonValue")]
pub struct ExportedJsonValue(
    #[ts(
        type = "number | string | boolean | Array<JsonValue> | { [key: string]: JsonValue } | null"
    )]
    pub serde_json::Value,
);

/// Details for splitting a transaction among multiple users.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, rename = "SplitDetail")]
pub struct SplitDetail {
    pub receiver_email: String,
    #[ts(type = "string")]
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
}

/// P2P request with sender's name.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, rename = "P2pRequestWithSender")]
pub struct P2pRequestWithSender {
    #[serde(flatten)]
    pub request: entities::p2p_requests::Model,
    pub sender_name: Option<String>,
}

/// Response for OCR transaction creation.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, rename = "OcrTransactionResponse")]
pub struct OcrTransactionResponse {
    pub transaction: entities::transactions::Model,
    pub contact_created: bool,
    #[serde(default = "default_batch_count")]
    pub batch_count: i32,
}

const fn default_batch_count() -> i32 {
    1
}

/// Transaction with optional wallet and contact info.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, rename = "TransactionWithDetail")]
pub struct TransactionWithDetail {
    #[serde(flatten)]
    pub transaction: entities::transactions::Model,
    pub source_wallet_name: Option<String>,
    pub destination_wallet_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_id: Option<String>,
    pub category_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS, FromQueryResult)]
#[ts(export, rename = "GroupMemberDetail")]
pub struct GroupMemberDetail {
    pub user_id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, rename = "ContactDetail")]
pub struct ContactDetail {
    pub contact: entities::contacts::Model,
    pub identifiers: Vec<entities::contact_identifiers::Model>,
    pub transactions: Vec<entities::transactions::Model>,
}

/// Paginated response for transactions.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, rename = "PaginatedTransactions")]
pub struct PaginatedTransactions {
    pub items: Vec<TransactionWithDetail>,
    pub total_count: u64,
}

/// Trend data for a single month.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, rename = "MonthlyTrend")]
pub struct MonthlyTrend {
    pub month: String,
    #[ts(type = "string")]
    pub income: Decimal,
    #[ts(type = "string")]
    pub expense: Decimal,
}

/// Distribution of expenses by category or contact.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, rename = "NamedAmount")]
pub struct NamedAmount {
    pub name: String,
    #[ts(type = "string")]
    pub amount: Decimal,
}

/// Summary data for the dashboard.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, rename = "DashboardSummary")]
pub struct DashboardSummary {
    #[ts(type = "string")]
    pub total_balance: Decimal,
    #[ts(type = "string")]
    pub monthly_spend: Decimal,
    #[ts(type = "string")]
    pub monthly_income: Decimal,
    pub pending_p2p_count: u64,
    pub total_transactions: u64,
    pub monthly_trends: Vec<MonthlyTrend>,
    pub weekly_trends: Vec<MonthlyTrend>, // Reuse MonthlyTrend for weekly too
    pub category_distribution: Vec<NamedAmount>,
    pub top_expenses: Vec<NamedAmount>,
    pub top_income: Vec<NamedAmount>,
}
