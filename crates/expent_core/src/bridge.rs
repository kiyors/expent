use ::contacts::ContactsManager;
use chrono::{DateTime, Utc};
use db::entities;
use db::entities::enums::{
    IdentifierType, TransactionDirection, TransactionSource, TransactionStatus, TxnPartyRole,
};
use db::{AppError, GPayExtraction, OcrResult, OcrTransactionResponse, ProcessedOcr};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use std::sync::Arc;

pub async fn process_ocr(
    db: &DatabaseConnection,
    contacts_manager: Arc<ContactsManager>,
    user_id: &str,
    processed: ProcessedOcr,
) -> Result<OcrTransactionResponse, AppError> {
    let user_id = user_id.to_string();

    // 3.2 Idempotency check: if r2_key is provided, check if we already have a transaction for it
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
            });
        }
    }

    db.transaction::<_, OcrTransactionResponse, AppError>(|txn_db| {
        let contacts_manager = contacts_manager.clone();
        let user_id = user_id.clone();
        Box::pin(async move {
            let mut contact_created = false;

            if processed.doc_type == "GPAY" {
                let gpay: GPayExtraction = serde_json::from_value(processed.data.0.clone())
                    .map_err(|e| AppError::Ocr(format!("Failed to parse GPAY data: {}", e)))?;

                let mut contact_id = gpay.contact_id.clone();
                let wallet_id = gpay.wallet_id.clone();
                let category_id = gpay.category_id.clone();

                // 2.6 Robust Contact Resolution
                if contact_id.is_none() {
                    let resolution = contacts_manager
                        .resolve(
                            txn_db,
                            &user_id,
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
                            serde_json::to_value(resolution.collision_candidates).unwrap(),
                        ));
                    }

                    if let Some(c_id) = resolution.contact_id {
                        contact_id = Some(c_id);
                    } else {
                        // Create new contact using manager if possible (currently manager uses self.db, but we are inside txn_db)
                        // For now we keep manual entity insertion but we use the resolution logic from manager
                        let new_contact = entities::contacts::ActiveModel {
                            id: Set(uuid::Uuid::now_v7().to_string()),
                            name: Set(gpay.counterparty_name.clone()),
                            phone: Set(gpay.counterparty_phone.clone()),
                            is_pinned: Set(false),
                            normalized_name: Set(Some(
                                gpay.counterparty_name.trim().to_lowercase(),
                            )),
                            phonetic_name: Set(Some(
                                rphonetic::DoubleMetaphone::default()
                                    .double_metaphone(&gpay.counterparty_name)
                                    .primary(),
                            )),
                        };
                        let c_result = new_contact.insert(txn_db).await?;
                        contact_id = Some(c_result.id.clone());
                        contact_created = true;

                        // Create identifier
                        if let Some(upi_id) = &gpay.counterparty_upi_id {
                            let new_ident = entities::contact_identifiers::ActiveModel {
                                id: Set(uuid::Uuid::now_v7().to_string()),
                                contact_id: Set(c_result.id.clone()),
                                r#type: Set(IdentifierType::Upi),
                                value: Set(upi_id.clone()),
                                linked_user_id: Set(None),
                            };
                            new_ident.insert(txn_db).await?;
                        }

                        // Create link for user
                        let new_link = entities::contact_links::ActiveModel {
                            user_id: Set(user_id.clone()),
                            contact_id: Set(c_result.id),
                        };
                        new_link.insert(txn_db).await?;
                    }
                }

                let direction = match gpay.direction.as_str() {
                    "IN" => TransactionDirection::In,
                    _ => TransactionDirection::Out,
                };

                // Parse status
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

                let txn = entities::transactions::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    user_id: Set(user_id.clone()),
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
                    source: Set(TransactionSource::Ocr),
                    ..Default::default()
                };

                let result = txn.insert(txn_db).await?;

                // Create transaction party record for the contact
                if let Some(c_id) = contact_id {
                    let party = entities::txn_parties::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        transaction_id: Set(result.id.clone()),
                        user_id: Set(None),
                        contact_id: Set(Some(c_id)),
                        role: Set(match direction {
                            TransactionDirection::In => TxnPartyRole::Sender,
                            TransactionDirection::Out => TxnPartyRole::Receiver,
                        }),
                    };
                    party.insert(txn_db).await?;
                }

                // Create transaction source record
                if let Some(r2_key) = processed.r2_key {
                    let source = entities::transaction_sources::ActiveModel {
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
                })
            } else {
                // Generic OCR path
                let generic: OcrResult = serde_json::from_value(processed.data.0.clone())
                    .map_err(|e| AppError::Ocr(format!("Failed to parse generic data: {}", e)))?;

                let mut contact_id = generic.contact_id.clone();
                let wallet_id = generic.wallet_id.clone();
                let category_id = generic.category_id.clone();

                // 2.6 Robust Contact Resolution for Generic OCR
                if contact_id.is_none() && generic.vendor.is_some() {
                    let resolution = contacts_manager
                        .resolve(
                            txn_db,
                            &user_id,
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
                            serde_json::to_value(resolution.collision_candidates).unwrap(),
                        ));
                    }

                    if let Some(c_id) = resolution.contact_id {
                        contact_id = Some(c_id);
                    }
                }

                let amount = generic.amount.unwrap_or(Decimal::ZERO);

                let txn = entities::transactions::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    user_id: Set(user_id.clone()),
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
                    source: Set(TransactionSource::Ocr),
                    ..Default::default()
                };

                let result = txn.insert(txn_db).await?;

                // Create transaction party record for the contact
                if let Some(c_id) = contact_id {
                    let party = entities::txn_parties::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        transaction_id: Set(result.id.clone()),
                        user_id: Set(None),
                        contact_id: Set(Some(c_id)),
                        role: Set(TxnPartyRole::Receiver),
                    };
                    party.insert(txn_db).await?;
                }

                // Create transaction source record
                if let Some(r2_key) = processed.r2_key {
                    let source = entities::transaction_sources::ActiveModel {
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
                })
            }
        })
    })
    .await
    .map_err(|e| match e {
        sea_orm::TransactionError::Connection(ce) => AppError::Db(ce),
        sea_orm::TransactionError::Transaction(te) => te,
    })
}
