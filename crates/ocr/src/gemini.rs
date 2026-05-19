use crate::schema::{UnifiedExtraction, generate_cleaned_schema};
use crate::strategies::{bank, generic, upi};
use crate::utils::{extract_pdf_text, get_media_type, parse_csv, parse_excel};
use base64::prelude::*;
use reqwest::Client;
use serde_json::json;

pub struct GeminiOcrClient {
    api_key: String,
    model_name: String,
    client: Client,
}

impl GeminiOcrClient {
    pub fn new(api_key: String) -> Self {
        let model_name =
            std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.0-flash-exp".to_string());
        Self {
            api_key,
            model_name,
            client: Client::new(),
        }
    }

    pub fn get_system_prompt(&self) -> String {
        format!(
            r#"
You are an advanced financial data extraction engine. Your task is to analyze the provided document (image, PDF, CSV, or Excel) and extract structured information.

STEP 1: CLASSIFY THE DOCUMENT
- GPAY: Google Pay payment confirmation screenshots (digital).
- BANK_STATEMENT: Monthly bank statements (PDF, CSV, or Image tables).
- GENERIC: Retail receipts, invoices, or other payment proofs.

STEP 2: EXTRACT DATA BASED ON TYPE

--- RULES FOR GPAY ---
{}

--- RULES FOR BANK_STATEMENT ---
{}

--- RULES FOR GENERIC ---
{}

STEP 3: FORMAT OUTPUT
- Return ONLY a valid JSON object matching the requested schema.
- Use snake_case for all keys.
- If a field is missing, set it to null.
- Ensure 'confidence_score' reflects your certainty (0.0 to 1.0).
"#,
            upi::gpay::prompts::SYSTEM_PROMPT,
            bank::icici::prompts::ICICI_PROMPT,
            generic::prompts::SYSTEM_PROMPT
        )
    }

    pub async fn extract_from_bytes(
        &self,
        data: &[u8],
        filename: &str,
    ) -> Result<UnifiedExtraction, anyhow::Error> {
        let media_type = get_media_type(filename);

        let mut extracted_text = String::new();
        if media_type == "application/pdf" {
            extracted_text = extract_pdf_text(data).unwrap_or_default();
        } else if media_type == "text/csv" {
            extracted_text = parse_csv(data).unwrap_or_default();
        } else if media_type == "application/vnd.ms-excel"
            || media_type == "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        {
            extracted_text = parse_excel(data).unwrap_or_default();
        }

        let base64_data = BASE64_STANDARD.encode(data);
        let schema = generate_cleaned_schema();

        let mut parts = vec![json!({ "text": "Extract data from this document." })];

        if !extracted_text.is_empty() {
            parts.push(json!({ "text": format!("EXTRACTED TEXT CONTENT:\n{}", extracted_text) }));
        }

        parts.push(json!({
            "inlineData": {
                "mimeType": media_type,
                "data": base64_data
            }
        }));

        let payload = json!({
            "systemInstruction": {
                "parts": [{ "text": self.get_system_prompt() }]
            },
            "contents": [{
                "parts": parts
            }],
            "generationConfig": {
                "responseMimeType": "application/json",
                "responseSchema": schema,
                "temperature": 0.0
            }
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model_name, self.api_key
        );

        let res = self.client.post(&url).json(&payload).send().await?;
        let json_res: serde_json::Value = res.json().await?;

        let text_result = json_res["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Gemini response: {:?}", json_res))?;

        // Handle potential markdown formatting in Gemini response
        let cleaned_json = if text_result.trim().starts_with("```json") {
            text_result
                .trim()
                .strip_prefix("```json")
                .unwrap()
                .strip_suffix("```")
                .unwrap()
                .trim()
        } else if text_result.trim().starts_with("```") {
            text_result
                .trim()
                .strip_prefix("```")
                .unwrap()
                .strip_suffix("```")
                .unwrap()
                .trim()
        } else {
            text_result.trim()
        };

        let extracted: UnifiedExtraction = serde_json::from_str(cleaned_json)?;
        Ok(extracted)
    }
}
