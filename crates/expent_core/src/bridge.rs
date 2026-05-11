use ::contacts::ContactsManager;
use chrono::{DateTime, Utc};
use db::entities;
use db::entities::enums::{IdentifierType, TransactionDirection, TransactionStatus, TxnPartyRole};
use db::{
    AppError, BankExtractionResult, GPayExtraction, OcrResult, OcrTransactionResponse, ProcessedOcr,
};
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

    db.transaction::<_, OcrTransactionResponse, AppError>(move |txn_db| {
        let user_id = user_id_owned;
        Box::pin(async move {
            let mut contact_created = false;

            if processed.doc_type == "BANK_STATEMENT" {
                let bank_result: BankExtractionResult =
                    serde_json::from_value(processed.data.0.clone()).map_err(|e| {
                        AppError::Ocr(format!("Failed to parse bank statement data: {}", e))
                    })?;

                let mut last_txn = None;
                let mut total_processed = 0;

                for bt in bank_result.bank_data.transactions {
                    let mut current_contact_id = None;

                    // Contact Resolution
                    if let Some(contact_name) = &bt.contact_name {
                        let resolution = contacts_manager
                            .resolve(
                                txn_db,
                                &user_id,
                                ::contacts::ops::ResolveParams {
                                    name: Some(contact_name.clone()),
                                    phone: None,
                                    email: None,
                                    upi_id: bt.upi_id.clone(),
                                },
                            )
                            .await?;

                        if resolution.is_collision {
                            // For batch processing, we might want to skip or flag collisions
                            // but for now let's just create a new one if it's not clear
                        }

                        if let Some(c_id) = resolution.contact_id {
                            current_contact_id = Some(c_id);
                        } else {
                            // Create new contact
                            let c_result = ::contacts::ops::create_contact(
                                txn_db,
                                &user_id,
                                contact_name.clone(),
                                None,
                            )
                            .await?;
                            current_contact_id = Some(c_result.id.clone());
                            contact_created = true;

                            // Add UPI identifier if present
                            if let Some(upi) = &bt.upi_id {
                                let _ = ::contacts::ops::add_contact_identifier(
                                    txn_db,
                                    &user_id,
                                    &c_result.id,
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

                    let timestamp = parse_bank_date(&bt.transaction_date).unwrap_or_else(Utc::now);

                    let txn = entities::transactions::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        user_id: Set(user_id.clone()),
                        amount: Set(amount),
                        direction: Set(direction),
                        status: Set(TransactionStatus::Completed),
                        date: Set(timestamp.into()),
                        source_wallet_id: Set(bt.wallet_id.clone()),
                        category_id: Set(bt.category_id.clone()),
                        notes: Set(Some(bt.description.clone())),
                        ..Default::default()
                    };

                    let result = txn.insert(txn_db).await?;

                    // Party Link
                    if let Some(c_id) = current_contact_id {
                        let party = entities::txn_parties::ActiveModel {
                            id: Set(uuid::Uuid::now_v7().to_string()),
                            transaction_id: Set(result.id.clone()),
                            contact_id: Set(Some(c_id)),
                            role: Set(match direction {
                                TransactionDirection::In => TxnPartyRole::Sender,
                                TransactionDirection::Out => TxnPartyRole::Receiver,
                            }),
                            ..Default::default()
                        };
                        party.insert(txn_db).await?;
                    }

                    // Source record for the first transaction in batch
                    if total_processed == 0 {
                        if let Some(r2_key) = processed.r2_key.clone() {
                            let source = entities::transaction_sources::ActiveModel {
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

                let final_txn = last_txn.ok_or_else(|| {
                    AppError::Ocr("No transactions found in bank statement".to_string())
                })?;

                Ok(OcrTransactionResponse {
                    transaction: final_txn,
                    contact_created,
                    batch_count: total_processed,
                })
            } else if processed.doc_type == "GPAY" {
                let gpay: GPayExtraction = serde_json::from_value(processed.data.0.clone())
                    .map_err(|e| AppError::Ocr(format!("Failed to parse GPAY data: {}", e)))?;

                let mut contact_id = gpay.contact_id.clone();
                let wallet_id = gpay.wallet_id.clone();
                let category_id = gpay.category_id.clone();

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
                        let c_result = ::contacts::ops::create_contact(
                            txn_db,
                            &user_id,
                            gpay.counterparty_name.clone(),
                            gpay.counterparty_phone.clone(),
                        )
                        .await?;
                        contact_id = Some(c_result.id.clone());
                        contact_created = true;

                        if let Some(upi_id) = &gpay.counterparty_upi_id {
                            let _ = ::contacts::ops::add_contact_identifier(
                                txn_db,
                                &user_id,
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
                    ..Default::default()
                };

                let result = txn.insert(txn_db).await?;

                if let Some(c_id) = contact_id {
                    let party = entities::txn_parties::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        transaction_id: Set(result.id.clone()),
                        contact_id: Set(Some(c_id)),
                        role: Set(match direction {
                            TransactionDirection::In => TxnPartyRole::Sender,
                            TransactionDirection::Out => TxnPartyRole::Receiver,
                        }),
                        ..Default::default()
                    };
                    party.insert(txn_db).await?;
                }

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
                    batch_count: 1,
                })
            } else {
                let generic: OcrResult = serde_json::from_value(processed.data.0.clone())
                    .map_err(|e| AppError::Ocr(format!("Failed to parse generic data: {}", e)))?;

                let mut contact_id = generic.contact_id.clone();
                let wallet_id = generic.wallet_id.clone();
                let category_id = generic.category_id.clone();

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
                    ..Default::default()
                };

                let result = txn.insert(txn_db).await?;

                if let Some(c_id) = contact_id {
                    let party = entities::txn_parties::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        transaction_id: Set(result.id.clone()),
                        contact_id: Set(Some(c_id)),
                        role: Set(TxnPartyRole::Receiver),
                        ..Default::default()
                    };
                    party.insert(txn_db).await?;
                }

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
                    batch_count: 1,
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

fn parse_bank_date(date_str: &str) -> Option<DateTime<Utc>> {
    let formats = ["%d-%m-%Y", "%d/%m/%Y", "%Y-%m-%d"];
    for fmt in formats {
        if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, fmt) {
            return Some(DateTime::from_naive_utc_and_offset(
                dt.and_hms_opt(0, 0, 0)?,
                Utc,
            ));
        }
    }
    None
}
