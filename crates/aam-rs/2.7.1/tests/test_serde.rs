#![cfg(feature = "serde")]

use aam_rs::builder::{AAMBuilder, SchemaField};
use aam_rs::found_value::FoundValue;

#[test]
fn test_serde_found_value() {
    let value = FoundValue::new("[a, b, c]");
    let serialized = serde_json::to_string(&value).expect("Failed to serialize FoundValue");
    let deserialized: FoundValue =
        serde_json::from_str(&serialized).expect("Failed to deserialize FoundValue");

    assert_eq!(deserialized.as_str(), "[a, b, c]");
    assert_eq!(deserialized.as_list().unwrap(), vec!["a", "b", "c"]);
}

#[test]
fn test_serde_builder_roundtrip() {
    let mut builder = AAMBuilder::new();
    builder
        .schema(
            "Server",
            [
                SchemaField::required("host", "string"),
                SchemaField::required("port", "i32"),
            ],
        )
        .add_line("host", "localhost")
        .add_line("port", "8080");

    let serialized = serde_json::to_string(&builder).expect("Failed to serialize AAMBuilder");
    let deserialized: AAMBuilder =
        serde_json::from_str(&serialized).expect("Failed to deserialize AAMBuilder");

    let output = deserialized.build();
    assert!(output.contains("@schema Server"));
    assert!(output.contains("host = localhost"));
    assert!(output.contains("port = 8080"));
}
