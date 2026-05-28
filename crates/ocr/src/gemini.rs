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
            std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
        // Bound connect/request time so a hung Gemini call cannot hold a worker
        // semaphore permit indefinitely. Falls back to a default client if the
        // builder ever fails (it effectively never does for these options).
        let client = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            api_key,
            model_name,
            client,
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
        self.extract_batch(vec![(data.to_vec(), filename.to_string())])
            .await
            .map(|mut v| v.remove(0))
    }

    pub async fn extract_batch(
        &self,
        files: Vec<(Vec<u8>, String)>,
    ) -> Result<Vec<UnifiedExtraction>, anyhow::Error> {
        let mut futures = Vec::new();

        for (data, filename) in files {
            futures.push(self.extract_single(data, filename));
        }

        let results = futures::future::join_all(futures).await;
        let mut successful = Vec::new();
        let mut errors = Vec::new();

        for res in results {
            match res {
                Ok(ext) => successful.push(ext),
                Err(e) => errors.push(e),
            }
        }

        if !errors.is_empty() && successful.is_empty() {
            return Err(anyhow::anyhow!(
                "All batch OCR attempts failed. First error: {}",
                errors[0]
            ));
        }

        Ok(successful)
    }

    async fn extract_single(
        &self,
        data: Vec<u8>,
        filename: String,
    ) -> Result<UnifiedExtraction, anyhow::Error> {
        let media_type = get_media_type(&filename);

        let mut extracted_text = String::new();
        if media_type == "application/pdf" {
            extracted_text = extract_pdf_text(&data).unwrap_or_default();
        } else if media_type == "text/csv" {
            extracted_text = parse_csv(&data).unwrap_or_default();
        } else if media_type == "application/vnd.ms-excel"
            || media_type == "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        {
            extracted_text = parse_excel(&data).unwrap_or_default();
        }

        let base64_data = BASE64_STANDARD.encode(&data);
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

        // Pass the API key via header rather than the URL query string so it does
        // not leak into request logs or referrers.
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model_name
        );

        let res = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&payload)
            .send()
            .await?;

        // Surface real API failures (quota, auth, safety blocks, 5xx) instead of
        // letting them fall through to an opaque "failed to parse" error below.
        let status = res.status();
        if !status.is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gemini API error {}: {}", status, body));
        }

        let json_res: serde_json::Value = res.json().await?;

        let text_result = json_res["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Gemini response: {:?}", json_res))?;

        // Strip optional markdown code fences without panicking on malformed
        // (e.g. unterminated) responses from the model.
        let trimmed = text_result.trim();
        let cleaned_json = trimmed
            .strip_prefix("```json")
            .or_else(|| trimmed.strip_prefix("```"))
            .map(|inner| inner.strip_suffix("```").unwrap_or(inner).trim())
            .unwrap_or(trimmed);

        let extracted: UnifiedExtraction = serde_json::from_str(cleaned_json)?;
        Ok(extracted)
    }
}
