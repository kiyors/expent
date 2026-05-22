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

    let mut results = Vec::new();

    for row in &rows {
        let mut best_match: Option<BatchMatchResult> = None;

        let row_amount = row
            .debit
            .as_ref()
            .or(row.credit.as_ref())
            .and_then(|a| Decimal::from_str(a).ok())
            .unwrap_or(Decimal::ZERO)
            .abs();

        let row_date_ms = chrono::DateTime::parse_from_rfc3339(&row.date)
            .map(|d| d.timestamp_millis())
            .unwrap_or(0);

        for txn in &txns {
            let txn_date_ms = chrono::DateTime::parse_from_rfc3339(&txn.date)
                .map(|d| d.timestamp_millis())
                .unwrap_or(0);

            let txn_desc = txn
                .purpose_tag
                .as_deref()
                .unwrap_or_else(|| txn.notes.as_deref().unwrap_or(""));

            let score = calculate_match_score_internal(
                row_date_ms,
                &row.description,
                row_amount,
                txn_date_ms,
                txn_desc,
                Decimal::from_str(&txn.amount)
                    .unwrap_or(Decimal::ZERO)
                    .abs(),
            );

            if score >= 50 {
                if best_match.as_ref().map_or(true, |m| score > m.confidence) {
                    best_match = Some(BatchMatchResult {
                        row_id: row.id.clone(),
                        transaction_id: txn.id.clone(),
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

    calculate_match_score_internal(
        row_date_ms,
        row_desc,
        row_amt,
        txn_date_ms,
        txn_desc,
        txn_amt,
    )
}

pub fn calculate_match_score_internal(
    row_date_ms: i64,
    row_desc: &str,
    row_amt: Decimal,
    txn_date_ms: i64,
    txn_desc: &str,
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

    // Description fuzzy match
    let desc_score = fuzzy_score(&normalize_text(row_desc), &normalize_text(txn_desc));
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
