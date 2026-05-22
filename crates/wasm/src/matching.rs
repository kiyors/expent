use crate::text::{fuzzy_score, normalize_text};
use rust_decimal::Decimal;
use std::str::FromStr;
use ts_rs::TS;
use wasm_bindgen::prelude::*;

#[derive(serde::Deserialize, TS)]
#[ts(export)]
pub struct StatementRowMinimal {
    pub id: String,
    pub date: String,
    pub description: String,
    pub debit: Option<String>,
    pub credit: Option<String>,
}

#[derive(serde::Deserialize, TS)]
#[ts(export)]
pub struct TransactionMinimal {
    pub id: String,
    pub date: String,
    pub amount: String,
    pub notes: Option<String>,
    pub purpose_tag: Option<String>,
}

#[derive(serde::Serialize, TS)]
#[ts(export)]
pub struct BatchMatchResult {
    pub row_id: String,
    pub transaction_id: String,
    pub confidence: i32,
}

#[wasm_bindgen]
pub fn match_statement_batch(
    statement_rows: JsValue,
    transactions: JsValue,
) -> Result<JsValue, JsError> {
    let rows: Vec<StatementRowMinimal> = serde_wasm_bindgen::from_value(statement_rows)?;
    let txns: Vec<TransactionMinimal> = serde_wasm_bindgen::from_value(transactions)?;

    // Pre-calculate data for efficiency
    let row_data: Vec<_> = rows
        .iter()
        .map(|row| {
            let amount = row
                .debit
                .as_ref()
                .or(row.credit.as_ref())
                .and_then(|a| Decimal::from_str(a).ok())
                .unwrap_or(Decimal::ZERO)
                .abs();
            let date_ms = chrono::DateTime::parse_from_rfc3339(&row.date)
                .map(|d| d.timestamp_millis())
                .unwrap_or(0);
            let norm_desc = normalize_text(&row.description);
            (row.id.clone(), amount, date_ms, norm_desc)
        })
        .collect();

    let txn_data: Vec<_> = txns
        .iter()
        .map(|txn| {
            let amount = Decimal::from_str(&txn.amount)
                .unwrap_or(Decimal::ZERO)
                .abs();
            let date_ms = chrono::DateTime::parse_from_rfc3339(&txn.date)
                .map(|d| d.timestamp_millis())
                .unwrap_or(0);
            let desc = txn
                .purpose_tag
                .as_deref()
                .unwrap_or_else(|| txn.notes.as_deref().unwrap_or(""));
            let norm_desc = normalize_text(desc);
            (txn.id.clone(), amount, date_ms, norm_desc)
        })
        .collect();

    let mut results = Vec::new();

    for (r_id, r_amount, r_date_ms, r_norm_desc) in &row_data {
        let mut best_match: Option<BatchMatchResult> = None;

        for (t_id, t_amount, t_date_ms, t_norm_desc) in &txn_data {
            let score = calculate_match_score_optimized(
                *r_date_ms,
                r_norm_desc,
                *r_amount,
                *t_date_ms,
                t_norm_desc,
                *t_amount,
            );

            if score >= 50 {
                if best_match.as_ref().map_or(true, |m| score > m.confidence) {
                    best_match = Some(BatchMatchResult {
                        row_id: r_id.clone(),
                        transaction_id: t_id.clone(),
                        confidence: score,
                    });
                }
            }
        }

        if let Some(m) = best_match {
            results.push(m);
        }
    }

    Ok(serde_wasm_bindgen::to_value(&results)?)
}

#[wasm_bindgen]
pub fn calculate_match_score(
    row_date_ms: i64,
    row_desc: &str,
    row_amount: String,
    txn_date_ms: i64,
    txn_desc: &str,
    txn_amount: String,
) -> i32 {
    let row_amt = Decimal::from_str(&row_amount)
        .unwrap_or(Decimal::ZERO)
        .abs();
    let txn_amt = Decimal::from_str(&txn_amount)
        .unwrap_or(Decimal::ZERO)
        .abs();

    let r_norm = normalize_text(row_desc);
    let t_norm = normalize_text(txn_desc);

    calculate_match_score_optimized(row_date_ms, &r_norm, row_amt, txn_date_ms, &t_norm, txn_amt)
}

pub fn calculate_match_score_optimized(
    row_date_ms: i64,
    row_norm_desc: &str,
    row_amt: Decimal,
    txn_date_ms: i64,
    txn_norm_desc: &str,
    txn_amt: Decimal,
) -> i32 {
    let mut score = 0;

    // Amount match (Absolute)
    if row_amt == txn_amt {
        score += 60;
    } else {
        // Proximity for amounts
        let diff = (row_amt - txn_amt).abs();
        if diff < Decimal::from(10) {
            score += 20;
        }
    }

    // Date proximity
    let date_diff_ms = (row_date_ms - txn_date_ms).abs();
    let three_days_ms = 3 * 24 * 60 * 60 * 1000;

    if date_diff_ms == 0 {
        score += 30;
    } else if date_diff_ms < three_days_ms {
        score += 15;
    }

    // Description fuzzy match (using already normalized strings)
    let desc_score = fuzzy_score(row_norm_desc, txn_norm_desc);
    score += (desc_score * 10.0) as i32;

    score.min(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_scoring() {
        let score = calculate_match_score(
            1000,
            "Starbucks Coffee",
            "500".to_string(),
            1000,
            "Starbucks",
            "500".to_string(),
        );
        assert!(score >= 90);
    }
}
