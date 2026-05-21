use ocr::schema::generate_cleaned_schema;

#[test]
fn test_generate_cleaned_schema() {
    let schema = generate_cleaned_schema();

    // 1. Basic checks
    assert!(schema.is_object());
    let obj = schema.as_object().unwrap();

    // 2. Gemini unsupported keys should be removed
    assert!(!obj.contains_key("$schema"));
    assert!(!obj.contains_key("$defs"));
    assert!(!obj.contains_key("definitions"));
    assert!(!obj.contains_key("additionalProperties"));

    // 3. Properties should exist and be cleaned
    let props = obj
        .get("properties")
        .expect("Should have properties")
        .as_object()
        .unwrap();
    assert!(props.contains_key("doc_type"));
    assert!(props.contains_key("confidence_score"));

    // 4. Verify anyOf (Optional fields) cleaning
    // raw_text is Option<String>, schemars usually emits anyOf: [type: string, type: null]
    let raw_text = props.get("raw_text").unwrap().as_object().unwrap();
    assert!(!raw_text.contains_key("anyOf"));
    assert_eq!(raw_text.get("type").unwrap(), "string");
}

#[test]
fn test_recursive_cleaning() {
    let schema = generate_cleaned_schema();
    let props = schema.get("properties").unwrap().as_object().unwrap();

    // generic_data is Option<OcrResult>, which has nested fields like items: Vec<LineItem>
    let generic_data = props.get("generic_data").unwrap().as_object().unwrap();
    assert_eq!(generic_data.get("type").unwrap(), "object");

    let inner_props = generic_data.get("properties").unwrap().as_object().unwrap();
    let items = inner_props.get("items").unwrap().as_object().unwrap();
    assert_eq!(items.get("type").unwrap(), "array");

    let items_inner = items.get("items").unwrap().as_object().unwrap();
    assert_eq!(items_inner.get("type").unwrap(), "object");
    assert!(!items_inner.contains_key("title"));
}
