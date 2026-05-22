use db::{LineItem, OcrResult};
use rust_decimal::Decimal;
use std::collections::HashMap;

pub fn merge_ocr_results(results: Vec<OcrResult>) -> OcrResult {
    if results.is_empty() {
        return OcrResult {
            raw_text: String::new(),
            vendor: None,
            amount: None,
            date: None,
            upi_id: None,
            category_id: None,
            wallet_id: None,
            contact_id: None,
            confidence_score: 0.0,
            items: Vec::new(),
        };
    }

    let mut merged_raw_text = String::new();
    let mut vendor_counts: HashMap<String, usize> = HashMap::new();
    let mut max_amount: Option<Decimal> = None;
    let mut earliest_date = None;
    let mut all_items = Vec::new();
    let mut total_confidence = 0.0;

    for res in &results {
        // 1. Combine raw text
        if !merged_raw_text.is_empty() {
            merged_raw_text.push_str("\n---\n");
        }
        merged_raw_text.push_str(&res.raw_text);

        // 2. Track vendors
        if let Some(v) = &res.vendor {
            *vendor_counts.entry(v.clone()).or_insert(0) += 1;
        }

        // 3. Max amount
        if let Some(amt) = res.amount {
            if max_amount.is_none() || Some(amt) > max_amount {
                max_amount = Some(amt);
            }
        }

        // 4. Earliest date
        if let Some(d) = res.date {
            if earliest_date.is_none() || Some(d) < earliest_date {
                earliest_date = Some(d);
            }
        }

        // 5. Collect items
        all_items.extend(res.items.clone());

        // 6. Accumulate confidence for averaging
        total_confidence += res.confidence_score;
    }

    // Pick most frequent vendor
    let vendor = vendor_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(v, _)| v)
        .or_else(|| results.first().and_then(|r| r.vendor.clone()));

    // Deduplicate items (basic by name/price)
    // and specifically handle "Tax" deduplication as per plan
    let mut unique_items: Vec<LineItem> = Vec::new();
    for item in all_items {
        let is_tax = item.name.to_lowercase().contains("tax")
            || item.name.to_lowercase().contains("vat")
            || item.name.to_lowercase().contains("gst");

        if is_tax {
            // For taxes, we might want to be careful.
            // If multiple pages repeat the same tax, we should deduplicate.
            // For now, let's keep it simple: if an item with same name and price exists, skip.
            if !unique_items
                .iter()
                .any(|i| i.name == item.name && i.price == item.price)
            {
                unique_items.push(item);
            }
        } else {
            unique_items.push(item);
        }
    }

    OcrResult {
        raw_text: merged_raw_text,
        vendor,
        amount: max_amount,
        date: earliest_date,
        upi_id: results.first().and_then(|r| r.upi_id.clone()),
        category_id: results.first().and_then(|r| r.category_id.clone()),
        wallet_id: results.first().and_then(|r| r.wallet_id.clone()),
        contact_id: results.first().and_then(|r| r.contact_id.clone()),
        confidence_score: total_confidence / results.len() as f32,
        items: unique_items,
    }
}
