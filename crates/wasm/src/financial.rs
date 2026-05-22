use regex::Regex;
use std::str::FromStr;
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

static SYMBOLS_RE: OnceLock<Regex> = OnceLock::new();
static COMPACT_RE: OnceLock<Regex> = OnceLock::new();

#[wasm_bindgen]
pub fn parse_numeric_like(input: &str) -> Option<f64> {
    // 1. Clean whitespace including special unicode ones
    let mut s = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{00A0}' && *c != '\u{202F}')
        .collect::<String>();

    if s.is_empty() {
        return None;
    }

    // 2. Accounting negatives: (1234) -> -1234
    if s.starts_with('(') && s.ends_with(')') {
        s = format!("-{}", &s[1..s.len() - 1]);
    }

    // 3. Strip currency and percent symbols
    let symbols = SYMBOLS_RE.get_or_init(|| Regex::new(r"[%$€£¥₩₹₽₺₪₫฿₦₴₡₲₵₸]").unwrap());
    s = symbols.replace_all(&s, "").to_string();

    // 4. Handle thousands and decimal separators
    let last_comma = s.rfind(',');
    let last_dot = s.rfind('.');

    match (last_comma, last_dot) {
        (Some(c), Some(d)) => {
            if c > d {
                // Comma is decimal, dot is thousand
                s = s.replace('.', "").replace(',', ".");
            } else {
                // Dot is decimal, comma is thousand
                s = s.replace(',', "");
            }
        }
        (Some(c), None) => {
            if has_grouped_thousands(&s, ',') {
                s = s.replace(',', "");
            } else {
                let frac = s.len() - c - 1;
                if (1..=3).contains(&frac) {
                    s = s.replace(',', ".");
                } else {
                    s = s.replace(',', "");
                }
            }
        }
        (None, Some(_)) => {
            if has_grouped_thousands(&s, '.') {
                s = s.replace('.', "");
            } else if s.chars().filter(|&c| c == '.').count() > 1 {
                s = s.replace('.', "");
            }
        }
        _ => {}
    }

    // 5. Handle compact notation
    let compact_re = COMPACT_RE
        .get_or_init(|| Regex::new(r"(?i)^([+-]?\d+\.?\d*|\d*\.\d+)([KMBTPG]B?|B)$").unwrap());
    if let Some(caps) = compact_re.captures(&s) {
        let base_num = f64::from_str(&caps[1]).ok()?;
        let suffix = caps[2].to_uppercase();

        if suffix == "B" {
            return if base_num.fract() == 0.0 && base_num < 1024.0 {
                Some(base_num)
            } else {
                Some(base_num * 1_000_000_000.0)
            };
        }

        let multiplier = match suffix.as_str() {
            "K" => 1_000.0,
            "KB" => 1024.0,
            "M" => 1_000_000.0,
            "MB" => 1024.0 * 1024.0,
            "G" => 1_000_000_000.0,
            "GB" => 1024.0 * 1024.0 * 1024.0,
            "T" => 1_000_000_000_000.0,
            "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
            "P" => 1_000_000_000_000_000.0,
            "PB" => 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0,
            _ => 1.0,
        };
        return Some(base_num * multiplier);
    }

    f64::from_str(&s).ok()
}

fn has_grouped_thousands(value: &str, sep: char) -> bool {
    let s = if value.starts_with(['+', '-']) {
        &value[1..]
    } else {
        value
    };
    let parts: Vec<&str> = s.split(sep).collect();
    if parts.len() < 2 {
        return false;
    }
    if parts.iter().any(|p| p.is_empty()) {
        return false;
    }
    if parts[0].len() > 3 || parts[0] == "0" {
        return false;
    }
    parts[1..]
        .iter()
        .all(|p| p.len() == 3 && p.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_numeric_like() {
        assert_eq!(parse_numeric_like("123"), Some(123.0));
        assert_eq!(parse_numeric_like("-123.45"), Some(-123.45));
        assert_eq!(parse_numeric_like("(1234)"), Some(-1234.0));
        assert_eq!(parse_numeric_like("$1,234.56"), Some(1234.56));
        assert_eq!(parse_numeric_like("€1.234,56"), Some(1234.56));
        assert_eq!(parse_numeric_like("50%"), Some(50.0));
        assert_eq!(parse_numeric_like("1.5M"), Some(1_500_000.0));
        assert_eq!(
            parse_numeric_like("2GB"),
            Some(2.0 * 1024.0 * 1024.0 * 1024.0)
        );
        assert_eq!(parse_numeric_like("500B"), Some(500.0));
        assert_eq!(parse_numeric_like("1.5B"), Some(1_500_000_000.0));
    }
}
