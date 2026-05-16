use crate::ocr_strategies::get_strategy;
use ::contacts::ContactsManager;
use ::wallets::WalletsManager;
use chrono::{DateTime, Utc};
use db::entities;
use db::{AppError, OcrTransactionResponse, ProcessedOcr};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use std::sync::Arc;

pub async fn enrich_ocr(
    db: &DatabaseConnection,
    contacts_manager: Arc<ContactsManager>,
    wallets_manager: Arc<WalletsManager>,
    user_id: &str,
    processed: ProcessedOcr,
) -> Result<ProcessedOcr, AppError> {
    let strategy = get_strategy(&processed.doc_type);
    strategy
        .enrich(db, contacts_manager, wallets_manager, user_id, processed)
        .await
}

pub async fn process_ocr(
    db: &DatabaseConnection,
    contacts_manager: Arc<ContactsManager>,
    wallets_manager: Arc<WalletsManager>,
    user_id: &str,
    processed: ProcessedOcr,
) -> Result<OcrTransactionResponse, AppError> {
    // 3.2 Idempotency check
    if let Some(ref key) = processed.r2_key {
        let existing_source = entities::transaction_sources::Entity::find()
            .filter(entities::transaction_sources::Column::R2FileUrl.eq(Some(key.to_string())))
            .one(db)
            .await?;

        if let Some(source) = existing_source {
            let txn = entities::transactions::Entity::find_by_id(source.transaction_id)
                .one(db)
                .await?
                .ok_or_else(|| {
                    AppError::Generic("Source exists but transaction not found".to_string())
                })?;

            return Ok(OcrTransactionResponse {
                transaction: txn,
                contact_created: false,
                batch_count: 1,
            });
        }
    }

    let user_id_owned = user_id.to_string();
    let contacts_clone = contacts_manager.clone();
    let wallets_clone = wallets_manager.clone();
    let doc_type = processed.doc_type.clone();

    db.transaction::<_, OcrTransactionResponse, AppError>(move |txn_db| {
        let user_id = user_id_owned;
        let processed = processed;
        let contacts = contacts_clone;
        let wallets = wallets_clone;
        let doc_type = doc_type;

        Box::pin(async move {
            let strategy = get_strategy(&doc_type);
            strategy
                .extract_and_save(txn_db, contacts, wallets, &user_id, processed)
                .await
        })
    })
    .await
    .map_err(|e| match e {
        sea_orm::TransactionError::Connection(ce) => AppError::Db(ce),
        sea_orm::TransactionError::Transaction(te) => te,
    })
}

pub fn parse_bank_date(date_str: &str) -> Option<DateTime<Utc>> {
    let formats = [
        "%d-%m-%Y",
        "%d/%m/%Y",
        "%Y-%m-%d",
        "%d-%b-%Y",
        "%d %b %Y",
        "%m/%d/%Y",
        "%b %d, %Y",
    ];
    for fmt in formats {
        if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, fmt) {
            return Some(DateTime::from_naive_utc_and_offset(
                dt.and_hms_opt(0, 0, 0)?,
                Utc,
            ));
        }
    }
    tracing::error!("❌ Failed to parse bank transaction date: '{}'", date_str);
    None
}
