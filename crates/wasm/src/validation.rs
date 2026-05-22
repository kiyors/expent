use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use wasm_bindgen::prelude::*;

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

#[wasm_bindgen]
pub fn validate_transaction_wasm(amount: &str, purpose: &str) -> JsValue {
    let mut errors = Vec::new();

    // 1. Amount validation
    match amount.parse::<Decimal>() {
        Ok(amt) => {
            if amt <= Decimal::ZERO {
                errors.push(ValidationError {
                    field: "amount".to_string(),
                    message: "Amount must be positive".to_string(),
                });
            }
        }
        Err(_) => {
            errors.push(ValidationError {
                field: "amount".to_string(),
                message: "Invalid amount format".to_string(),
            });
        }
    }

    // 2. Purpose validation
    if purpose.trim().is_empty() {
        errors.push(ValidationError {
            field: "purpose".to_string(),
            message: "Purpose cannot be empty".to_string(),
        });
    } else if purpose.len() > 255 {
        errors.push(ValidationError {
            field: "purpose".to_string(),
            message: "Purpose is too long".to_string(),
        });
    }

    let result = ValidationResult {
        is_valid: errors.is_empty(),
        errors,
    };

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn validate_budget_wasm(amount: &str) -> JsValue {
    let mut errors = Vec::new();

    match amount.parse::<Decimal>() {
        Ok(amt) => {
            if amt <= Decimal::ZERO {
                errors.push(ValidationError {
                    field: "amount".to_string(),
                    message: "Amount must be positive".to_string(),
                });
            }
        }
        Err(_) => {
            errors.push(ValidationError {
                field: "amount".to_string(),
                message: "Invalid amount format".to_string(),
            });
        }
    }

    let result = ValidationResult {
        is_valid: errors.is_empty(),
        errors,
    };

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn validate_wallet_wasm(name: &str, balance: &str) -> JsValue {
    let mut errors = Vec::new();

    if name.trim().is_empty() {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Wallet name cannot be empty".to_string(),
        });
    }

    match balance.parse::<Decimal>() {
        Ok(_) => {}
        Err(_) => {
            errors.push(ValidationError {
                field: "balance".to_string(),
                message: "Invalid balance format".to_string(),
            });
        }
    }

    let result = ValidationResult {
        is_valid: errors.is_empty(),
        errors,
    };

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn validate_contact_wasm(name: &str) -> JsValue {
    let mut errors = Vec::new();

    if name.trim().is_empty() {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Contact name cannot be empty".to_string(),
        });
    } else if name.len() > 255 {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Contact name is too long".to_string(),
        });
    }

    let result = ValidationResult {
        is_valid: errors.is_empty(),
        errors,
    };

    serde_wasm_bindgen::to_value(&result).unwrap()
}
