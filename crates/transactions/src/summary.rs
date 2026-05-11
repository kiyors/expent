use chrono::{Datelike, Duration, TimeZone, Utc};
use db::entities;
use db::entities::enums::TransactionDirection;
use db::{AppError, DashboardSummary, MonthlyTrend, NamedAmount};
use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait,
};

pub async fn get_dashboard_summary(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<DashboardSummary, AppError> {
    // 0. Get user email for P2P requests
    let user = entities::users::Entity::find_by_id(user_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    // 1. Total Balance (sum of all wallet balances)
    #[derive(FromQueryResult)]
    struct TotalResult {
        total: Option<Decimal>,
    }

    let balance_res: Option<TotalResult> = entities::wallets::Entity::find()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .select_only()
        .column_as(entities::wallets::Column::Balance.sum(), "total")
        .into_model::<TotalResult>()
        .one(db)
        .await?;

    let total_balance = balance_res.and_then(|r| r.total).unwrap_or(Decimal::ZERO);

    // 2. Monthly Spend & Income
    let now = Utc::now();
    let start_of_month = Utc
        .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
        .latest()
        .expect("Valid start of month");

    #[derive(FromQueryResult)]
    struct SumResult {
        total: Option<Decimal>,
    }

    let monthly_spend: Option<Decimal> = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Direction.eq("OUT"))
        .filter(entities::transactions::Column::Date.gte(start_of_month))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .select_only()
        .column_as(entities::transactions::Column::Amount.sum(), "total")
        .into_model::<SumResult>()
        .one(db)
        .await?
        .and_then(|r| r.total);

    let monthly_income: Option<Decimal> = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Direction.eq("IN"))
        .filter(entities::transactions::Column::Date.gte(start_of_month))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .select_only()
        .column_as(entities::transactions::Column::Amount.sum(), "total")
        .into_model::<SumResult>()
        .one(db)
        .await?
        .and_then(|r| r.total);

    // 3. Pending P2P count
    let pending_p2p_count = entities::p2p_requests::Entity::find()
        .filter(entities::p2p_requests::Column::ReceiverEmail.eq(user.email))
        .filter(entities::p2p_requests::Column::Status.is_in(["PENDING", "GROUP_INVITE"]))
        .count(db)
        .await?;

    // 3b. Total Transactions
    let total_transactions = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .count(db)
        .await?;

    // 4. Trends (Optimized Group-By queries)
    let monthly_trends = get_monthly_trends(db, user_id).await?;
    let weekly_trends = get_weekly_trends(db, user_id).await?;

    // 5. Category Distribution
    #[derive(FromQueryResult)]
    struct CatDist {
        category_name: String,
        amount: Decimal,
    }

    let category_distribution = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Direction.eq("OUT"))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .filter(entities::transactions::Column::CategoryId.is_not_null())
        .join(
            sea_orm::JoinType::InnerJoin,
            entities::transactions::Relation::Categories.def(),
        )
        .select_only()
        .column_as(entities::categories::Column::Name, "category_name")
        .column_as(entities::transactions::Column::Amount.sum(), "amount")
        .group_by(entities::categories::Column::Name)
        .into_model::<CatDist>()
        .all(db)
        .await?
        .into_iter()
        .map(|c| NamedAmount {
            name: c.category_name,
            amount: c.amount,
        })
        .collect();

    // 6. Top Sources (by contact)
    #[derive(FromQueryResult)]
    struct TopSource {
        contact_id: String,
        amount: Decimal,
    }

    async fn get_top_sources(
        db: &DatabaseConnection,
        user_id: &str,
        direction: &str,
    ) -> Result<Vec<NamedAmount>, AppError> {
        let results: Vec<TopSource> = entities::transactions::Entity::find()
            .filter(entities::transactions::Column::UserId.eq(user_id))
            .filter(entities::transactions::Column::Direction.eq(direction))
            .filter(entities::transactions::Column::DeletedAt.is_null())
            .join(
                sea_orm::JoinType::InnerJoin,
                entities::transactions::Relation::TxnParties.def(),
            )
            .filter(entities::txn_parties::Column::Role.eq("COUNTERPARTY"))
            .filter(entities::txn_parties::Column::ContactId.is_not_null())
            .select_only()
            .column(entities::txn_parties::Column::ContactId)
            .column_as(entities::transactions::Column::Amount.sum(), "amount")
            .group_by(entities::txn_parties::Column::ContactId)
            .order_by_desc(entities::transactions::Column::Amount.sum())
            .limit(5)
            .into_model::<TopSource>()
            .all(db)
            .await?;

        if results.is_empty() {
            return Ok(Vec::new());
        }

        let contact_ids: Vec<String> = results.iter().map(|r| r.contact_id.clone()).collect();
        let contacts = entities::contacts::Entity::find()
            .filter(entities::contacts::Column::Id.is_in(contact_ids))
            .all(db)
            .await?;

        let contact_map: std::collections::HashMap<String, String> =
            contacts.into_iter().map(|c| (c.id, c.name)).collect();

        Ok(results
            .into_iter()
            .map(|r| NamedAmount {
                name: contact_map
                    .get(&r.contact_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
                amount: r.amount,
            })
            .collect())
    }

    let top_expenses = get_top_sources(db, user_id, "OUT").await?;
    let top_income = get_top_sources(db, user_id, "IN").await?;

    Ok(DashboardSummary {
        total_balance,
        monthly_spend: monthly_spend.unwrap_or(Decimal::ZERO),
        monthly_income: monthly_income.unwrap_or(Decimal::ZERO),
        pending_p2p_count: pending_p2p_count,
        total_transactions: total_transactions,
        monthly_trends,
        weekly_trends,
        category_distribution,
        top_expenses,
        top_income,
    })
}

async fn get_monthly_trends(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<MonthlyTrend>, AppError> {
    let now = Utc::now();
    let six_months_ago = now - Duration::days(180);

    #[derive(FromQueryResult)]
    struct TrendResult {
        date_key: String,
        direction: TransactionDirection,
        total_amount: Decimal,
    }

    let trends = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Date.gte(six_months_ago))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .select_only()
        .column_as(
            sea_orm::sea_query::Expr::cust("strftime('%Y-%m', date)"),
            "date_key",
        )
        .column(entities::transactions::Column::Direction)
        .column_as(entities::transactions::Column::Amount.sum(), "total_amount")
        .group_by(sea_orm::sea_query::Expr::cust("strftime('%Y-%m', date)"))
        .group_by(entities::transactions::Column::Direction)
        .into_model::<TrendResult>()
        .all(db)
        .await?;

    let mut trends_map = std::collections::BTreeMap::new();

    // Initialize the last 6 months with zeros
    for i in (0..6).rev() {
        let date = now - Duration::days(i64::from(i) * 30);
        let key = format!("{}-{:02}", date.year(), date.month());
        trends_map.insert(key, (Decimal::ZERO, Decimal::ZERO));
    }

    for t in trends {
        if let Some(entry) = trends_map.get_mut(&t.date_key) {
            match t.direction {
                TransactionDirection::In => entry.0 += t.total_amount,
                TransactionDirection::Out => entry.1 += t.total_amount,
            }
        }
    }

    let mut result = Vec::new();
    for (key, (inc, exp)) in trends_map {
        let month_num = key.split('-').nth(1).unwrap().parse::<u32>().unwrap();
        let month_name = match month_num {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "???",
        };
        result.push(MonthlyTrend {
            month: month_name.to_string(),
            income: inc,
            expense: exp,
        });
    }

    Ok(result)
}

async fn get_weekly_trends(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<MonthlyTrend>, AppError> {
    let now = Utc::now();
    let seven_days_ago = now - Duration::days(7);

    #[derive(FromQueryResult)]
    struct TrendResult {
        date_key: String,
        direction: TransactionDirection,
        total_amount: Decimal,
    }

    let trends = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Date.gte(seven_days_ago))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .select_only()
        .column_as(
            sea_orm::sea_query::Expr::cust("strftime('%Y-%m-%d', date)"),
            "date_key",
        )
        .column(entities::transactions::Column::Direction)
        .column_as(entities::transactions::Column::Amount.sum(), "total_amount")
        .group_by(sea_orm::sea_query::Expr::cust("strftime('%Y-%m-%d', date)"))
        .group_by(entities::transactions::Column::Direction)
        .into_model::<TrendResult>()
        .all(db)
        .await?;

    let mut trends_map = std::collections::BTreeMap::new();

    // Initialize last 7 days
    for i in (0..7).rev() {
        let date = now - Duration::days(i64::from(i));
        let key = format!("{}-{:02}-{:02}", date.year(), date.month(), date.day());
        trends_map.insert(key, (Decimal::ZERO, Decimal::ZERO));
    }

    for t in trends {
        if let Some(entry) = trends_map.get_mut(&t.date_key) {
            match t.direction {
                TransactionDirection::In => entry.0 += t.total_amount,
                TransactionDirection::Out => entry.1 += t.total_amount,
            }
        }
    }

    let mut result = Vec::new();
    for (key, (inc, exp)) in trends_map {
        let y = key.split('-').next().unwrap().parse::<i32>().unwrap();
        let m = key.split('-').nth(1).unwrap().parse::<u32>().unwrap();
        let d = key.split('-').nth(2).unwrap().parse::<u32>().unwrap();

        let date = Utc.with_ymd_and_hms(y, m, d, 0, 0, 0).unwrap();
        result.push(MonthlyTrend {
            month: date.format("%a").to_string(),
            income: inc,
            expense: exp,
        });
    }

    Ok(result)
}
