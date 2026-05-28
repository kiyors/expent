use chrono::{Datelike, Duration, TimeZone, Utc};
use db::entities;
use db::entities::enums::TransactionDirection;
use db::{AppError, DashboardSummary, MonthlyTrend, NamedAmount};
use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, FromQueryResult,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};

/// Retrieves a comprehensive summary for the user's dashboard.
///
/// # Errors
/// Returns `AppError::Db` if any database query fails.
/// Returns `AppError::NotFound` if the user is not found.
///
/// # Panics
/// Panics if the start-of-month timestamp (year/month/1 00:00) cannot be constructed,
/// which is unreachable for any valid current UTC instant.
// Aggregates many independent summary queries in parallel; splitting would
// fragment the `tokio::try_join!` and hurt readability.
#[allow(clippy::too_many_lines)]
pub async fn get_dashboard_summary(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<DashboardSummary, AppError> {
    #[derive(FromQueryResult)]
    struct SumResult {
        total: Option<Decimal>,
    }

    #[derive(FromQueryResult)]
    struct TotalResult {
        total: Option<Decimal>,
    }

    // 0. Get user email for P2P requests
    let user = entities::users::Entity::find_by_id(user_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    let now = Utc::now();
    let start_of_month = Utc
        .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
        .latest()
        .expect("Valid start of month");

    // Parallelize independent queries
    let (
        balance_res,
        monthly_spend_res,
        monthly_income_res,
        pending_p2p_count,
        total_transactions,
        monthly_trends,
        weekly_trends,
        category_distribution,
        top_expenses,
        top_income,
    ) = tokio::try_join!(
        // 1. Total Balance
        async move {
            entities::wallets::Entity::find()
                .filter(entities::wallets::Column::UserId.eq(user_id))
                .select_only()
                .column_as(entities::wallets::Column::Balance.sum(), "total")
                .into_model::<TotalResult>()
                .one(db)
                .await
                .map_err(AppError::from)
        },
        // 2a. Monthly Spend
        async move {
            entities::transactions::Entity::find()
                .filter(entities::transactions::Column::UserId.eq(user_id))
                .filter(entities::transactions::Column::Direction.eq("OUT"))
                .filter(entities::transactions::Column::Date.gte(start_of_month))
                .filter(entities::transactions::Column::DeletedAt.is_null())
                .select_only()
                .column_as(entities::transactions::Column::Amount.sum(), "total")
                .into_model::<SumResult>()
                .one(db)
                .await
                .map_err(AppError::from)
        },
        // 2b. Monthly Income
        async move {
            entities::transactions::Entity::find()
                .filter(entities::transactions::Column::UserId.eq(user_id))
                .filter(entities::transactions::Column::Direction.eq("IN"))
                .filter(entities::transactions::Column::Date.gte(start_of_month))
                .filter(entities::transactions::Column::DeletedAt.is_null())
                .select_only()
                .column_as(entities::transactions::Column::Amount.sum(), "total")
                .into_model::<SumResult>()
                .one(db)
                .await
                .map_err(AppError::from)
        },
        // 3. Pending P2P count
        async move {
            entities::p2p_requests::Entity::find()
                .filter(entities::p2p_requests::Column::ReceiverEmail.eq(user.email))
                .filter(entities::p2p_requests::Column::Status.is_in(["PENDING", "GROUP_INVITE"]))
                .count(db)
                .await
                .map_err(AppError::from)
        },
        // 3b. Total Transactions
        async move {
            entities::transactions::Entity::find()
                .filter(entities::transactions::Column::UserId.eq(user_id))
                .filter(entities::transactions::Column::DeletedAt.is_null())
                .count(db)
                .await
                .map_err(AppError::from)
        },
        // 4. Trends
        get_monthly_trends(db, user_id),
        get_weekly_trends(db, user_id),
        // 5. Category Distribution
        get_category_distribution(db, user_id),
        // 6. Top Sources
        get_top_sources(db, user_id, "OUT"),
        get_top_sources(db, user_id, "IN"),
    )?;

    let total_balance = balance_res.and_then(|r| r.total).unwrap_or(Decimal::ZERO);
    let monthly_spend = monthly_spend_res
        .and_then(|r| r.total)
        .unwrap_or(Decimal::ZERO);
    let monthly_income = monthly_income_res
        .and_then(|r| r.total)
        .unwrap_or(Decimal::ZERO);

    Ok(DashboardSummary {
        total_balance,
        monthly_spend,
        monthly_income,
        pending_p2p_count,
        total_transactions,
        monthly_trends,
        weekly_trends,
        category_distribution,
        top_expenses,
        top_income,
    })
}

async fn get_category_distribution(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<NamedAmount>, AppError> {
    #[derive(FromQueryResult)]
    struct CatDist {
        category_name: String,
        amount: Decimal,
    }

    let results = entities::transactions::Entity::find()
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
        .await?;

    Ok(results
        .into_iter()
        .map(|c| NamedAmount {
            name: c.category_name,
            amount: c.amount,
        })
        .collect())
}

async fn get_top_sources(
    db: &DatabaseConnection,
    user_id: &str,
    direction: &str,
) -> Result<Vec<NamedAmount>, AppError> {
    #[derive(FromQueryResult)]
    struct TopSourceWithContact {
        contact_name: String,
        amount: Decimal,
    }

    let results = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Direction.eq(direction))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .join(
            sea_orm::JoinType::InnerJoin,
            entities::transactions::Relation::TxnParties.def(),
        )
        .filter(entities::txn_parties::Column::Role.eq("COUNTERPARTY"))
        .filter(entities::txn_parties::Column::ContactId.is_not_null())
        .join(
            sea_orm::JoinType::InnerJoin,
            entities::txn_parties::Relation::Contacts.def(),
        )
        .select_only()
        .column_as(entities::contacts::Column::Name, "contact_name")
        .column_as(entities::transactions::Column::Amount.sum(), "amount")
        .group_by(entities::contacts::Column::Name)
        .order_by_desc(entities::transactions::Column::Amount.sum())
        .limit(5)
        .into_model::<TopSourceWithContact>()
        .all(db)
        .await?;

    Ok(results
        .into_iter()
        .map(|r| NamedAmount {
            name: r.contact_name,
            amount: r.amount,
        })
        .collect())
}

async fn get_monthly_trends(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<MonthlyTrend>, AppError> {
    #[derive(FromQueryResult)]
    struct TrendResult {
        date_key: String,
        direction: TransactionDirection,
        total_amount: Decimal,
    }

    let now = Utc::now();
    let six_months_ago = now - Duration::days(180);

    let backend = db.get_database_backend();
    let date_expr = match backend {
        DbBackend::Postgres => "to_char(date, 'YYYY-MM')",
        _ => "strftime('%Y-%m', date)",
    };

    let trends = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Date.gte(six_months_ago))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .select_only()
        .column_as(sea_orm::sea_query::Expr::cust(date_expr), "date_key")
        .column(entities::transactions::Column::Direction)
        .column_as(entities::transactions::Column::Amount.sum(), "total_amount")
        .group_by(sea_orm::sea_query::Expr::cust(date_expr))
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
        let month_num = key
            .split('-')
            .nth(1)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
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
    #[derive(FromQueryResult)]
    struct TrendResult {
        date_key: String,
        direction: TransactionDirection,
        total_amount: Decimal,
    }

    let now = Utc::now();
    let seven_days_ago = now - Duration::days(7);

    let backend = db.get_database_backend();
    let date_expr = match backend {
        DbBackend::Postgres => "to_char(date, 'YYYY-MM-DD')",
        _ => "strftime('%Y-%m-%d', date)",
    };

    let trends = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Date.gte(seven_days_ago))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .select_only()
        .column_as(sea_orm::sea_query::Expr::cust(date_expr), "date_key")
        .column(entities::transactions::Column::Direction)
        .column_as(entities::transactions::Column::Amount.sum(), "total_amount")
        .group_by(sea_orm::sea_query::Expr::cust(date_expr))
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
        let parts: Vec<&str> = key.split('-').collect();
        let y = parts
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        let m = parts
            .get(1)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let d = parts
            .get(2)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let date = Utc
            .with_ymd_and_hms(y, m, d, 0, 0, 0)
            .single()
            .unwrap_or_else(Utc::now);
        result.push(MonthlyTrend {
            month: date.format("%a").to_string(),
            income: inc,
            expense: exp,
        });
    }

    Ok(result)
}
