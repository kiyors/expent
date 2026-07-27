use db::{BankStatementResponse, GPayExtraction, OcrResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocType {
    Gpay,
    BankStatement,
    Generic,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct UnifiedExtraction {
    pub doc_type: DocType,
    pub confidence_score: f32,
    pub raw_text: Option<String>,

    pub gpay_data: Option<GPayExtraction>,
    pub bank_data: Option<BankStatementResponse>,
    pub generic_data: Option<OcrResult>,
}

pub fn generate_cleaned_schema() -> serde_json::Value {
    let mut settings = schemars::generate::SchemaSettings::draft07();
    settings.inline_subschemas = true;
    let schema_gen = settings.into_generator();
    let schema = schema_gen.into_root_schema_for::<UnifiedExtraction>();
    let json = serde_json::to_value(schema).unwrap();

    clean_schema(json)
}

fn clean_schema(mut schema: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = schema.as_object_mut() {
        // Remove keys not supported by Gemini
        obj.remove("$schema");
        obj.remove("$defs");
        obj.remove("definitions");
        obj.remove("title");
        obj.remove("description");
        obj.remove("additionalProperties");

        if let Some(props) = obj.get_mut("properties")
            && let Some(props_obj) = props.as_object_mut()
        {
            for (_, v) in props_obj.iter_mut() {
                *v = clean_schema(v.take());
            }
        }

        if let Some(items) = obj.get_mut("items") {
            *items = clean_schema(items.take());
        }

        // Handle anyOf (for Optionals)
        if let Some(any_of) = obj.remove("anyOf")
            && let serde_json::Value::Array(arr) = any_of
        {
            // Find the first non-null type
            let non_null = arr
                .into_iter()
                .find(|v| v.get("type").and_then(|t| t.as_str()) != Some("null"));

            if let Some(val) = non_null {
                let mut cleaned_val = clean_schema(val);

                // If the cleaned value is an object, merge our existing metadata into it
                if let Some(cleaned_obj) = cleaned_val.as_object_mut() {
                    for (k, v) in obj.iter_mut() {
                        if !cleaned_obj.contains_key(k) {
                            cleaned_obj.insert(k.clone(), v.take());
                        }
                    }
                }
                return cleaned_val;
            }
        }

        // Handle array of types: e.g. "type": ["string", "null"] -> "type": "string"
        if let Some(t) = obj.get_mut("type") {
            let extracted_val = if let Some(arr) = t.as_array_mut() {
                arr.iter().position(|v| v.as_str() != Some("null")).map(|pos| arr.swap_remove(pos))
            } else {
                None
            };
            if let Some(val) = extracted_val {
                *t = val;
            }
        }

        // Ensure 'type' is present if 'properties' is present
        if obj.contains_key("properties") && !obj.contains_key("type") {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String("object".to_string()),
            );
        }
    }
    schema
}
