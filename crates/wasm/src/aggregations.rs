use crate::text::normalize_text;
use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use ts_rs::TS;
use wasm_bindgen::prelude::*;

/// Trend data for a single month.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct MonthlyTrend {
    pub month: String,
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub income: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub expense: Decimal,
}

/// Distribution of expenses by category or contact.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct NamedAmount {
    pub name: String,
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub amount: Decimal,
}

/// Summary data for the dashboard.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct DashboardSummary {
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub total_balance: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub monthly_spend: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub monthly_income: Decimal,
    pub pending_p2p_count: u64,
    pub total_transactions: u64,
    pub monthly_trends: Vec<MonthlyTrend>,
    pub weekly_trends: Vec<MonthlyTrend>, // Reuse MonthlyTrend for weekly too
    pub category_distribution: Vec<NamedAmount>,
    pub top_expenses: Vec<NamedAmount>,
    pub top_income: Vec<NamedAmount>,
}

#[derive(serde::Deserialize, TS)]
#[ts(export)]
pub struct FullTxn {
    pub amount: String,
    pub direction: String,
    pub date: String,
    pub status: Option<String>,
    pub category_id: Option<String>,
    pub contact_name: Option<String>,
    pub purpose_tag: Option<String>,
}

#[derive(serde::Deserialize, TS)]
#[ts(export)]
pub struct WalletMinimal {
    pub balance: String,
}

#[derive(serde::Deserialize, TS)]
#[ts(export)]
pub struct CategoryMinimal {
    pub id: String,
    pub name: String,
}

#[wasm_bindgen]
pub fn generate_dashboard_summary(
    transactions: JsValue,
    wallets: JsValue,
    categories: JsValue,
) -> Result<JsValue, JsError> {
    let txns: Vec<FullTxn> = serde_wasm_bindgen::from_value(transactions)?;
    let wallets: Vec<WalletMinimal> = serde_wasm_bindgen::from_value(wallets)?;
    let categories: Vec<CategoryMinimal> = serde_wasm_bindgen::from_value(categories)?;

    let cat_map: HashMap<String, String> = categories.into_iter().map(|c| (c.id, c.name)).collect();

    let now = Utc::now();
    let start_of_month = Utc
        .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
        .latest()
        .unwrap_or(now);

    let mut total_balance = Decimal::ZERO;
    for w in wallets {
        total_balance += Decimal::from_str(&w.balance).unwrap_or(Decimal::ZERO);
    }

    let mut monthly_spend = Decimal::ZERO;
    let mut monthly_income = Decimal::ZERO;
    let mut total_transactions = 0;

    let mut monthly_trends_map = BTreeMap::new();
    for i in (0..6).rev() {
        let date = now - Duration::days(i64::from(i) * 30);
        let key = (date.year(), date.month());
        monthly_trends_map.insert(key, (Decimal::ZERO, Decimal::ZERO));
    }

    let mut weekly_trends_map = BTreeMap::new();
    for i in (0..7).rev() {
        let date = now - Duration::days(i64::from(i));
        let key = (date.year(), date.month(), date.day());
        weekly_trends_map.insert(key, (Decimal::ZERO, Decimal::ZERO));
    }

    let mut cat_dist_map: HashMap<String, Decimal> = HashMap::new();
    let mut contact_exp_map: HashMap<String, Decimal> = HashMap::new();
    let mut contact_inc_map: HashMap<String, Decimal> = HashMap::new();

    for tx in &txns {
        if tx.status.as_deref() == Some("CANCELLED") {
            continue;
        }
        total_transactions += 1;
        let amt = Decimal::from_str(&tx.amount).unwrap_or(Decimal::ZERO);
        let tx_date = DateTime::parse_from_rfc3339(&tx.date)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or(now);

        if tx_date >= start_of_month {
            if tx.direction == "IN" {
                monthly_income += amt;
            } else {
                monthly_spend += amt;
            }
        }

        // Monthly trends
        let m_key = (tx_date.year(), tx_date.month());
        if let Some(entry) = monthly_trends_map.get_mut(&m_key) {
            if tx.direction == "IN" {
                entry.0 += amt;
            } else {
                entry.1 += amt;
            }
        }

        // Weekly trends (last 7 days)
        let w_key = (tx_date.year(), tx_date.month(), tx_date.day());
        if let Some(entry) = weekly_trends_map.get_mut(&w_key) {
            if tx.direction == "IN" {
                entry.0 += amt;
            } else {
                entry.1 += amt;
            }
        }

        if tx.direction == "OUT" {
            // Category distribution
            if let Some(cat_id) = &tx.category_id {
                if let Some(cat_name) = cat_map.get(cat_id) {
                    *cat_dist_map.entry(cat_name.clone()).or_default() += amt;
                }
            }
            // Top expenses
            if let Some(name) = &tx.contact_name {
                *contact_exp_map.entry(name.clone()).or_default() += amt;
            }
        } else if tx.direction == "IN" {
            // Top income
            if let Some(name) = &tx.contact_name {
                *contact_inc_map.entry(name.clone()).or_default() += amt;
            }
        }
    }

    let monthly_trends = monthly_trends_map
        .into_iter()
        .map(|(key, (inc, exp))| {
            let (_, month_num) = key;
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
            MonthlyTrend {
                month: month_name.to_string(),
                income: inc,
                expense: exp,
            }
        })
        .collect();

    let weekly_trends = weekly_trends_map
        .into_iter()
        .map(|(key, (inc, exp))| {
            let (y, m, d) = key;
            let date = Utc
                .with_ymd_and_hms(y, m, d, 0, 0, 0)
                .single()
                .unwrap_or(now);
            MonthlyTrend {
                month: date.format("%a").to_string(),
                income: inc,
                expense: exp,
            }
        })
        .collect();

    let mut category_distribution: Vec<NamedAmount> = cat_dist_map
        .into_iter()
        .map(|(name, amount)| NamedAmount { name, amount })
        .collect();
    category_distribution.sort_by(|a, b| b.amount.cmp(&a.amount));

    let mut top_expenses: Vec<NamedAmount> = contact_exp_map
        .into_iter()
        .map(|(name, amount)| NamedAmount { name, amount })
        .collect();
    top_expenses.sort_by(|a, b| b.amount.cmp(&a.amount));
    top_expenses.truncate(5);

    let mut top_income: Vec<NamedAmount> = contact_inc_map
        .into_iter()
        .map(|(name, amount)| NamedAmount { name, amount })
        .collect();
    top_income.sort_by(|a, b| b.amount.cmp(&a.amount));
    top_income.truncate(5);

    let result = DashboardSummary {
        total_balance,
        monthly_spend,
        monthly_income,
        pending_p2p_count: 0, // Cannot calculate pending P2P locally without all requests
        total_transactions: total_transactions as u64,
        monthly_trends,
        weekly_trends,
        category_distribution,
        top_expenses,
        top_income,
    };

    Ok(serde_wasm_bindgen::to_value(&result)?)
}

#[derive(serde::Deserialize, serde::Serialize, TS)]
#[ts(export)]
pub struct Txn {
    pub amount: String,
    pub direction: String,
    pub status: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export)]
pub struct AggregatedMetrics {
    pub total_income: String,
    pub total_expense: String,
    pub net_balance: String,
    pub count: usize,
}

#[wasm_bindgen]
pub fn aggregate_transactions(transactions: JsValue) -> Result<JsValue, JsError> {
    let txns: Vec<Txn> = serde_wasm_bindgen::from_value(transactions)?;
    let result = aggregate_transactions_internal(txns);
    Ok(serde_wasm_bindgen::to_value(&result)?)
}

pub fn aggregate_transactions_internal(txns: Vec<Txn>) -> AggregatedMetrics {
    let mut total_income = Decimal::ZERO;
    let mut total_expense = Decimal::ZERO;
    let mut count = 0;

    for tx in txns {
        if tx.status.as_deref() == Some("CANCELLED") {
            continue;
        }
        let amt = Decimal::from_str(&tx.amount).unwrap_or(Decimal::ZERO);
        if tx.direction == "IN" {
            total_income += amt;
        } else {
            total_expense += amt;
        }
        count += 1;
    }

    AggregatedMetrics {
        total_income: total_income.to_string(),
        total_expense: total_expense.to_string(),
        net_balance: (total_income - total_expense).to_string(),
        count,
    }
}

#[derive(serde::Deserialize, serde::Serialize, TS)]
#[ts(export)]
pub struct TxnPattern {
    pub amount: String,
    pub date: String,
    pub purpose_tag: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export)]
pub struct DetectedSubscription {
    pub name: String,
    pub amount: String,
    pub cycle: String,
    pub last_date: String,
    pub count: usize,
}

#[wasm_bindgen]
pub fn detect_subscription_patterns(transactions: JsValue) -> Result<JsValue, JsError> {
    let txns: Vec<TxnPattern> = serde_wasm_bindgen::from_value(transactions)?;
    let suspected = detect_subscription_patterns_internal(txns);
    Ok(serde_wasm_bindgen::to_value(&suspected)?)
}

pub fn detect_subscription_patterns_internal(
    mut txns: Vec<TxnPattern>,
) -> Vec<DetectedSubscription> {
    // Sort by date
    txns.sort_by_key(|t| t.date.clone());

    let mut patterns = HashMap::new();

    // Group by normalized purpose + amount
    for tx in &txns {
        let key = format!(
            "{}:{}",
            normalize_text(tx.purpose_tag.as_deref().unwrap_or("unknown")),
            tx.amount
        );
        patterns.entry(key).or_insert_with(Vec::new).push(tx);
    }

    let mut suspected = Vec::new();

    for (key, group) in patterns {
        if group.len() < 2 {
            continue;
        }

        let mut intervals = Vec::new();
        for i in 0..group.len() - 1 {
            let d1 = chrono::DateTime::parse_from_rfc3339(&group[i].date).ok();
            let d2 = chrono::DateTime::parse_from_rfc3339(&group[i + 1].date).ok();
            if let (Some(d1), Some(d2)) = (d1, d2) {
                let diff = (d2 - d1).num_days();
                intervals.push(diff);
            }
        }

        if intervals.is_empty() {
            continue;
        }

        // Heuristic: if intervals are mostly around 30 days (+- 3) or 7 days (+- 1)
        let is_monthly = intervals.iter().all(|&d| (27..=33).contains(&d));
        let is_weekly = intervals.iter().all(|&d| (6..=8).contains(&d));

        if is_monthly || is_weekly {
            suspected.push(DetectedSubscription {
                name: group[0].purpose_tag.clone().unwrap_or_else(|| key.clone()),
                amount: group[0].amount.clone(),
                cycle: (if is_monthly { "MONTHLY" } else { "WEEKLY" }).to_string(),
                last_date: group.last().unwrap().date.clone(),
                count: group.len(),
            });
        }
    }
    suspected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation() {
        let txns = vec![
            Txn {
                amount: "100".to_string(),
                direction: "IN".to_string(),
                status: None,
            },
            Txn {
                amount: "50".to_string(),
                direction: "OUT".to_string(),
                status: None,
            },
            Txn {
                amount: "1000".to_string(),
                direction: "OUT".to_string(),
                status: Some("CANCELLED".to_string()),
            },
        ];
        let res = aggregate_transactions_internal(txns);

        assert_eq!(res.total_income, "100");
        assert_eq!(res.total_expense, "50");
        assert_eq!(res.net_balance, "50");
        assert_eq!(res.count, 2);
    }

    #[test]
    fn test_subscription_detection() {
        let txns = vec![
            TxnPattern {
                amount: "500".to_string(),
                date: "2023-01-01T10:00:00Z".to_string(),
                purpose_tag: Some("Netflix".to_string()),
            },
            TxnPattern {
                amount: "500".to_string(),
                date: "2023-02-01T10:00:00Z".to_string(),
                purpose_tag: Some("Netflix".to_string()),
            },
            TxnPattern {
                amount: "500".to_string(),
                date: "2023-03-01T10:00:00Z".to_string(),
                purpose_tag: Some("Netflix".to_string()),
            },
        ];
        let res = detect_subscription_patterns_internal(txns);

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "Netflix");
        assert_eq!(res[0].cycle, "MONTHLY");
    }
}
