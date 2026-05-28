use chrono::{Datelike, TimeZone, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use ts_rs::TS;
use wasm_bindgen::prelude::*;

pub trait DecimalExt {
    fn to_f64_low_precision(&self) -> f64;
}

impl DecimalExt for Decimal {
    fn to_f64_low_precision(&self) -> f64 {
        use rust_decimal::prelude::ToPrimitive;
        self.to_f64().unwrap_or(0.0)
    }
}

/// Compute `spent / limit * 100` as a Decimal-precise percentage string.
///
/// Returns `None` (becomes `undefined` in JS) when either input fails to parse,
/// surfacing bad caller data instead of silently reporting "0". A zero `limit`
/// is still treated as 0% so dividing-by-zero doesn't blow up.
#[wasm_bindgen]
pub fn calculate_budget_percentage(spent: String, limit: String) -> Option<String> {
    let spent = Decimal::from_str(&spent).ok()?;
    let limit = Decimal::from_str(&limit).ok()?;

    if limit.is_zero() {
        return Some("0".to_string());
    }

    let percentage = (spent / limit) * Decimal::from(100);
    Some(percentage.round_dp(2).to_string())
}

#[wasm_bindgen]
#[derive(TS)]
#[ts(export)]
pub struct PeriodBounds {
    pub start_ms: i64,
    pub end_ms: i64,
}

#[wasm_bindgen]
pub fn get_period_bounds(period: &str) -> Option<PeriodBounds> {
    let now = Utc::now();
    let (start, end) = match period.to_uppercase().as_str() {
        "WEEKLY" => {
            let weekday = now.weekday().num_days_from_monday();
            let start = Utc
                .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
                .single()
                .unwrap_or(now)
                - chrono::Duration::days(i64::from(weekday));
            let end = start + chrono::Duration::days(7);
            (start, end)
        }
        "MONTHLY" => {
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
        "YEARLY" => {
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
        _ => return None,
    };

    Some(PeriodBounds {
        start_ms: start.timestamp_millis(),
        end_ms: end.timestamp_millis(),
    })
}

#[wasm_bindgen]
pub fn is_transaction_in_period(txn_date_ms: i64, period: &str) -> bool {
    let bounds = match get_period_bounds(period) {
        Some(b) => b,
        None => return false,
    };
    txn_date_ms >= bounds.start_ms && txn_date_ms < bounds.end_ms
}

#[wasm_bindgen]
#[derive(TS)]
#[ts(export)]
pub struct SpendingVelocity {
    pub daily_burn_rate: f64,
    pub target_daily_rate: f64,
    pub is_overpacing: bool,
    pub projected_total: f64,
}

#[wasm_bindgen]
pub fn calculate_spending_velocity(
    spent: String,
    limit: String,
    period: &str,
) -> Option<SpendingVelocity> {
    let spent = Decimal::from_str(&spent).ok()?;
    let limit = Decimal::from_str(&limit).ok()?;
    let bounds = get_period_bounds(period)?;

    let now_ms = Utc::now().timestamp_millis();
    let total_duration_ms = (bounds.end_ms - bounds.start_ms).max(1);
    let elapsed_ms = (now_ms - bounds.start_ms).max(1).min(total_duration_ms);

    let elapsed_days = elapsed_ms as f64 / 86_400_000.0;
    let total_days = total_duration_ms as f64 / 86_400_000.0;

    let daily_burn_rate = if elapsed_days > 0.0 {
        spent.to_f64_low_precision() / elapsed_days
    } else {
        0.0
    };

    let target_daily_rate = limit.to_f64_low_precision() / total_days;
    let projected_total = daily_burn_rate * total_days;

    Some(SpendingVelocity {
        daily_burn_rate,
        target_daily_rate,
        is_overpacing: daily_burn_rate > target_daily_rate,
        projected_total,
    })
}

#[wasm_bindgen]
#[derive(TS)]
#[ts(export)]
pub struct SavingsProjection {
    pub monthly_contribution: f64,
    pub months_to_goal: i32,
    pub is_attainable: bool,
}

#[wasm_bindgen]
pub fn project_savings_goal(
    current_balance: String,
    target_amount: String,
    monthly_income: String,
    monthly_expenses: String,
) -> Option<SavingsProjection> {
    let balance = Decimal::from_str(&current_balance).ok()?;
    let target = Decimal::from_str(&target_amount).ok()?;
    let income = Decimal::from_str(&monthly_income).ok()?;
    let expenses = Decimal::from_str(&monthly_expenses).ok()?;

    let monthly_surplus = income - expenses;
    if monthly_surplus <= Decimal::ZERO {
        return Some(SavingsProjection {
            monthly_contribution: monthly_surplus.to_f64_low_precision(),
            months_to_goal: -1, // Impossible
            is_attainable: false,
        });
    }

    let needed = target - balance;
    if needed <= Decimal::ZERO {
        return Some(SavingsProjection {
            monthly_contribution: 0.0,
            months_to_goal: 0,
            is_attainable: true,
        });
    }

    let months = (needed / monthly_surplus).round_dp(0);
    let months_val = months.to_f64_low_precision() as i32;

    Some(SavingsProjection {
        monthly_contribution: monthly_surplus.to_f64_low_precision(),
        months_to_goal: months_val,
        is_attainable: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(
            calculate_budget_percentage("50".to_string(), "100".to_string()),
            Some("50.00".to_string())
        );
        assert_eq!(
            calculate_budget_percentage("0".to_string(), "100".to_string()),
            Some("0".to_string())
        );
        // Zero limit short-circuits to "0" rather than dividing by zero.
        assert_eq!(
            calculate_budget_percentage("50".to_string(), "0".to_string()),
            Some("0".to_string())
        );
        // Unparseable input surfaces as None instead of silently producing "0".
        assert_eq!(
            calculate_budget_percentage("abc".to_string(), "100".to_string()),
            None
        );
    }

    #[test]
    fn test_period_bounds_smoke() {
        let weekly = get_period_bounds("WEEKLY").unwrap();
        assert!(weekly.end_ms > weekly.start_ms);
    }

    #[test]
    fn test_savings_projection() {
        let proj = project_savings_goal(
            "1000".to_string(),
            "2000".to_string(),
            "5000".to_string(),
            "4000".to_string(),
        )
        .unwrap();
        assert_eq!(proj.months_to_goal, 1);
        assert!(proj.is_attainable);
    }
}
