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

/// Internal: validate UPI ID format `<handle>@<provider>` per NPCI rules.
///
/// Handle: 3-256 chars of [a-zA-Z0-9._-]. Provider: 2-64 chars of [a-zA-Z0-9.].
/// Deliberately tighter than RFC 5321 emails — UPI handles are a controlled
/// namespace and rejecting odd characters early avoids spurious downstream
/// resolutions. Returns the first failure or None if valid.
fn check_upi_id(upi: &str) -> Option<ValidationError> {
    let trimmed = upi.trim();
    if trimmed.is_empty() {
        return Some(ValidationError::new("upi_id", "UPI ID cannot be empty"));
    }
    let Some((handle, provider)) = trimmed.split_once('@') else {
        return Some(ValidationError::new(
            "upi_id",
            "UPI ID must contain a single '@' (e.g. name@bank)",
        ));
    };
    if handle.contains('@') || provider.contains('@') {
        return Some(ValidationError::new(
            "upi_id",
            "UPI ID must contain exactly one '@'",
        ));
    }
    if !(3..=256).contains(&handle.len()) {
        return Some(ValidationError::new(
            "upi_id",
            "UPI handle must be between 3 and 256 characters",
        ));
    }
    if !(2..=64).contains(&provider.len()) {
        return Some(ValidationError::new(
            "upi_id",
            "UPI provider must be between 2 and 64 characters",
        ));
    }
    let handle_ok = handle
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    let provider_ok = provider
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.');
    if !handle_ok {
        return Some(ValidationError::new(
            "upi_id",
            "UPI handle may only contain letters, digits, '.', '_', '-'",
        ));
    }
    if !provider_ok {
        return Some(ValidationError::new(
            "upi_id",
            "UPI provider may only contain letters, digits, and '.'",
        ));
    }
    None
}

#[wasm_bindgen]
pub fn validate_upi_id_wasm(upi_id: &str) -> JsValue {
    to_js(check_upi_id(upi_id).into_iter().collect())
}

/// Internal: minimal RFC-5321-shaped email check.
///
/// We deliberately avoid a full RFC validator — the cost/benefit there is poor.
/// Instead: exactly one '@', non-empty local part of <= 64 chars, domain with
/// at least one '.' and a TLD of >= 2 chars, no whitespace anywhere.
fn check_email(email: &str) -> Option<ValidationError> {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return Some(ValidationError::new("email", "Email cannot be empty"));
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Some(ValidationError::new(
            "email",
            "Email must not contain whitespace",
        ));
    }
    let Some((local, domain)) = trimmed.split_once('@') else {
        return Some(ValidationError::new("email", "Email must contain '@'"));
    };
    if local.is_empty() || local.len() > 64 || domain.contains('@') {
        return Some(ValidationError::new("email", "Invalid email format"));
    }
    let Some((host, tld)) = domain.rsplit_once('.') else {
        return Some(ValidationError::new(
            "email",
            "Email domain must contain a '.'",
        ));
    };
    if host.is_empty() || tld.len() < 2 {
        return Some(ValidationError::new("email", "Invalid email domain"));
    }
    None
}

#[wasm_bindgen]
pub fn validate_email_wasm(email: &str) -> JsValue {
    to_js(check_email(email).into_iter().collect())
}

/// Internal: phone number validation.
///
/// Strips spaces, dashes, parentheses, and a leading '+', then requires the
/// remainder to be 7-15 ASCII digits (E.164 range). Looser than full E.164 (no
/// country-code-table check) but rejects the common typos that resolve_bulk's
/// phone match would otherwise quietly miss.
fn check_phone(phone: &str) -> Option<ValidationError> {
    let trimmed = phone.trim();
    if trimmed.is_empty() {
        return Some(ValidationError::new("phone", "Phone cannot be empty"));
    }
    let digits: String = trimmed
        .chars()
        .filter(|c| !matches!(c, ' ' | '-' | '(' | ')' | '+'))
        .collect();
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return Some(ValidationError::new(
            "phone",
            "Phone may only contain digits and ' -()+' separators",
        ));
    }
    if !(7..=15).contains(&digits.len()) {
        return Some(ValidationError::new(
            "phone",
            "Phone must have between 7 and 15 digits",
        ));
    }
    None
}

#[wasm_bindgen]
pub fn validate_phone_wasm(phone: &str) -> JsValue {
    to_js(check_phone(phone).into_iter().collect())
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

    #[test]
    fn upi_id_accepts_canonical_forms() {
        assert!(check_upi_id("john.doe@okhdfcbank").is_none());
        assert!(check_upi_id("user_name-1@paytm").is_none());
        assert!(check_upi_id("abc@sbi").is_none());
    }

    #[test]
    fn upi_id_rejects_malformed() {
        assert!(check_upi_id("").is_some());
        assert!(check_upi_id("no-at-sign").is_some());
        assert!(check_upi_id("two@@signs").is_some());
        assert!(check_upi_id("ab@bank").is_some()); // handle too short
        assert!(check_upi_id("name@x").is_some()); // provider too short
        assert!(check_upi_id("name with space@bank").is_some());
        assert!(check_upi_id("name@bank!").is_some()); // provider has non-allowed char
    }

    #[test]
    fn email_accepts_common_forms() {
        assert!(check_email("john@example.com").is_none());
        assert!(check_email("a.b+c@sub.example.co.uk").is_none());
    }

    #[test]
    fn email_rejects_malformed() {
        assert!(check_email("").is_some());
        assert!(check_email("noatsign.com").is_some());
        assert!(check_email("user@").is_some()); // missing dot
        assert!(check_email("user@x").is_some()); // domain missing '.'
        assert!(check_email("user@host.x").is_some()); // tld too short
        assert!(check_email("us er@example.com").is_some());
        assert!(check_email("a@@b.com").is_some());
    }

    #[test]
    fn phone_accepts_common_forms() {
        assert!(check_phone("+91 98765 43210").is_none());
        assert!(check_phone("(555) 123-4567").is_none());
        assert!(check_phone("1234567").is_none());
    }

    #[test]
    fn phone_rejects_malformed() {
        assert!(check_phone("").is_some());
        assert!(check_phone("123").is_some()); // too short
        assert!(check_phone(&"1".repeat(16)).is_some()); // too long
        assert!(check_phone("abc1234567").is_some());
    }
}
