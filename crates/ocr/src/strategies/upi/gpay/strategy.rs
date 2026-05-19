use crate::strategies::OcrExtractionStrategy;
use ::contacts::ContactsManager;
use ::wallets::WalletsManager;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use db::entities::enums::{IdentifierType, TransactionDirection, TransactionStatus};
use db::{AppError, GPayExtraction, OcrTransactionResponse, ProcessedOcr};
use sea_orm::{ActiveModelTrait, DatabaseConnection, DatabaseTransaction, Set};
use std::sync::Arc;

pub struct GPayStrategy;

#[async_trait]
impl OcrExtractionStrategy for GPayStrategy {
    async fn enrich(
        &self,
        db: &DatabaseConnection,
        contacts_manager: Arc<ContactsManager>,
        _wallets_manager: Arc<WalletsManager>,
        user_id: &str,
        mut processed: ProcessedOcr,
    ) -> Result<ProcessedOcr, AppError> {
        let mut gpay: GPayExtraction = serde_json::from_value(processed.data.0.clone())
            .map_err(|e| AppError::Ocr(format!("Failed to parse GPAY data: {}", e)))?;

        let resolution = contacts_manager
            .resolve(
                db,
                user_id,
                ::contacts::ops::ResolveParams {
                    name: Some(gpay.counterparty_name.clone()),
                    phone: gpay.counterparty_phone.clone(),
                    email: None,
                    upi_id: gpay.counterparty_upi_id.clone(),
                },
            )
            .await?;

        if let Some(c_id) = resolution.contact_id {
            gpay.contact_id = Some(c_id);
        }

        processed.data.0 = serde_json::to_value(gpay)
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
        let gpay: GPayExtraction = serde_json::from_value(processed.data.0.clone())
            .map_err(|e| AppError::Ocr(format!("Failed to parse GPAY data: {}", e)))?;

        let mut contact_id = gpay.contact_id.clone();
        let mut contact_created = false;
        let wallet_id = gpay.wallet_id.clone();
        let category_id = gpay.category_id.clone();

        if contact_id.is_none() {
            let resolution = contacts_manager
                .resolve(
                    txn_db,
                    user_id,
                    ::contacts::ops::ResolveParams {
                        name: Some(gpay.counterparty_name.clone()),
                        phone: gpay.counterparty_phone.clone(),
                        email: None,
                        upi_id: gpay.counterparty_upi_id.clone(),
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
            } else {
                let c_result = ::contacts::ops::create_contact(
                    txn_db,
                    user_id,
                    gpay.counterparty_name.clone(),
                    gpay.counterparty_phone.clone(),
                )
                .await?;
                contact_id = Some(c_result.id.clone());
                contact_created = true;

                if let Some(upi_id) = &gpay.counterparty_upi_id {
                    let _ = ::contacts::ops::add_contact_identifier(
                        txn_db,
                        user_id,
                        &c_result.id,
                        IdentifierType::Upi,
                        upi_id.clone(),
                    )
                    .await;
                }
            }
        }

        let direction = match gpay.direction.as_str() {
            "IN" => TransactionDirection::In,
            _ => TransactionDirection::Out,
        };

        let status = match gpay.status.as_deref() {
            Some("COMPLETED") => TransactionStatus::Completed,
            Some("PENDING") => TransactionStatus::Pending,
            Some("FAILED") | Some("CANCELLED") => TransactionStatus::Cancelled,
            _ => TransactionStatus::Completed,
        };

        let timestamp = gpay
            .datetime_str
            .as_ref()
            .and_then(|s| DateTime::parse_from_str(s, "%d %b %Y, %I:%M %p").ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let txn = db::entities::transactions::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            amount: Set(gpay.amount),
            direction: Set(direction),
            status: Set(status),
            date: Set(timestamp.into()),
            source_wallet_id: Set(wallet_id),
            category_id: Set(category_id),
            notes: Set(Some(format!(
                "Extracted via Google Pay OCR. UPI: {:?}",
                gpay.upi_transaction_id
            ))),
            ..Default::default()
        };

        let result = txn.insert(txn_db).await?;

        if let Some(c_id) = contact_id {
            let party = db::entities::txn_parties::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(result.id.clone()),
                contact_id: Set(Some(c_id)),
                role: Set(direction.counterparty_role()),
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
            contact_created,
            batch_count: 1,
        })
    }
}
