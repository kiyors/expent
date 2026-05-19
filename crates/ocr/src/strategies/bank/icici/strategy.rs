use crate::strategies::OcrExtractionStrategy;
use ::contacts::ContactsManager;
use ::wallets::WalletsManager;
use async_trait::async_trait;
use chrono::Utc;
use db::entities::enums::{IdentifierType, TransactionDirection, TransactionStatus};
use db::{AppError, BankExtractionResult, OcrTransactionResponse, ProcessedOcr};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DatabaseTransaction, Set};
use std::collections::HashMap;
use std::sync::Arc;

pub struct BankStatementStrategy;

#[async_trait]
impl OcrExtractionStrategy for BankStatementStrategy {
    async fn enrich(
        &self,
        db: &DatabaseConnection,
        contacts_manager: Arc<ContactsManager>,
        wallets_manager: Arc<WalletsManager>,
        user_id: &str,
        mut processed: ProcessedOcr,
    ) -> Result<ProcessedOcr, AppError> {
        let mut bank_result: BankExtractionResult =
            serde_json::from_value(processed.data.0.clone()).map_err(|e| {
                AppError::Ocr(format!("Failed to parse bank statement data: {}", e))
            })?;

        let wallet = wallets_manager
            .resolve(
                db,
                user_id,
                ::wallets::ops::ResolveWalletParams {
                    bank_name: bank_result.bank_data.bank_name.clone(),
                    account_number: bank_result.bank_data.account_number.clone(),
                },
            )
            .await?;

        for bt in &mut bank_result.bank_data.transactions {
            if bt.wallet_id.is_none() {
                bt.wallet_id = Some(wallet.id.clone());
            }

            if let Some(contact_name) = &bt.contact_name {
                let resolution = contacts_manager
                    .resolve(
                        db,
                        user_id,
                        ::contacts::ops::ResolveParams {
                            name: Some(contact_name.clone()),
                            phone: None,
                            email: None,
                            upi_id: bt.upi_id.clone(),
                        },
                    )
                    .await?;

                if let Some(c_id) = resolution.contact_id {
                    bt.contact_id = Some(c_id);
                }
            }
        }

        processed.data.0 = serde_json::to_value(bank_result)
            .map_err(|e| AppError::Ocr(format!("Serialization error: {}", e)))?;
        Ok(processed)
    }

    async fn extract_and_save(
        &self,
        txn_db: &DatabaseTransaction,
        contacts_manager: Arc<ContactsManager>,
        wallets_manager: Arc<WalletsManager>,
        user_id: &str,
        processed: ProcessedOcr,
    ) -> Result<OcrTransactionResponse, AppError> {
        let bank_result: BankExtractionResult = serde_json::from_value(processed.data.0.clone())
            .map_err(|e| AppError::Ocr(format!("Failed to parse bank statement data: {}", e)))?;

        let wallet = wallets_manager
            .resolve(
                txn_db,
                user_id,
                ::wallets::ops::ResolveWalletParams {
                    bank_name: bank_result.bank_data.bank_name.clone(),
                    account_number: bank_result.bank_data.account_number.clone(),
                },
            )
            .await?;

        let mut contact_created = false;
        let mut last_txn = None;
        let mut total_processed = 0;
        let mut local_contact_cache: HashMap<String, String> = HashMap::new();

        let resolve_batch: Vec<::contacts::ops::ResolveParams> = bank_result
            .bank_data
            .transactions
            .iter()
            .map(|bt| ::contacts::ops::ResolveParams {
                name: bt.contact_name.clone(),
                phone: None,
                email: None,
                upi_id: bt.upi_id.clone(),
            })
            .collect();

        let bulk_resolutions = contacts_manager
            .resolve_bulk(txn_db, user_id, resolve_batch)
            .await?;
        let mut bulk_iter = bulk_resolutions.into_iter();

        for bt in bank_result.bank_data.transactions {
            let mut current_contact_id = None;
            let resolution = bulk_iter.next().expect("bulk_iter size mismatch");

            if let Some(contact_name) = &bt.contact_name {
                if let Some(c_id) = local_contact_cache.get(contact_name) {
                    current_contact_id = Some(c_id.clone());
                } else if let Some(c_id) = resolution.contact_id {
                    current_contact_id = Some(c_id);
                } else {
                    let c_result = ::contacts::ops::create_contact(
                        txn_db,
                        user_id,
                        contact_name.clone(),
                        None,
                    )
                    .await?;
                    let c_id = c_result.id.clone();
                    current_contact_id = Some(c_id.clone());
                    contact_created = true;
                    local_contact_cache.insert(contact_name.clone(), c_id.clone());

                    if let Some(upi) = &bt.upi_id {
                        let _ = ::contacts::ops::add_contact_identifier(
                            txn_db,
                            user_id,
                            &c_id,
                            IdentifierType::Upi,
                            upi.clone(),
                        )
                        .await;
                    }
                }
            }

            let amount = bt
                .debit_amount
                .or(bt.credit_amount)
                .unwrap_or(Decimal::ZERO);

            let direction = if bt.debit_amount.is_some() {
                TransactionDirection::Out
            } else {
                TransactionDirection::In
            };

            let timestamp =
                crate::utils::parse_bank_date(&bt.transaction_date).unwrap_or_else(Utc::now);

            let txn = db::entities::transactions::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                user_id: Set(user_id.to_string()),
                amount: Set(amount),
                direction: Set(direction),
                status: Set(TransactionStatus::Completed),
                date: Set(timestamp.into()),
                source_wallet_id: Set(bt.wallet_id.clone().or(Some(wallet.id.clone()))),
                category_id: Set(bt.category_id.clone()),
                notes: Set(Some(bt.description.clone())),
                ..Default::default()
            };

            let result = txn.insert(txn_db).await?;

            if let Some(c_id) = current_contact_id {
                let party = db::entities::txn_parties::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    transaction_id: Set(result.id.clone()),
                    contact_id: Set(Some(c_id)),
                    role: Set(direction.counterparty_role()),
                    ..Default::default()
                };
                party.insert(txn_db).await?;
            }

            if total_processed == 0 {
                if let Some(r2_key) = processed.r2_key.clone() {
                    let source = db::entities::transaction_sources::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        transaction_id: Set(result.id.clone()),
                        source_type: Set("OCR".to_string()),
                        r2_file_url: Set(Some(r2_key)),
                        raw_metadata: Set(Some(processed.data.0.clone())),
                    };
                    source.insert(txn_db).await?;
                }
            }

            last_txn = Some(result);
            total_processed += 1;
        }

        let final_txn = last_txn
            .ok_or_else(|| AppError::Ocr("No transactions found in bank statement".to_string()))?;

        Ok(OcrTransactionResponse {
            transaction: final_txn,
            contact_created,
            batch_count: total_processed,
        })
    }
}
