//! Currency helpers: formatting (Decimal-precise, locale-light) and detection
//! of the currency referenced by a piece of free-form text (typically OCR
//! output).

use rust_decimal::Decimal;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

/// (ISO code, symbol) for the currencies we care about. Order matters for
/// detection: codes are matched before symbols, and within each list the
/// earlier entries win on ties. INR-first reflects the primary user base.
const CURRENCIES: &[(&str, &str)] = &[
    ("INR", "₹"),
    ("USD", "$"),
    ("EUR", "€"),
    ("GBP", "£"),
    ("JPY", "¥"),
    ("AUD", "A$"),
    ("SGD", "S$"),
    ("AED", "د.إ"),
    ("CHF", "CHF"),
    ("CAD", "C$"),
];

fn symbol_for(code: &str) -> Option<&'static str> {
    CURRENCIES
        .iter()
        .find(|(c, _)| c.eq_ignore_ascii_case(code))
        .map(|(_, sym)| *sym)
}

/// Group digits with comma thousand separators (Western style).
fn group_thousands_western(integer: &str) -> String {
    // Operate on the digits only; the sign is handled by the caller.
    let bytes = integer.as_bytes();
    let mut out = String::with_capacity(integer.len() + integer.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

/// Indian lakhs/crores grouping: last three digits, then groups of two.
/// e.g. 1234567 → "12,34,567"
fn group_thousands_indian(integer: &str) -> String {
    let bytes = integer.as_bytes();
    if bytes.len() <= 3 {
        return integer.to_string();
    }
    let (head, tail) = bytes.split_at(bytes.len() - 3);
    // Re-group head in pairs from the right.
    let mut head_out = String::with_capacity(head.len() + head.len() / 2);
    for (i, b) in head.iter().enumerate() {
        if i > 0 && (head.len() - i).is_multiple_of(2) {
            head_out.push(',');
        }
        head_out.push(*b as char);
    }
    let tail_str = std::str::from_utf8(tail).unwrap_or("");
    format!("{head_out},{tail_str}")
}

/// Format a Decimal-precision amount as a currency string.
///
/// Returns `None` when `amount` does not parse. INR uses Indian
/// lakhs/crores grouping (e.g. `₹12,34,567.89`); other supported codes use
/// Western grouping (`$1,234,567.89`). Unknown codes are still formatted with
/// the code itself as the prefix (e.g. `XYZ 1,234.56`) — the function never
/// silently substitutes a different currency.
#[wasm_bindgen]
pub fn format_currency(amount: &str, currency_code: &str) -> Option<String> {
    let value = Decimal::from_str(amount).ok()?;
    let is_negative = value.is_sign_negative();
    let abs = value.abs();

    // Fixed to 2 decimal places; matches what bank statements and the dashboard
    // expect today. Decimal here avoids the float drift you'd get with f64.
    let rounded = abs.round_dp(2);
    let s = rounded.to_string();
    let (int_part, frac_part) = match s.split_once('.') {
        Some((i, f)) => (i, f),
        None => (s.as_str(), ""),
    };

    let grouped = if currency_code.eq_ignore_ascii_case("INR") {
        group_thousands_indian(int_part)
    } else {
        group_thousands_western(int_part)
    };

    // Always emit two decimal places (zero-padded) for visual stability.
    let frac = match frac_part.len() {
        0 => "00".to_string(),
        1 => format!("{frac_part}0"),
        _ => frac_part[..2].to_string(),
    };

    let prefix = symbol_for(currency_code).unwrap_or(currency_code);
    let sign = if is_negative { "-" } else { "" };
    // Code-prefix (e.g. "CHF") needs a space; symbol-prefix doesn't.
    let separator = if symbol_for(currency_code).is_some() {
        ""
    } else {
        " "
    };
    Some(format!("{sign}{prefix}{separator}{grouped}.{frac}"))
}

/// Best-effort guess at the currency a piece of free-form text refers to.
///
/// Priority: explicit ISO codes (case-insensitive whole-word match) win over
/// symbols, because OCR pipelines often see "INR 250" alongside a unicode
/// rupee sign, and the code is the surer signal. Returns `None` when nothing
/// matches — callers should keep their existing fallback.
#[wasm_bindgen]
pub fn detect_currency_from_text(text: &str) -> Option<String> {
    let lower = text.to_lowercase();

    // 1) Look for ISO codes as whole tokens. We split on common boundary chars
    //    rather than using a regex to keep the wasm bundle small.
    let tokens: Vec<&str> = lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    for (code, _) in CURRENCIES {
        let code_lower = code.to_ascii_lowercase();
        if tokens.iter().any(|t| *t == code_lower) {
            return Some((*code).to_string());
        }
    }

    // 2) Look for symbols. CURRENCIES order is intentional — INR before USD
    //    means a string containing both "₹" and "$" resolves to INR (likely
    //    a forex quote against rupees).
    for (code, sym) in CURRENCIES {
        if text.contains(sym) {
            return Some((*code).to_string());
        }
    }

    // 3) "Rs" / "Rs." prefix is a very common shorthand for INR in receipts.
    if tokens.iter().any(|t| *t == "rs" || *t == "inr") {
        return Some("INR".to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn western_grouping() {
        assert_eq!(group_thousands_western("1234567"), "1,234,567");
        assert_eq!(group_thousands_western("100"), "100");
        assert_eq!(group_thousands_western("1000"), "1,000");
    }

    #[test]
    fn indian_grouping() {
        assert_eq!(group_thousands_indian("1234567"), "12,34,567");
        assert_eq!(group_thousands_indian("100"), "100");
        assert_eq!(group_thousands_indian("1000"), "1,000");
        assert_eq!(group_thousands_indian("100000"), "1,00,000");
    }

    #[test]
    fn formats_known_currencies() {
        assert_eq!(
            format_currency("1234567.89", "INR").as_deref(),
            Some("₹12,34,567.89")
        );
        assert_eq!(
            format_currency("1234567.5", "USD").as_deref(),
            Some("$1,234,567.50")
        );
        assert_eq!(format_currency("-100", "EUR").as_deref(), Some("-€100.00"));
    }

    #[test]
    fn unknown_code_uses_code_prefix() {
        assert_eq!(format_currency("100", "XYZ").as_deref(), Some("XYZ 100.00"));
    }

    #[test]
    fn rejects_bad_amount() {
        assert!(format_currency("not-a-number", "USD").is_none());
    }

    #[test]
    fn detects_iso_code_first() {
        // "INR 250" wins even when "$" also appears.
        assert_eq!(
            detect_currency_from_text("Paid INR 250 (was $4)").as_deref(),
            Some("INR")
        );
    }

    #[test]
    fn detects_symbols() {
        assert_eq!(
            detect_currency_from_text("Total: ₹1,234.50").as_deref(),
            Some("INR")
        );
        assert_eq!(
            detect_currency_from_text("Total: $50").as_deref(),
            Some("USD")
        );
        assert_eq!(
            detect_currency_from_text("Total: £30").as_deref(),
            Some("GBP")
        );
    }

    #[test]
    fn detects_rs_shorthand() {
        assert_eq!(
            detect_currency_from_text("Rs. 250 for chai").as_deref(),
            Some("INR")
        );
    }

    #[test]
    fn returns_none_when_nothing_matches() {
        assert!(detect_currency_from_text("just some text").is_none());
    }
}
