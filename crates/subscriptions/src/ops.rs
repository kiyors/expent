use chrono::{DateTime, Duration, FixedOffset, Utc};
use db::AppError;
use db::entities;
use db::entities::enums::{AlertChannel, SubscriptionCycle};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::str::FromStr;
use strsim::jaro_winkler;

pub async fn detect_subscriptions(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::subscriptions::Model>, AppError> {
    let ninety_days_ago = Utc::now() - Duration::days(90);
    let transactions = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Date.gte(ninety_days_ago))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .all(db)
        .await?;

    // Group transactions by "candidate" subscriptions
    let mut groups: Vec<(entities::transactions::Model, Vec<DateTime<FixedOffset>>)> = Vec::new();

    let ten_percent = Decimal::from_str("0.10").unwrap();

    for txn in transactions {
        let name = txn.purpose_tag.as_deref().unwrap_or("Unknown");
        let amount = txn.amount;

        let mut found_group = false;
        for (group_txn, dates) in &mut groups {
            let group_name = group_txn.purpose_tag.as_deref().unwrap_or("Unknown");
            let group_amount = group_txn.amount;

            // 1. Fuzzy Name Match (85% similarity)
            let name_match = jaro_winkler(name, group_name) > 0.85;

            // 2. Amount Epsilon (+/- 10%)
            let amount_diff = (amount - group_amount).abs();
            let amount_match = if group_amount > Decimal::ZERO {
                amount_diff / group_amount <= ten_percent
            } else {
                amount_diff == Decimal::ZERO
            };

            if name_match && amount_match {
                dates.push(txn.date);
                found_group = true;
                break;
            }
        }

        if !found_group {
            groups.push((txn.clone(), vec![txn.date]));
        }
    }

    let mut potential_subs = Vec::new();
    for (group_txn, mut dates) in groups {
        if dates.len() >= 2 {
            dates.sort();

            let mut detected_cycle = None;
            let last_date = *dates.last().unwrap();

            for i in 0..dates.len() - 1 {
                let diff = (dates[i + 1] - dates[i]).num_days();

                if (6..=8).contains(&diff) {
                    detected_cycle = Some(SubscriptionCycle::Weekly);
                } else if (27..=33).contains(&diff) {
                    detected_cycle = Some(SubscriptionCycle::Monthly);
                } else if (360..=370).contains(&diff) {
                    detected_cycle = Some(SubscriptionCycle::Yearly);
                }
            }

            if let Some(cycle) = detected_cycle {
                let next_charge = match cycle {
                    SubscriptionCycle::Weekly => last_date + Duration::days(7),
                    SubscriptionCycle::Yearly => last_date + Duration::days(365),
                    _ => last_date + Duration::days(30),
                };

                let sub = entities::subscriptions::Model {
                    id: uuid::Uuid::now_v7().to_string(),
                    user_id: user_id.to_string(),
                    name: group_txn
                        .purpose_tag
                        .unwrap_or_else(|| "Unknown".to_string()),
                    amount: group_txn.amount,
                    cycle,
                    start_date: dates[0],
                    next_charge_date: next_charge,
                    detection_keywords: None,
                };
                potential_subs.push(sub);
            }
        }
    }

    Ok(potential_subs)
}

pub async fn list_confirmed_subscriptions(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::subscriptions::Model>, AppError> {
    entities::subscriptions::Entity::find()
        .filter(entities::subscriptions::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(AppError::from)
}

pub struct ConfirmSubscriptionParams {
    pub user_id: String,
    pub name: String,
    pub amount: Decimal,
    pub cycle: SubscriptionCycle,
    pub start_date: DateTime<FixedOffset>,
    pub next_charge_date: DateTime<FixedOffset>,
    pub keywords: Option<serde_json::Value>,
}

pub async fn confirm_subscription(
    db: &DatabaseConnection,
    params: ConfirmSubscriptionParams,
) -> Result<entities::subscriptions::Model, AppError> {
    // Idempotency check: don't create duplicate subscriptions
    let existing = entities::subscriptions::Entity::find()
        .filter(entities::subscriptions::Column::UserId.eq(&params.user_id))
        .filter(entities::subscriptions::Column::Name.eq(params.name.clone()))
        .filter(entities::subscriptions::Column::Cycle.eq(params.cycle))
        .one(db)
        .await?;

    if let Some(sub) = existing {
        tracing::info!("⏭️ Subscription already confirmed: {}", params.name);
        return Ok(sub);
    }

    let sub = entities::subscriptions::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(params.user_id),
        name: Set(params.name),
        amount: Set(params.amount),
        cycle: Set(params.cycle),
        start_date: Set(params.start_date),
        next_charge_date: Set(params.next_charge_date),
        detection_keywords: Set(params.keywords),
    };
    sub.insert(db).await.map_err(AppError::from)
}

pub async fn stop_tracking_subscription(
    db: &DatabaseConnection,
    user_id: &str,
    sub_id: &str,
) -> Result<(), AppError> {
    entities::subscriptions::Entity::delete_many()
        .filter(entities::subscriptions::Column::Id.eq(sub_id))
        .filter(entities::subscriptions::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn configure_subscription_alert(
    db: &DatabaseConnection,
    sub_id: &str,
    days_before: i32,
    channel: AlertChannel,
) -> Result<entities::sub_alerts::Model, AppError> {
    let alert = entities::sub_alerts::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        subscription_id: Set(sub_id.to_string()),
        days_before: Set(days_before),
        channel: Set(channel),
        sent_at: Set(None),
    };
    alert.insert(db).await.map_err(AppError::from)
}
