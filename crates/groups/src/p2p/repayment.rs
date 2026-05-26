use chrono::Utc;
use db::AppError;
use db::entities;
use db::entities::enums::{
    LedgerTabStatus, TransactionDirection, TransactionSource, TransactionStatus,
};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionError, TransactionTrait,
};

use std::sync::Arc;
use wallets::WalletsManager;

pub async fn register_repayment(
    db: &DatabaseConnection,
    wallets: Arc<WalletsManager>,
    user_id: &str,
    tab_id: &str,
    amount: Decimal,
    wallet_id: Option<String>,
) -> Result<entities::transactions::Model, AppError> {
    let user_id = user_id.to_string();
    let tab_id = tab_id.to_string();
    db.transaction::<_, entities::transactions::Model, AppError>(|txn_db| {
        let wallets = Arc::clone(&wallets);
        Box::pin(async move {
            let tab = entities::ledger_tabs::Entity::find_by_id(tab_id)
                .one(txn_db)
                .await?
                .ok_or_else(|| AppError::not_found("Ledger tab not found"))?;

            if tab.creator_id != user_id && tab.counterparty_id.as_deref() != Some(&user_id) {
                return Err(AppError::unauthorized(
                    "Not authorized to register repayment for this tab",
                ));
            }

            let txn = entities::transactions::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                user_id: Set(user_id.clone()),
                amount: Set(amount),
                direction: Set(TransactionDirection::In),
                date: Set(Utc::now().into()),
                source: Set(TransactionSource::Manual),
                status: Set(TransactionStatus::Completed),
                category_id: Set(None),
                purpose_tag: Set(Some(format!("Repayment for: {}", tab.title))),
                group_id: Set(None),
                source_wallet_id: Set(None),
                destination_wallet_id: Set(wallet_id.clone()),
                ledger_tab_id: Set(Some(tab.id.clone())),
                deleted_at: Set(None),
                notes: Set(None),
            };

            let result = txn.insert(txn_db).await?;

            // Adjust wallet balances using unified logic
            // We need wallets manager here, but since Core re-exports it, we can use it.
            // Actually, in the bridge we'll just point to the ops for now to avoid circular deps if any.
            ::transactions::ops::adjust_transaction_wallets(txn_db, wallets, None, Some(&result))
                .await?;

            let total_paid: Decimal = entities::transactions::Entity::find()
                .filter(entities::transactions::Column::LedgerTabId.eq(tab.id.clone()))
                .filter(entities::transactions::Column::DeletedAt.is_null())
                .all(txn_db)
                .await?
                .iter()
                .map(|t| t.amount)
                .sum();

            if total_paid >= tab.target_amount {
                let mut tab_active: entities::ledger_tabs::ActiveModel = tab.into();
                tab_active.status = Set(LedgerTabStatus::Settled);
                tab_active.update(txn_db).await?;
            } else if total_paid > Decimal::ZERO {
                let mut tab_active: entities::ledger_tabs::ActiveModel = tab.into();
                tab_active.status = Set(LedgerTabStatus::PartiallyPaid);
                tab_active.update(txn_db).await?;
            }

            Ok(result)
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(ce) => AppError::Db(ce),
        TransactionError::Transaction(te) => te,
    })
}
