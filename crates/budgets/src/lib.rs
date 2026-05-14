use chrono::{Datelike, TimeZone, Utc};
use db::AppError;
use db::entities::{budgets, categories, enums::BudgetPeriod, transactions};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/types/src/db/BudgetHealth.ts")]
pub struct BudgetHealth {
    pub budget_id: String,
    pub category_name: Option<String>,
    #[ts(type = "string")]
    pub limit_amount: Decimal,
    #[ts(type = "string")]
    pub spent_amount: Decimal,
    #[ts(type = "string")]
    pub remaining_amount: Decimal,
    #[ts(type = "string")]
    pub percentage_consumed: Decimal,
    pub period: BudgetPeriod,
}

#[derive(Clone)]
pub struct BudgetsManager {
    db: Arc<DatabaseConnection>,
}

impl BudgetsManager {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: &str,
        category_id: Option<String>,
        amount: Decimal,
        period: BudgetPeriod,
    ) -> Result<budgets::Model, AppError> {
        let budget = budgets::ActiveModel {
            id: ActiveValue::Set(Uuid::now_v7().to_string()),
            user_id: ActiveValue::Set(user_id.to_string()),
            category_id: ActiveValue::Set(category_id),
            amount: ActiveValue::Set(amount),
            period: ActiveValue::Set(period),
            created_at: ActiveValue::Set(Utc::now().naive_utc()),
            updated_at: ActiveValue::Set(Utc::now().naive_utc()),
        };

        Ok(budget.insert(self.db.as_ref()).await?)
    }

    pub async fn list(&self, user_id: &str) -> Result<Vec<budgets::Model>, AppError> {
        Ok(budgets::Entity::find()
            .filter(budgets::Column::UserId.eq(user_id))
            .order_by_desc(budgets::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?)
    }

    pub async fn update(
        &self,
        user_id: &str,
        budget_id: &str,
        amount: Option<Decimal>,
        period: Option<BudgetPeriod>,
    ) -> Result<budgets::Model, AppError> {
        let mut budget: budgets::ActiveModel = budgets::Entity::find()
            .filter(budgets::Column::Id.eq(budget_id))
            .filter(budgets::Column::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| AppError::not_found("Budget not found"))?
            .into();

        if let Some(amount) = amount {
            budget.amount = ActiveValue::Set(amount);
        }
        if let Some(period) = period {
            budget.period = ActiveValue::Set(period);
        }
        budget.updated_at = ActiveValue::Set(Utc::now().naive_utc());

        Ok(budget.update(self.db.as_ref()).await?)
    }

    pub async fn delete(&self, user_id: &str, budget_id: &str) -> Result<u64, AppError> {
        let res = budgets::Entity::delete_many()
            .filter(budgets::Column::Id.eq(budget_id))
            .filter(budgets::Column::UserId.eq(user_id))
            .exec(self.db.as_ref())
            .await?;
        Ok(res.rows_affected)
    }

    pub async fn get_all_budget_health(
        &self,
        user_id: &str,
    ) -> Result<Vec<BudgetHealth>, AppError> {
        let budgets = self.list(user_id).await?;
        let mut health_results = Vec::new();

        for budget in budgets {
            let (start_date, end_date) = get_period_bounds(budget.period);

            // Sum transactions in this period for this category
            let mut query = transactions::Entity::find()
                .filter(transactions::Column::UserId.eq(user_id))
                .filter(transactions::Column::Direction.eq("OUT"))
                .filter(transactions::Column::Date.gte(start_date))
                .filter(transactions::Column::Date.lt(end_date))
                .filter(transactions::Column::DeletedAt.is_null());

            #[derive(sea_orm::FromQueryResult)]
            struct SumResult {
                total: Option<Decimal>,
            }

            if let Some(ref cat_id) = budget.category_id {
                query = query.filter(transactions::Column::CategoryId.eq(cat_id));
            }

            let res: Option<SumResult> = query
                .select_only()
                .column_as(transactions::Column::Amount.sum(), "total")
                .into_model::<SumResult>()
                .one(self.db.as_ref())
                .await?;

            let spent = res.and_then(|r| r.total).unwrap_or(Decimal::ZERO);
            let remaining = budget.amount - spent;
            let percentage = if budget.amount.is_zero() {
                Decimal::ZERO
            } else {
                (spent / budget.amount) * Decimal::from(100)
            };

            // Get category name if it exists
            let category_name = if let Some(ref cat_id) = budget.category_id {
                categories::Entity::find_by_id(cat_id)
                    .one(self.db.as_ref())
                    .await?
                    .map(|c| c.name)
            } else {
                Some("All Categories".to_string())
            };

            health_results.push(BudgetHealth {
                budget_id: budget.id,
                category_name,
                limit_amount: budget.amount,
                spent_amount: spent,
                remaining_amount: remaining,
                percentage_consumed: percentage,
                period: budget.period,
            });
        }

        Ok(health_results)
    }
}

#[cfg(test)]
mod tests;

fn get_period_bounds(period: BudgetPeriod) -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
    let now = Utc::now();
    match period {
        BudgetPeriod::Weekly => {
            // Monday of this week
            let weekday = now.weekday().num_days_from_monday();
            let start = Utc
                .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
                .single()
                .unwrap_or(now)
                - chrono::Duration::days(i64::from(weekday));
            let end = start + chrono::Duration::days(7);
            (start, end)
        }
        BudgetPeriod::Monthly => {
            let start = Utc
                .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
                .single()
                .unwrap_or(now);
            let (next_year, next_month) = if now.month() == 12 {
                (now.year() + 1, 1)
            } else {
                (now.year(), now.month() + 1)
            };
            let end = Utc
                .with_ymd_and_hms(next_year, next_month, 1, 0, 0, 0)
                .single()
                .unwrap_or(now);
            (start, end)
        }
        BudgetPeriod::Yearly => {
            let start = Utc
                .with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0)
                .single()
                .unwrap_or(now);
            let end = Utc
                .with_ymd_and_hms(now.year() + 1, 1, 1, 0, 0, 0)
                .single()
                .unwrap_or(now);
            (start, end)
        }
    }
}
