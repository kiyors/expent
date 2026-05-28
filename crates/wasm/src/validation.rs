use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use wasm_bindgen::prelude::*;

const MAX_NAME_LEN: usize = 255;
const MAX_PURPOSE_LEN: usize = 255;

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationError {
    fn new(field: &str, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Serialize a validation result back to JS. Falls back to `null` instead of
/// panicking on the (effectively impossible) serde failure path — better to
/// let JS see `null` than to abort the wasm instance with an unrecoverable
/// `RuntimeError`.
fn to_js(errors: Vec<ValidationError>) -> JsValue {
    let result = ValidationResult {
        is_valid: errors.is_empty(),
        errors,
    };
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Returns `Some(error)` if `amount` isn't a parseable positive Decimal.
fn check_amount_positive(amount: &str) -> Option<ValidationError> {
    match amount.parse::<Decimal>() {
        Ok(amt) if amt <= Decimal::ZERO => {
            Some(ValidationError::new("amount", "Amount must be positive"))
        }
        Err(_) => Some(ValidationError::new("amount", "Invalid amount format")),
        Ok(_) => None,
    }
}

/// Returns `Some(error)` if `value` is empty after trim or exceeds `max_len`.
fn check_required_string(
    field: &str,
    label: &str,
    value: &str,
    max_len: usize,
) -> Option<ValidationError> {
    if value.trim().is_empty() {
        Some(ValidationError::new(
            field,
            format!("{label} cannot be empty"),
        ))
    } else if value.len() > max_len {
        Some(ValidationError::new(field, format!("{label} is too long")))
    } else {
        None
    }
}

#[wasm_bindgen]
pub fn validate_transaction_wasm(amount: &str, purpose: &str) -> JsValue {
    let errors = [
        check_amount_positive(amount),
        check_required_string("purpose", "Purpose", purpose, MAX_PURPOSE_LEN),
    ]
    .into_iter()
    .flatten()
    .collect();
    to_js(errors)
}

#[wasm_bindgen]
pub fn validate_budget_wasm(amount: &str) -> JsValue {
    to_js(check_amount_positive(amount).into_iter().collect())
}

#[wasm_bindgen]
pub fn validate_wallet_wasm(name: &str, balance: &str) -> JsValue {
    let name_err = name
        .trim()
        .is_empty()
        .then(|| ValidationError::new("name", "Wallet name cannot be empty"));
    let balance_err = balance
        .parse::<Decimal>()
        .err()
        .map(|_| ValidationError::new("balance", "Invalid balance format"));
    let errors = [name_err, balance_err].into_iter().flatten().collect();
    to_js(errors)
}

#[wasm_bindgen]
pub fn validate_contact_wasm(name: &str) -> JsValue {
    to_js(
        check_required_string("name", "Contact name", name, MAX_NAME_LEN)
            .into_iter()
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn errors(name: &str, value: &str, max_len: usize) -> Vec<ValidationError> {
        check_required_string(name, "Field", value, max_len)
            .into_iter()
            .collect()
    }

    #[test]
    fn amount_must_be_positive() {
        assert!(check_amount_positive("10.50").is_none());
        assert_eq!(
            check_amount_positive("0")
                .as_ref()
                .map(|e| e.field.as_str()),
            Some("amount")
        );
        assert_eq!(
            check_amount_positive("-1")
                .as_ref()
                .map(|e| e.field.as_str()),
            Some("amount")
        );
        assert_eq!(
            check_amount_positive("abc")
                .as_ref()
                .map(|e| e.message.as_str()),
            Some("Invalid amount format")
        );
    }

    #[test]
    fn required_string_rejects_empty_and_long() {
        assert!(errors("name", "ok", 10).is_empty());
        assert_eq!(errors("name", "   ", 10).len(), 1);
        assert_eq!(errors("name", &"x".repeat(20), 10).len(), 1);
    }
}
