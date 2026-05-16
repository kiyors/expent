use chrono::{DateTime, Utc};

pub fn parse_bank_date(date_str: &str) -> Option<DateTime<Utc>> {
    let formats = [
        "%d-%m-%Y",
        "%d/%m/%Y",
        "%Y-%m-%d",
        "%d-%b-%Y",
        "%d %b %Y",
        "%m/%d/%Y",
        "%b %d, %Y",
    ];
    for fmt in formats {
        if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, fmt) {
            return Some(DateTime::from_naive_utc_and_offset(
                dt.and_hms_opt(0, 0, 0)?,
                Utc,
            ));
        }
    }
    tracing::error!("❌ Failed to parse bank transaction date: '{}'", date_str);
    None
}
