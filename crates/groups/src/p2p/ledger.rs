use chrono::Utc;
use db::AppError;
use db::entities;
use db::entities::enums::{LedgerTabStatus, LedgerTabType};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};

#[allow(clippy::missing_errors_doc)]
pub async fn create_ledger_tab(
    db: &DatabaseConnection,
    creator_id: &str,
    counterparty_id: Option<String>,
    tab_type: LedgerTabType,
    title: &str,
    description: Option<String>,
    target_amount: Decimal,
) -> Result<entities::ledger_tabs::Model, AppError> {
    let tab = entities::ledger_tabs::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        creator_id: Set(creator_id.to_string()),
        counterparty_id: Set(counterparty_id),
        tab_type: Set(tab_type),
        title: Set(title.to_string()),
        description: Set(description),
        target_amount: Set(target_amount),
        status: Set(LedgerTabStatus::Open),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    tab.insert(db).await.map_err(AppError::from)
}
