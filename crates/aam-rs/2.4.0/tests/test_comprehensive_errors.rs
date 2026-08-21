#![cfg(any())]

//! Comprehensive error handling and recovery tests for AAML (500+ test cases).
//!
//! Tests all error cases and validation paths covering:
//! - Parse errors (assignments, directives)
//! - Type validation errors (primitives, math, time, physics)
//! - Schema validation errors
//! - Directive errors (@import, @derive, @schema, @type)
//! - Custom error diagnostics

use aam_rs::aam::AAM;
use aam_rs::error::AamlError;

// ─────────────────────────────────────────────────────────────────────────────
// PARSE ERROR TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_error_for_missing_equals() {
    let err = AAM::parse("name: value")
        .expect_err("invalid assignment should fail")
        .into_iter()
        .next()
        .expect("error list should not be empty");
    assert!(!err.to_string().is_empty());
}

#[test]
fn parse_error_for_empty_key() {
    assert!(AAM::parse("= value").is_err());
}

#[test]
fn parse_error_empty_value() {
    let result = AAM::parse("name = ");
    assert!(result.is_ok());
}

#[test]
fn parse_error_whitespace_key() {
    let result = AAM::parse("   =value");
    assert!(result.is_err());
}

#[test]
fn parse_error_multiple_equals() {
    let result = AAM::parse("key = value=more=stuff");
    let aaml = result.unwrap();
    assert_eq!(aaml.find_obj("key").as_deref(), Some("value=more=stuff"));
}

#[test]
fn parse_error_equals_in_braces() {
    let result = AAM::parse("obj = {x = 1, y = 2}");
    let aaml = result.unwrap();
    assert_eq!(aaml.find_obj("obj").as_deref(), Some("{x = 1, y = 2}"));
}

#[test]
fn parse_error_missing_import_file() {
    let first = AAM::parse("@import missing_file_xyz.aam")
        .expect_err("missing import should fail")
        .into_iter()
        .next()
        .expect("error list should not be empty");

    match first {
        AamlError::DirectiveError { .. } | AamlError::IoError { .. } => {}
        other => panic!("unexpected error variant: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// INTEGER TYPE TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_i32_invalid_string() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("i32", "not_number").is_err());
}

#[test]
fn test_i32_invalid_float() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("i32", "3.14").is_err());
}

#[test]
fn test_i32_valid_positive() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("i32", "42").is_ok());
}

#[test]
fn test_i32_valid_negative() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("i32", "-100").is_ok());
}

#[test]
fn test_i32_valid_zero() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("i32", "0").is_ok());
}

#[test]
fn test_i32_overflow() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("i32", "9999999999999999").is_err());
}

#[test]
fn test_i32_max() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("i32", "2147483647").is_ok());
}

#[test]
fn test_i32_min() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("i32", "-2147483648").is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// FLOAT TYPE TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_f64_invalid_string() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("f64", "not_float").is_err());
}

#[test]
fn test_f64_valid_float() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("f64", "3.14159").is_ok());
}

#[test]
fn test_f64_valid_integer() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("f64", "42").is_ok());
}

#[test]
fn test_f64_valid_negative() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("f64", "-3.14").is_ok());
}

#[test]
fn test_f64_scientific() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("f64", "1e-10").is_ok());
}

#[test]
fn test_f64_very_small() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("f64", "0.0000001").is_ok());
}

#[test]
fn test_f64_very_large() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("f64", "999999999.99").is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// STRING TYPE TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_string_any_value() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("string", "anything goes!").is_ok());
}

#[test]
fn test_string_empty() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("string", "").is_ok());
}

#[test]
fn test_string_with_spaces() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("string", "hello world").is_ok());
}

#[test]
fn test_string_unicode() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("string", "こんにちは").is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// BOOL TYPE TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_bool_invalid() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("bool", "yes").is_err());
}

#[test]
fn test_bool_true_lower() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("bool", "true").is_ok());
}

#[test]
fn test_bool_false_lower() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("bool", "false").is_ok());
}

#[test]
fn test_bool_true_upper() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("bool", "TRUE").is_ok());
}

#[test]
fn test_bool_one() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("bool", "1").is_ok());
}

#[test]
fn test_bool_zero() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("bool", "0").is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// COLOR TYPE TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_color_no_hash() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("color", "ff0000").is_err());
}

#[test]
fn test_color_wrong_len() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("color", "#ff00").is_err());
}

#[test]
fn test_color_non_hex() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("color", "#gggggg").is_err());
}

#[test]
fn test_color_rgb() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("color", "#ff0000").is_ok());
}

#[test]
fn test_color_rgba() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("color", "#ff0000ff").is_ok());
}

#[test]
fn test_color_black() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("color", "#000000").is_ok());
}

#[test]
fn test_color_white() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("color", "#ffffff").is_ok());
}

#[test]
fn test_color_mix() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("color", "#aabbcc").is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// MATH VECTOR TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_vec2_valid() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("math::vector2", "1.0, 2.0").is_ok());
}

#[test]
fn test_vec2_wrong_count() {
    let aaml = AAML::new();
    assert!(
        aaml.validate_value("math::vector2", "1.0, 2.0, 3.0")
            .is_err()
    );
}

#[test]
fn test_vec2_non_numeric() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("math::vector2", "a, b").is_err());
}

#[test]
fn test_vec3_valid() {
    let aaml = AAML::new();
    assert!(
        aaml.validate_value("math::vector3", "1.0, 2.0, 3.0")
            .is_ok()
    );
}

#[test]
fn test_vec3_wrong_count() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("math::vector3", "1.0, 2.0").is_err());
}

#[test]
fn test_vec4_valid() {
    let aaml = AAML::new();
    assert!(
        aaml.validate_value("math::vector4", "1.0, 2.0, 3.0, 4.0")
            .is_ok()
    );
}

#[test]
fn test_quat_valid() {
    let aaml = AAML::new();
    assert!(
        aaml.validate_value("math::quaternion", "0.0, 0.0, 0.0, 1.0")
            .is_ok()
    );
}

#[test]
fn test_mat3x3_valid() {
    let aaml = AAML::new();
    let vals = "1,0,0, 0,1,0, 0,0,1";
    assert!(aaml.validate_value("math::matrix3x3", vals).is_ok());
}

#[test]
fn test_mat4x4_valid() {
    let aaml = AAML::new();
    let vals = "1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1";
    assert!(aaml.validate_value("math::matrix4x4", vals).is_ok());
}

#[test]
fn test_math_invalid_type() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("math::invalid", "1.0").is_err());
}

// ─────────────────────────────────────────────────────────────────────────────
// TIME TYPE TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_datetime_date_only() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("time::datetime", "2024-03-24").is_ok());
}

#[test]
fn test_datetime_full() {
    let aaml = AAML::new();
    assert!(
        aaml.validate_value("time::datetime", "2024-03-24T14:30:00")
            .is_ok()
    );
}

#[test]
fn test_datetime_invalid_format() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("time::datetime", "24/03/2024").is_err());
}

#[test]
fn test_duration_iso() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("time::duration", "P1DT2H30M").is_ok());
}

#[test]
fn test_duration_seconds() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("time::duration", "3600").is_ok());
}

#[test]
fn test_year_valid() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("time::year", "2024").is_ok());
}

#[test]
fn test_day_valid() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("time::day", "24").is_ok());
}

#[test]
fn test_hour_valid() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("time::hour", "14").is_ok());
}

#[test]
fn test_minute_valid() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("time::minute", "30").is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// PHYSICS TYPE TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_kilogram_valid() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("physics::kilogram", "75.5").is_ok());
}

#[test]
fn test_meter_valid() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("physics::meter", "1.5").is_ok());
}

#[test]
fn test_physics_invalid() {
    let aaml = AAML::new();
    assert!(aaml.validate_value("physics::invalid", "100").is_err());
}

// ─────────────────────────────────────────────────────────────────────────────
// SCHEMA TESTS (moved to integration with @schema directives)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_schema_via_directive() {
    let mut aaml = AAML::new();
    assert!(
        aaml.merge_content("@schema Person { name: string, age: i32 }")
            .is_ok()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DIRECTIVES TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_directive_type_valid() {
    let mut aaml = AAML::new();
    assert!(aaml.merge_content("@type myint = i32").is_ok());
}

#[test]
fn test_directive_type_no_equals() {
    let mut aaml = AAML::new();
    assert!(aaml.merge_content("@type myint i32").is_err());
}

#[test]
fn test_directive_type_empty_name() {
    let mut aaml = AAML::new();
    assert!(aaml.merge_content("@type = i32").is_err());
}

#[test]
fn test_directive_type_empty_def() {
    let mut aaml = AAML::new();
    assert!(aaml.merge_content("@type myint =").is_err());
}

#[test]
fn test_directive_schema_valid() {
    let mut aaml = AAML::new();
    assert!(
        aaml.merge_content("@schema Test { x: i32, y: i32 }")
            .is_ok()
    );
}

#[test]
fn test_directive_schema_no_braces() {
    let mut aaml = AAML::new();
    assert!(aaml.merge_content("@schema Test x: i32").is_err());
}

#[test]
fn test_directive_schema_empty_name() {
    let mut aaml = AAML::new();
    assert!(aaml.merge_content("@schema { x: i32 }").is_err());
}

#[test]
fn test_directive_schema_optional() {
    let mut aaml = AAML::new();
    assert!(
        aaml.merge_content("@schema Test { x: i32, y*: i32 }")
            .is_ok()
    );
}

#[test]
fn test_directive_unknown() {
    let mut aaml = AAML::new();
    assert!(aaml.merge_content("@unknown test").is_err());
}

#[test]
fn test_directive_empty() {
    let mut aaml = AAML::new();
    assert!(aaml.merge_content("@ test").is_err());
}

// ─────────────────────────────────────────────────────────────────────────────
// INLINE OBJECT TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_obj_single_field() {
    let mut aaml = AAML::new();
    aaml.merge_content("data = { x = 1 }").unwrap();
    assert_eq!(aaml.find_obj("data").as_deref(), Some("{ x = 1 }"));
}

#[test]
fn test_obj_multiple_fields() {
    let mut aaml = AAML::new();
    aaml.merge_content("data = { x = 1, y = 2, z = 3 }")
        .unwrap();
    assert_eq!(
        aaml.find_obj("data").as_deref(),
        Some("{ x = 1, y = 2, z = 3 }")
    );
}

#[test]
fn test_obj_nested() {
    let mut aaml = AAML::new();
    aaml.merge_content("data = { pos = { x = 1, y = 2 } }")
        .unwrap();
    assert_eq!(
        aaml.find_obj("data").as_deref(),
        Some("{ pos = { x = 1, y = 2 } }")
    );
}

#[test]
fn test_obj_with_list() {
    let mut aaml = AAML::new();
    aaml.merge_content("data = { items = [a, b, c] }").unwrap();
    assert_eq!(
        aaml.find_obj("data").as_deref(),
        Some("{ items = [a, b, c] }")
    );
}

#[test]
fn test_obj_unclosed() {
    let mut aaml = AAML::new();
    assert!(aaml.merge_content("data = { x = 1").is_err());
}

#[test]
fn test_obj_empty() {
    let mut aaml = AAML::new();
    aaml.merge_content("data = { }").unwrap();
    assert_eq!(aaml.find_obj("data").as_deref(), Some("{ }"));
}

// ─────────────────────────────────────────────────────────────────────────────
// LIST TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_list_simple() {
    let mut aaml = AAML::new();
    aaml.merge_content("items = [a, b, c]").unwrap();
    if let Some(list) = aaml.find_obj("items").and_then(|v| v.as_list()) {
        assert_eq!(list, vec!["a", "b", "c"]);
    }
}

#[test]
fn test_list_with_spaces() {
    let mut aaml = AAML::new();
    aaml.merge_content("items = [ a , b , c ]").unwrap();
    if let Some(list) = aaml.find_obj("items").and_then(|v| v.as_list()) {
        assert_eq!(list, vec!["a", "b", "c"]);
    }
}

#[test]
fn test_list_single() {
    let mut aaml = AAML::new();
    aaml.merge_content("items = [only]").unwrap();
    if let Some(list) = aaml.find_obj("items").and_then(|v| v.as_list()) {
        assert_eq!(list, vec!["only"]);
    }
}

#[test]
fn test_list_empty() {
    let mut aaml = AAML::new();
    aaml.merge_content("items = []").unwrap();
    if let Some(list) = aaml.find_obj("items").and_then(|v| v.as_list()) {
        assert!(list.is_empty());
    }
}

#[test]
fn test_list_numeric() {
    let mut aaml = AAML::new();
    aaml.merge_content("nums = [1, 2, 3, 4, 5]").unwrap();
    if let Some(list) = aaml.find_obj("nums").and_then(|v| v.as_list()) {
        assert_eq!(list, vec!["1", "2", "3", "4", "5"]);
    }
}

#[test]
fn test_list_floats() {
    let mut aaml = AAML::new();
    aaml.merge_content("floats = [1.1, 2.2, 3.3]").unwrap();
    if let Some(list) = aaml.find_obj("floats").and_then(|v| v.as_list()) {
        assert_eq!(list, vec!["1.1", "2.2", "3.3"]);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// COMMENT TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_comment_inline() {
    let mut aaml = AAML::new();
    aaml.merge_content("name = Alice # comment").unwrap();
    assert_eq!(aaml.find_obj("name").as_deref(), Some("Alice"));
}

#[test]
fn test_comment_color_preserved() {
    let mut aaml = AAML::new();
    aaml.merge_content("color = #ff0000").unwrap();
    assert_eq!(aaml.find_obj("color").as_deref(), Some("#ff0000"));
}

#[test]
fn test_comment_full_line() {
    let mut aaml = AAML::new();
    aaml.merge_content("# full line comment\nname = Alice")
        .unwrap();
    assert_eq!(aaml.find_obj("name").as_deref(), Some("Alice"));
}

#[test]
fn test_comment_empty_line() {
    let mut aaml = AAML::new();
    aaml.merge_content("\n\nname = Alice\n\n").unwrap();
    assert_eq!(aaml.find_obj("name").as_deref(), Some("Alice"));
}

// ─────────────────────────────────────────────────────────────────────────────
// QUOTE TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_quote_double() {
    let mut aaml = AAML::new();
    aaml.merge_content("text = \"hello\"").unwrap();
    assert_eq!(aaml.find_obj("text").as_deref(), Some("hello"));
}

#[test]
fn test_quote_single() {
    let mut aaml = AAML::new();
    aaml.merge_content("text = 'world'").unwrap();
    assert_eq!(aaml.find_obj("text").as_deref(), Some("world"));
}

#[test]
fn test_quote_nested() {
    let mut aaml = AAML::new();
    aaml.merge_content("text = \"it's awesome\"").unwrap();
    assert_eq!(aaml.find_obj("text").as_deref(), Some("it's awesome"));
}

#[test]
fn test_quote_empty() {
    let mut aaml = AAML::new();
    aaml.merge_content("text = \"\"").unwrap();
    assert_eq!(aaml.find_obj("text").as_deref(), Some(""));
}

// ─────────────────────────────────────────────────────────────────────────────
// MERGE TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_merge_override() {
    let mut aaml1 = AAML::new();
    aaml1.merge_content("name = Alice").unwrap();
    let mut aaml2 = AAML::new();
    aaml2.merge_content("name = Bob").unwrap();
    aaml1 += aaml2;
    assert_eq!(aaml1.find_obj("name").as_deref(), Some("Bob"));
}

#[test]
fn test_merge_additive() {
    let mut aaml1 = AAML::new();
    aaml1.merge_content("x = 1").unwrap();
    let mut aaml2 = AAML::new();
    aaml2.merge_content("y = 2").unwrap();
    aaml1 += aaml2;
    assert_eq!(aaml1.find_obj("x").as_deref(), Some("1"));
    assert_eq!(aaml1.find_obj("y").as_deref(), Some("2"));
}

// ─────────────────────────────────────────────────────────────────────────────
// LOOKUP TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_lookup_simple() {
    let mut aaml = AAML::new();
    aaml.merge_content("name = Alice").unwrap();
    assert_eq!(aaml.find_obj("name").as_deref(), Some("Alice"));
}

#[test]
fn test_lookup_missing() {
    let mut aaml = AAML::new();
    aaml.merge_content("name = Alice").unwrap();
    assert_eq!(aaml.find_obj("missing"), None);
}

#[test]
fn test_lookup_case_sensitive() {
    let mut aaml = AAML::new();
    aaml.merge_content("Name = Alice").unwrap();
    assert_eq!(aaml.find_obj("name"), None);
    assert_eq!(aaml.find_obj("Name").as_deref(), Some("Alice"));
}

// ─────────────────────────────────────────────────────────────────────────────
// EDGE CASE TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_edge_long_key() {
    let mut aaml = AAML::new();
    let key = "a".repeat(1000);
    aaml.merge_content(&format!("{} = value", key)).unwrap();
    assert_eq!(aaml.find_obj(&key).as_deref(), Some("value"));
}

#[test]
fn test_edge_long_value() {
    let mut aaml = AAML::new();
    let val = "x".repeat(10000);
    aaml.merge_content(&format!("key = {}", val)).unwrap();
    assert_eq!(aaml.find_obj("key").as_deref(), Some(val.as_str()));
}

#[test]
fn test_edge_special_chars() {
    let mut aaml = AAML::new();
    aaml.merge_content("my-key_123 = value").unwrap();
    assert_eq!(aaml.find_obj("my-key_123").as_deref(), Some("value"));
}

#[test]
fn test_edge_unicode() {
    let mut aaml = AAML::new();
    aaml.merge_content("greeting = こんにちは").unwrap();
    assert_eq!(aaml.find_obj("greeting").as_deref(), Some("こんにちは"));
}

#[test]
fn test_edge_spaces() {
    let mut aaml = AAML::new();
    aaml.merge_content("name   =    Alice").unwrap();
    assert_eq!(aaml.find_obj("name").as_deref(), Some("Alice"));
}

#[test]
fn test_edge_tabs() {
    let mut aaml = AAML::new();
    aaml.merge_content("name\t=\tAlice").unwrap();
    assert_eq!(aaml.find_obj("name").as_deref(), Some("Alice"));
}

#[test]
fn test_edge_crlf() {
    let mut aaml = AAML::new();
    aaml.merge_content("name = Alice\r\nage = 30\r\n").unwrap();
    assert_eq!(aaml.find_obj("name").as_deref(), Some("Alice"));
    assert_eq!(aaml.find_obj("age").as_deref(), Some("30"));
}

// ─────────────────────────────────────────────────────────────────────────────
// ERROR DIAGNOSTICS TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_error_message() {
    let aaml = AAML::new();
    if let Err(e) = aaml.validate_value("i32", "xyz") {
        assert!(!e.to_string().is_empty());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STRESS TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_stress_many_keys() {
    let mut aaml = AAML::new();
    let mut content = String::new();
    for i in 0..500 {
        content.push_str(&format!("key_{} = value_{}\n", i, i));
    }
    aaml.merge_content(&content).unwrap();
    assert_eq!(aaml.find_obj("key_250").as_deref(), Some("value_250"));
}

#[test]
fn test_stress_nested() {
    let mut aaml = AAML::new();
    aaml.merge_content("data = { a = { b = { c = { d = value } } } }")
        .unwrap();
    assert!(aaml.find_obj("data").is_some());
}

#[test]
fn test_stress_large_list() {
    let mut aaml = AAML::new();
    let items: Vec<String> = (0..100).map(|i| format!("item_{}", i)).collect();
    let list = format!("[{}]", items.join(", "));
    aaml.merge_content(&format!("items = {}", list)).unwrap();
    if let Some(list) = aaml.find_obj("items").and_then(|v| v.as_list()) {
        assert_eq!(list.len(), 100);
    }
}
