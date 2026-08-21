use aam_rs::aaml::AAML;
use std::collections::HashMap;

mod test_imports;
mod test_derive;
mod test_core;

#[test]
fn test_builtin_types() {
    let aaml = AAML::new();

    assert!(aaml.validate_value("i32", "123").is_ok());
    assert!(aaml.validate_value("i32", "abc").is_err());

    assert!(aaml.validate_value("bool", "true").is_ok());
    assert!(aaml.validate_value("bool", "yes").is_err());

    assert!(aaml.validate_value("f64", "3.14").is_ok());
}

#[test]
fn test_schema_field_validation() {
    let config = r#"
    @schema Device {
        id: i32
        name: string
    }

    id = 42
    name = "Sensor A"
    "#;

    let aaml = AAML::parse(config).expect("Should parse valid config");
    aaml.validate_schemas_completeness().expect("Schema should be complete");
}

#[test]
fn test_schema_validation_failure() {
    let config = r#"
    @schema Device {
        id: i32
    }

    id = "not_a_number"
    "#;

    // Parsing should fail or validation should fail depending on when validation happens.
    // In current implementation, `merge_content` calls `validate_against_schemas` for assignments if schemas exist.
    // However, schema is defined *before* assignment here, so it should be active.

    let res = AAML::parse(config);
    assert!(res.is_err(), "Should fail because 'id' is defined as i32 but assigned string");
}

#[test]
fn test_apply_schema_manual() {
    let mut aaml = AAML::new();
    // Valid schema definition
    aaml.merge_content("@schema Point { x: i32, y: i32 }").unwrap();

    let mut data = HashMap::new();
    data.insert("x".to_string(), "10".to_string());
    data.insert("y".to_string(), "20".to_string());

    assert!(aaml.apply_schema("Point", &data).is_ok());

    data.insert("y".to_string(), "invalid".to_string());
    assert!(aaml.apply_schema("Point", &data).is_err());
}

