use chrono::{DateTime, FixedOffset};
use db::AppError;
use db::entities;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub struct StatementRowInput {
    pub date: DateTime<FixedOffset>,
    pub description: String,
    pub amount: Decimal,
    pub raw_data: Option<serde_json::Value>,
}

#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub async fn upload_statement_batch(
    db: &DatabaseConnection,
    user_id: &str,
    rows: Vec<StatementRowInput>,
) -> Result<(), AppError> {
    if rows.is_empty() {
        return Ok(());
    }

    let min_date = rows
        .iter()
        .map(|r| r.date)
        .min()
        .expect("rows must not be empty");
    let max_date = rows
        .iter()
        .map(|r| r.date)
        .max()
        .expect("rows must not be empty");

    let existing_rows = entities::bank_statement_rows::Entity::find()
        .filter(entities::bank_statement_rows::Column::UserId.eq(user_id))
        .filter(entities::bank_statement_rows::Column::Date.gte(min_date))
        .filter(entities::bank_statement_rows::Column::Date.lte(max_date))
        .all(db)
        .await?;

    use std::collections::HashSet;
    let existing_set: HashSet<(
        DateTime<FixedOffset>,
        String,
        Option<Decimal>,
        Option<Decimal>,
    )> = existing_rows
        .into_iter()
        .map(|r| {
            (
                r.date,
                r.description.trim().to_lowercase(),
                r.debit,
                r.credit,
            )
        })
        .collect();

    let mut to_insert = Vec::new();

    for row in rows {
        let (debit, credit) = if row.amount < Decimal::ZERO {
            (Some(row.amount.abs()), None)
        } else {
            (None, Some(row.amount))
        };

        if existing_set.contains(&(
            row.date,
            row.description.trim().to_lowercase(),
            debit,
            credit,
        )) {
            tracing::info!("⏭️ Skipping duplicate statement row: {}", row.description);
            continue;
        }

        to_insert.push(entities::bank_statement_rows::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            date: Set(row.date),
            description: Set(row.description),
            debit: Set(debit),
            credit: Set(credit),
            balance: Set(Decimal::ZERO),
            is_matched: Set(false),
        });
    }

    if !to_insert.is_empty() {
        entities::bank_statement_rows::Entity::insert_many(to_insert)
            .exec(db)
            .await?;
    }

    Ok(())
}

#[allow(clippy::missing_errors_doc)]
pub async fn upload_statement(
    db: &DatabaseConnection,
    user_id: &str,
    date: DateTime<FixedOffset>,
    description: String,
    amount: Decimal,
    _raw_data: Option<serde_json::Value>,
) -> Result<entities::bank_statement_rows::Model, AppError> {
    let (debit, credit) = if amount < Decimal::ZERO {
        (Some(amount.abs()), None)
    } else {
        (None, Some(amount))
    };

    // Duplicate check: avoid uploading the same row multiple times
    let existing = entities::bank_statement_rows::Entity::find()
        .filter(entities::bank_statement_rows::Column::UserId.eq(user_id))
        .filter(entities::bank_statement_rows::Column::Date.eq(date))
        .filter(entities::bank_statement_rows::Column::Description.eq(description.clone()))
        .filter(entities::bank_statement_rows::Column::Debit.eq(debit))
        .filter(entities::bank_statement_rows::Column::Credit.eq(credit))
        .one(db)
        .await?;

    if let Some(row) = existing {
        tracing::info!("⏭️ Skipping duplicate statement row: {}", description);
        return Ok(row);
    }

    let row = entities::bank_statement_rows::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        date: Set(date),
        description: Set(description),
        debit: Set(debit),
        credit: Set(credit),
        balance: Set(Decimal::ZERO),
        is_matched: Set(false),
    };
    row.insert(db).await.map_err(AppError::from)
}
