use super::OcrExtractionStrategy;
use ::contacts::ContactsManager;
use ::wallets::WalletsManager;
use async_trait::async_trait;
use chrono::Utc;
use db::entities::enums::{TransactionDirection, TransactionStatus};
use db::{AppError, OcrResult, OcrTransactionResponse, ProcessedOcr};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DatabaseTransaction, Set};
use std::sync::Arc;

pub struct GenericStrategy;

#[async_trait]
impl OcrExtractionStrategy for GenericStrategy {
    async fn enrich(
        &self,
        db: &DatabaseConnection,
        contacts_manager: Arc<ContactsManager>,
        _wallets_manager: Arc<WalletsManager>,
        user_id: &str,
        mut processed: ProcessedOcr,
    ) -> Result<ProcessedOcr, AppError> {
        let mut generic: OcrResult = serde_json::from_value(processed.data.0.clone())
            .map_err(|e| AppError::Ocr(format!("Failed to parse generic data: {}", e)))?;

        if let Some(vendor) = &generic.vendor {
            let resolution = contacts_manager
                .resolve(
                    db,
                    user_id,
                    ::contacts::ops::ResolveParams {
                        name: Some(vendor.clone()),
                        phone: None,
                        email: None,
                        upi_id: generic.upi_id.clone(),
                    },
                )
                .await?;

            if let Some(c_id) = resolution.contact_id {
                generic.contact_id = Some(c_id);
            }
        }

        processed.data.0 = serde_json::to_value(generic)
            .map_err(|e| AppError::Ocr(format!("Serialization error: {}", e)))?;
        Ok(processed)
    }

    async fn extract_and_save(
        &self,
        txn_db: &DatabaseTransaction,
        contacts_manager: Arc<ContactsManager>,
        _wallets_manager: Arc<WalletsManager>,
        user_id: &str,
        processed: ProcessedOcr,
    ) -> Result<OcrTransactionResponse, AppError> {
        let generic: OcrResult = serde_json::from_value(processed.data.0.clone())
            .map_err(|e| AppError::Ocr(format!("Failed to parse generic data: {}", e)))?;

        let mut contact_id = generic.contact_id.clone();
        let wallet_id = generic.wallet_id.clone();
        let category_id = generic.category_id.clone();

        if contact_id.is_none() && generic.vendor.is_some() {
            let resolution = contacts_manager
                .resolve(
                    txn_db,
                    user_id,
                    ::contacts::ops::ResolveParams {
                        name: generic.vendor.clone(),
                        phone: None,
                        email: None,
                        upi_id: generic.upi_id.clone(),
                    },
                )
                .await?;

            if resolution.is_collision {
                return Err(AppError::ContactCollision(
                    serde_json::to_value(resolution.collision_candidates)
                        .map_err(|e| AppError::Ocr(format!("Serialization error: {}", e)))?,
                ));
            }

            if let Some(c_id) = resolution.contact_id {
                contact_id = Some(c_id);
            }
        }

        let amount = generic.amount.unwrap_or(Decimal::ZERO);

        let txn = db::entities::transactions::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            amount: Set(amount),
            direction: Set(TransactionDirection::Out),
            status: Set(TransactionStatus::Completed),
            date: Set(generic
                .date
                .map(|dt| dt.with_timezone(&Utc).into())
                .unwrap_or_else(|| Utc::now().into())),
            source_wallet_id: Set(wallet_id),
            category_id: Set(category_id),
            notes: Set(Some(format!(
                "Extracted via Generic OCR. Vendor: {:?}",
                generic.vendor
            ))),
            ..Default::default()
        };

        let result = txn.insert(txn_db).await?;

        if let Some(c_id) = contact_id {
            let party = db::entities::txn_parties::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(result.id.clone()),
                contact_id: Set(Some(c_id)),
                role: Set(TransactionDirection::Out.counterparty_role()), // Receiver for Outgoing
                ..Default::default()
            };
            party.insert(txn_db).await?;
        }

        if let Some(r2_key) = processed.r2_key {
            let source = db::entities::transaction_sources::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(result.id.clone()),
                source_type: Set("OCR".to_string()),
                r2_file_url: Set(Some(r2_key)),
                raw_metadata: Set(Some(processed.data.0)),
            };
            source.insert(txn_db).await?;
        }

        Ok(OcrTransactionResponse {
            transaction: result,
            contact_created: false,
            batch_count: 1,
        })
    }
}
