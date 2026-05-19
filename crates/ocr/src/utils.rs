use calamine::{Reader, Xlsx, open_workbook_from_rs};
use csv::ReaderBuilder;
use std::io::Cursor;

pub fn get_media_type(filename: &str) -> &'static str {
    let ext = filename.split('.').last().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf" => "application/pdf",
        "csv" => "text/csv",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xls" => "application/vnd.ms-excel",
        "webp" => "image/webp",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "image/png",
    }
}

pub fn extract_pdf_text(data: &[u8]) -> Result<String, anyhow::Error> {
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("temp_{}.pdf", uuid::Uuid::new_v4()));
    std::fs::write(&temp_path, data)
        .map_err(|e| anyhow::anyhow!("Failed to write temporary PDF file: {}", e))?;

    let text = pdf_extract::extract_text(&temp_path)
        .map_err(|e| anyhow::anyhow!("Failed to extract text from PDF: {}", e));

    let _ = std::fs::remove_file(&temp_path);
    text
}

pub fn parse_csv(data: &[u8]) -> Result<String, anyhow::Error> {
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(data);

    let mut result = String::new();
    for (i, record) in reader.records().enumerate() {
        let record =
            record.map_err(|e| anyhow::anyhow!("CSV parsing error at row {}: {}", i + 1, e))?;
        result.push_str(&record.iter().collect::<Vec<_>>().join(", "));
        result.push('\n');
    }

    if result.is_empty() {
        return Err(anyhow::anyhow!("CSV file is empty or has no valid rows"));
    }

    Ok(result)
}

pub fn parse_excel(data: &[u8]) -> Result<String, anyhow::Error> {
    let cursor = Cursor::new(data);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|e| anyhow::anyhow!("Failed to open Excel workbook: {}", e))?;

    let mut result = String::new();
    if let Some(res) = workbook.worksheet_range_at(0) {
        let range = res.map_err(|e| anyhow::anyhow!("Failed to read Excel worksheet: {}", e))?;
        for row in range.rows() {
            result.push_str(
                &row.iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            result.push('\n');
        }
    }

    if result.is_empty() {
        return Err(anyhow::anyhow!(
            "Excel worksheet is empty or could not be read"
        ));
    }

    Ok(result)
}

pub fn parse_bank_date(date_str: &str) -> Option<chrono::DateTime<chrono::Utc>> {
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
            return Some(chrono::DateTime::from_naive_utc_and_offset(
                dt.and_hms_opt(0, 0, 0)?,
                chrono::Utc,
            ));
        }
    }
    tracing::error!("❌ Failed to parse bank transaction date: '{}'", date_str);
    None
}
