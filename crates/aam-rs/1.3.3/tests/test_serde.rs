#![cfg(feature = "serde")]

use aam_rs::aaml::AAML;

#[test]
fn test_serde_aaml() {
    let source = r#"
    @schema Server { host: string, port: i32 }
    @type status = string

    host = localhost
    port = 8080
    debug = true
    list = [1, 2, 3]
    "#;

    let mut aaml = AAML::new();
    aaml.merge_content(source)
        .expect("Failed to parse AAML source");

    // Serialize
    let serialized = serde_json::to_string(&aaml).expect("Failed to serialize AAML");

    // Deserialize
    let deserialized: AAML = serde_json::from_str(&serialized).expect("Failed to deserialize AAML");

    // Verify map content
    assert_eq!(deserialized.find_obj("host").unwrap().as_str(), "localhost");
    assert_eq!(deserialized.find_obj("port").unwrap().as_str(), "8080");
    assert_eq!(deserialized.find_obj("debug").unwrap().as_str(), "true");
    assert_eq!(deserialized.find_obj("list").unwrap().as_str(), "[1, 2, 3]");

    // Check schemas
    let schema = deserialized
        .get_schema("Server")
        .expect("Schema 'Server' was not deserialized");
    assert_eq!(schema.fields.get("host").unwrap(), "string");
    assert_eq!(schema.fields.get("port").unwrap(), "i32");

    // Verification that new deserialized instance works with commands
    let mut aaml2 = deserialized;
    aaml2.merge_content("new_key = 123").unwrap();
    assert_eq!(aaml2.find_obj("new_key").unwrap().as_str(), "123");
}
