use chrono::{Duration, Utc};
use db::AppError;
use db::entities;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionError, TransactionTrait,
};

#[allow(clippy::missing_errors_doc)]
pub async fn list_unmatched_rows(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::bank_statement_rows::Model>, AppError> {
    entities::bank_statement_rows::Entity::find()
        .filter(entities::bank_statement_rows::Column::UserId.eq(user_id))
        .filter(entities::bank_statement_rows::Column::IsMatched.eq(false))
        .all(db)
        .await
        .map_err(AppError::from)
}

#[allow(clippy::missing_errors_doc)]
pub async fn get_row_matches(
    db: &DatabaseConnection,
    user_id: &str,
    row_id: &str,
) -> Result<Vec<(entities::transactions::Model, i32)>, AppError> {
    let row = entities::bank_statement_rows::Entity::find_by_id(row_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Statement row not found"))?;

    let amount = row.debit.or(row.credit).unwrap_or(Decimal::ZERO);

    // Find potential transactions within +/- 3 days and matching amount
    let start_date = row.date - Duration::days(3);
    let end_date = row.date + Duration::days(3);

    let txns = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Amount.eq(amount.abs()))
        .filter(entities::transactions::Column::Date.between(start_date, end_date))
        .all(db)
        .await?;

    let mut matches = Vec::new();
    for txn in txns {
        let mut score = 70; // Base score for amount + date range

        if txn.amount == amount.abs() {
            score += 10;
        }
        if txn.date == row.date {
            score += 10;
        }

        // Check narration/description
        if let Some(tag) = &txn.purpose_tag
            && row.description.to_lowercase().contains(&tag.to_lowercase())
        {
            score += 10;
        }

        matches.push((txn, score.min(100)));
    }

    Ok(matches)
}

#[allow(clippy::missing_errors_doc)]
pub async fn confirm_match(
    db: &DatabaseConnection,
    _user_id: &str,
    row_id: &str,
    txn_id: &str,
    confidence: i32,
) -> Result<(), AppError> {
    let row_id = row_id.to_string();
    let txn_id = txn_id.to_string();

    db.transaction::<_, (), AppError>(|txn_db| {
        Box::pin(async move {
            let match_record = entities::statement_txn_matches::ActiveModel {
                row_id: Set(row_id.clone()),
                transaction_id: Set(txn_id),
                confidence: Set(Decimal::from(confidence)),
                matched_at: Set(Utc::now().into()),
            };
            match_record.insert(txn_db).await?;

            let mut row: entities::bank_statement_rows::ActiveModel =
                entities::bank_statement_rows::Entity::find_by_id(row_id)
                    .one(txn_db)
                    .await?
                    .ok_or_else(|| AppError::not_found("Row not found"))?
                    .into();

            row.is_matched = Set(true);
            row.update(txn_db).await?;

            Ok(())
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(ce) => AppError::Db(ce),
        TransactionError::Transaction(te) => te,
    })
}
