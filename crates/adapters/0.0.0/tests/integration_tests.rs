//! Integration tests for the adapters library.
//!
//! Verifies standard structural derives, runtime dynamic schemas, pipeline conversions, and error formats.

use adapters::{
    Adapt, Adapter, ArraySchema, Error, FieldMapper, IntegerSchema, ObjectSchema, Pipeline, Schema,
    SchemaValidator, StringSchema, Validate, ValidationError, Value,
    json::{parse, stringify, stringify_pretty},
};
use adapters_macros::Schema as SchemaDerived;
use std::collections::BTreeMap;

fn make_obj(fields: &[(&str, Value)]) -> Value {
    let mut m = BTreeMap::new();
    for (k, v) in fields {
        m.insert(k.to_string(), v.clone());
    }
    Value::Object(m)
}

#[derive(SchemaDerived, Debug, PartialEq)]
struct User {
    #[schema(min_length = 3, max_length = 32)]
    username: String,
    #[schema(email)]
    email: String,
    #[schema(min = 18, max = 120)]
    age: u8,
}

#[derive(SchemaDerived, Debug, PartialEq)]
struct Address {
    city: String,
    country: String,
}

#[derive(SchemaDerived, Debug, PartialEq)]
struct UserWithAddress {
    username: String,
    address: Address,
}

#[derive(SchemaDerived, Debug, PartialEq)]
struct Profile {
    name: String,
    bio: Option<String>,
    #[schema(default = "India")]
    country: String,
    #[schema(default = 0)]
    score: i32,
}

#[derive(SchemaDerived, Debug, PartialEq)]
#[allow(dead_code)]
struct Strict {
    #[schema(strict)]
    amount: f64,
    #[schema(strict)]
    label: String,
}

#[test]
fn test_derive_basic_round_trip() {
    let u =
        User::from_json(r#"{"username":"alice","email":"alice@example.com","age":25}"#).unwrap();
    assert_eq!(u.username, "alice");
    assert_eq!(u.email, "alice@example.com");
    assert_eq!(u.age, 25);
}

#[test]
fn test_derive_json_to_struct_to_json() {
    let json = r#"{"username":"bob","email":"bob@example.com","age":30}"#;
    let u = User::from_json(json).unwrap();
    let out = u.to_json().unwrap();
    let v1 = parse(json).unwrap();
    let v2 = parse(&out).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn test_validation_min_length_fails() {
    let result = User::from_json(r#"{"username":"ab","email":"a@b.com","age":20}"#);
    assert!(result.is_err());
    let e = result.unwrap_err().to_string();
    assert!(e.contains("min") || e.contains("length"), "error: {e}");
}

#[test]
fn test_validation_max_length_fails() {
    let long = "a".repeat(33);
    let json = format!(r#"{{"username":"{long}","email":"a@b.com","age":20}}"#);
    let result = User::from_json(&json);
    assert!(result.is_err());
}

#[test]
fn test_validation_email_invalid() {
    let result = User::from_json(r#"{"username":"alice","email":"not-email","age":25}"#);
    assert!(result.is_err());
}

#[test]
fn test_validation_email_valid() {
    let result = User::from_json(r#"{"username":"alice","email":"alice@example.com","age":25}"#);
    assert!(result.is_ok());
}

#[test]
fn test_validation_min_int_fails() {
    let result = User::from_json(r#"{"username":"alice","email":"a@b.com","age":10}"#);
    assert!(result.is_err());
}

#[test]
fn test_validation_max_int_fails() {
    let result = User::from_json(r#"{"username":"alice","email":"a@b.com","age":200}"#);
    assert!(result.is_err());
}

#[test]
fn test_validation_required_field_missing() {
    let result = User::from_json(r#"{"email":"a@b.com","age":25}"#);
    assert!(result.is_err());
}

#[test]
fn test_nested_model_round_trip() {
    let json = r#"{"username":"alice","address":{"city":"Berlin","country":"Germany"}}"#;
    let u = UserWithAddress::from_json(json).unwrap();
    assert_eq!(u.username, "alice");
    assert_eq!(u.address.city, "Berlin");
    let out = u.to_json().unwrap();
    let u2 = UserWithAddress::from_json(&out).unwrap();
    assert_eq!(u, u2);
}

#[test]
fn test_nested_validation_error_path() {
    let schema = ObjectSchema::new()
        .field("username", StringSchema::new().required())
        .field(
            "address",
            ObjectSchema::new().field("city", StringSchema::new().required().min_length(3)),
        );
    let v = make_obj(&[
        ("username", Value::String("alice".into())),
        ("address", make_obj(&[("city", Value::String("ab".into()))])),
    ]);
    let err = schema.validate(&v, "root").unwrap_err();
    assert!(err.field.contains("address"), "field path: {}", err.field);
}

struct Celsius(f64);
struct Fahrenheit(f64);

impl Adapt<Celsius> for Fahrenheit {
    fn adapt(c: Celsius) -> Result<Self, Error> {
        Ok(Fahrenheit(c.0 * 9.0 / 5.0 + 32.0))
    }
}

#[test]
fn test_adapt_trait_basic() {
    let f = Fahrenheit::adapt(Celsius(0.0)).unwrap();
    assert_eq!(f.0, 32.0);
    let f2 = Fahrenheit::adapt(Celsius(100.0)).unwrap();
    assert_eq!(f2.0, 212.0);
}

#[test]
fn test_pipeline_chain() {
    let p = Pipeline::new()
        .step(|v| match v {
            Value::Int(n) => Ok(Value::Int(n * 2)),
            other => Ok(other),
        })
        .step(|v| match v {
            Value::Int(n) => Ok(Value::Int(n + 1)),
            other => Ok(other),
        });
    assert_eq!(p.run(Value::Int(5)).unwrap(), Value::Int(11));
}

#[test]
fn test_field_mapper_rename() {
    let mapper = FieldMapper::new()
        .map("first_name", "firstName")
        .map("last_name", "lastName");
    let input = make_obj(&[
        ("first_name", Value::String("Alice".into())),
        ("last_name", Value::String("Smith".into())),
    ]);
    let out = mapper.apply(&input).unwrap();
    assert!(out.get("firstName").is_some());
    assert!(out.get("first_name").is_none());
}

#[test]
fn test_object_schema_builder_valid() {
    let s = ObjectSchema::new()
        .field("name", StringSchema::new().required().min_length(2))
        .field("age", IntegerSchema::new().required().min(0));
    let v = make_obj(&[
        ("name", Value::String("alice".into())),
        ("age", Value::Int(30)),
    ]);
    assert!(s.validate(&v, "root").is_ok());
}

#[test]
fn test_object_schema_builder_invalid() {
    let s = ObjectSchema::new().field("name", StringSchema::new().required().min_length(5));
    let v = make_obj(&[("name", Value::String("ab".into()))]);
    assert!(s.validate(&v, "root").is_err());
}

#[test]
fn test_string_schema_email_validator() {
    let s = Schema::String(Schema::string().required().email());
    assert!(s.validate(&Value::String("a@b.com".into()), "e").is_ok());
    assert!(
        s.validate(&Value::String("notanemail".into()), "e")
            .is_err()
    );
}

#[test]
fn test_array_schema_validates_items() {
    let s = ArraySchema::new(StringSchema::new().email());
    let good = Value::Array(vec![Value::String("a@b.com".into())]);
    let bad = Value::Array(vec![Value::String("not-email".into())]);
    assert!(s.validate(&good, "emails").is_ok());
    assert!(s.validate(&bad, "emails").is_err());
}

#[test]
fn test_optional_field_absent() {
    let p = Profile::from_json(r#"{"name":"Alice"}"#).unwrap();
    assert!(p.bio.is_none());
}

#[test]
fn test_optional_field_present() {
    let p = Profile::from_json(r#"{"name":"Alice","bio":"Hello"}"#).unwrap();
    assert_eq!(p.bio, Some("Hello".to_string()));
}

#[test]
fn test_optional_field_explicit_null() {
    let p = Profile::from_json(r#"{"name":"Alice","bio":null}"#).unwrap();
    assert!(p.bio.is_none());
}

#[test]
fn test_default_value_applied_when_missing() {
    let p = Profile::from_json(r#"{"name":"Alice"}"#).unwrap();
    assert_eq!(p.country, "India");
    assert_eq!(p.score, 0);
}

#[test]
fn test_default_value_not_applied_when_present() {
    let p = Profile::from_json(r#"{"name":"Alice","country":"USA","score":100}"#).unwrap();
    assert_eq!(p.country, "USA");
    assert_eq!(p.score, 100);
}

#[test]
fn test_strict_mode_rejects_string_for_int() {
    let s = IntegerSchema::new().strict().required();
    assert!(s.validate(&Value::String("42".into()), "n").is_err());
}

#[test]
fn test_strict_mode_rejects_int_for_string() {
    let s = StringSchema::new().strict().required();
    assert!(s.validate(&Value::Int(42), "s").is_err());
}

#[test]
fn test_non_strict_coerces_string_to_int() {
    let s = IntegerSchema::new().min(0);
    assert!(s.validate(&Value::String("42".into()), "n").is_ok());
}

#[test]
fn test_json_parse_all_types() {
    assert_eq!(parse("null").unwrap(), Value::Null);
    assert_eq!(parse("true").unwrap(), Value::Bool(true));
    assert_eq!(parse("false").unwrap(), Value::Bool(false));
    assert_eq!(parse("42").unwrap(), Value::Int(42));
    assert_eq!(parse("1.23").unwrap(), Value::Float(1.23));
    assert_eq!(parse(r#""hi""#).unwrap(), Value::String("hi".into()));
    assert!(parse("[]").unwrap().is_array());
    assert!(parse("{}").unwrap().is_object());
}

#[test]
fn test_json_roundtrip_nested() {
    let input = r#"{"user":{"age":30,"name":"alice"}}"#;
    let v = parse(input).unwrap();
    let out = stringify(&v).unwrap();
    assert_eq!(parse(&out).unwrap(), v);
}

#[test]
fn test_json_unicode_escape() {
    let v = parse(r#""\u0041\u0042\u0043""#).unwrap();
    assert_eq!(v, Value::String("ABC".into()));
}

#[test]
fn test_json_number_int_vs_float() {
    assert!(matches!(parse("10").unwrap(), Value::Int(10)));
    assert!(matches!(parse("10.5").unwrap(), Value::Float(_)));
}

#[test]
fn test_json_malformed_returns_error() {
    assert!(parse("{bad}").is_err());
    assert!(parse("[1,2,").is_err());
    assert!(parse("").is_err());
}

#[test]
fn test_json_empty_object_array() {
    assert_eq!(parse("{}").unwrap(), Value::Object(BTreeMap::new()));
    assert_eq!(parse("[]").unwrap(), Value::Array(vec![]));
}

#[test]
fn test_json_stringify_pretty() {
    let mut m = BTreeMap::new();
    m.insert("a".to_string(), Value::Int(1));
    let v = Value::Object(m);
    let pretty = stringify_pretty(&v).unwrap();
    assert!(pretty.contains('\n'));
    assert!(pretty.contains("  \"a\""));
}

#[test]
fn test_value_coerce_to_int() {
    assert_eq!(Value::String("42".into()).coerce_to_int(), Some(42));
    assert_eq!(Value::Float(3.0).coerce_to_int(), Some(3));
    assert_eq!(Value::Float(3.5).coerce_to_int(), None);
}

#[test]
fn test_value_coerce_to_string() {
    assert_eq!(Value::Int(7).coerce_to_string(), Some("7".into()));
    assert_eq!(Value::Bool(true).coerce_to_string(), Some("true".into()));
}

#[test]
fn test_value_from_impls() {
    let _: Value = 42i64.into();
    let _: Value = 1.23f64.into();
    let _: Value = "hello".into();
    let _: Value = true.into();
    let v: Value = (None as Option<Value>).into();
    assert!(v.is_null());
}

#[test]
fn test_value_get_nested() {
    let mut inner = BTreeMap::new();
    inner.insert("city".to_string(), Value::String("Berlin".into()));
    let mut outer = BTreeMap::new();
    outer.insert("address".to_string(), Value::Object(inner));
    let root = Value::Object(outer);
    assert_eq!(
        root.get("address").and_then(|a| a.get("city")),
        Some(&Value::String("Berlin".into()))
    );
}

#[test]
fn test_validation_error_display() {
    let e = ValidationError::new("email", "invalid email", "email");
    let s = e.to_string();
    assert!(s.contains("email"));
    assert!(s.contains("invalid email"));
}

#[test]
fn test_error_from_conversions() {
    let ve = ValidationError::new("f", "m", "c");
    let e: Error = ve.into();
    assert!(matches!(e, Error::Validation(_)));

    let je = adapters::error::JsonError::new("bad json");
    let e2: Error = je.into();
    assert!(matches!(e2, Error::Json(_)));
}

#[test]
fn test_nested_model_validation_fails() {
    let json = r#"{"username":"alice","address":{"country":"Germany"}}"#;
    let res = UserWithAddress::from_json(json);
    assert!(res.is_err());
    let e = res.unwrap_err().to_string();
    assert!(
        e.contains("address.city") || e.contains("required"),
        "error: {e}"
    );
}

#[derive(Schema, Debug)]
struct ExtraMacroFeatures {
    #[schema(non_empty, alphanumeric)]
    code: String,
    #[schema(positive)]
    pos_int: i32,
    #[schema(negative)]
    neg_float: f32,
    #[schema(non_zero)]
    nonzero: i64,
}

#[test]
fn test_extra_macro_validations_success() {
    let json = r#"{
        "code": "ABC123xyz",
        "pos_int": 42,
        "neg_float": -1.23,
        "nonzero": -100
    }"#;
    let res = ExtraMacroFeatures::from_json(json);
    assert!(res.is_ok(), "Expected success, got: {:?}", res.err());
}

#[test]
fn test_extra_macro_validations_fail_empty() {
    let json = r#"{
        "code": "",
        "pos_int": 42,
        "neg_float": -1.23,
        "nonzero": -100
    }"#;
    let res = ExtraMacroFeatures::from_json(json);
    assert!(res.is_err());
}

#[test]
fn test_extra_macro_validations_fail_not_alphanumeric() {
    let json = r#"{
        "code": "abc-123",
        "pos_int": 42,
        "neg_float": -1.23,
        "nonzero": -100
    }"#;
    let res = ExtraMacroFeatures::from_json(json);
    assert!(res.is_err());
}

#[test]
fn test_extra_macro_validations_fail_negative_pos_int() {
    let json = r#"{
        "code": "ABC123xyz",
        "pos_int": -5,
        "neg_float": -1.23,
        "nonzero": -100
    }"#;
    let res = ExtraMacroFeatures::from_json(json);
    assert!(res.is_err());
}

#[test]
fn test_extra_macro_validations_fail_positive_neg_float() {
    let json = r#"{
        "code": "ABC123xyz",
        "pos_int": 42,
        "neg_float": 1.23,
        "nonzero": -100
    }"#;
    let res = ExtraMacroFeatures::from_json(json);
    assert!(res.is_err());
}

#[test]
fn test_extra_macro_validations_fail_zero_nonzero() {
    let json = r#"{
        "code": "ABC123xyz",
        "pos_int": 42,
        "neg_float": -1.23,
        "nonzero": 0
    }"#;
    let res = ExtraMacroFeatures::from_json(json);
    assert!(res.is_err());
}

// Custom validation function for macro test
fn validate_even_number(value: &Value, field: &str) -> Result<(), Error> {
    if let Some(n) = value.as_int() {
        if n % 2 == 0 {
            Ok(())
        } else {
            Err(Error::Validation(ValidationError::new(
                field,
                "must be an even number",
                "even_number",
            )))
        }
    } else {
        Ok(())
    }
}

#[derive(SchemaDerived, Debug)]
struct CustomValidStruct {
    #[schema(custom = validate_even_number)]
    even_value: i32,
}

#[test]
fn test_custom_macro_validator_success() {
    let json = r#"{"even_value": 4}"#;
    let res = CustomValidStruct::from_json(json);
    assert!(res.is_ok(), "Deserialization should succeed");
    let instance = res.unwrap();
    assert!(
        instance.validate().is_ok(),
        "Expected even value to validate successfully"
    );
}

#[test]
fn test_custom_macro_validator_failure() {
    let json = r#"{"even_value": 5}"#;
    let res = CustomValidStruct::from_json(json);
    assert!(res.is_ok(), "Deserialization should succeed");
    let instance = res.unwrap();
    assert!(
        instance.validate().is_err(),
        "Expected odd value to fail self-validation"
    );
}

#[derive(SchemaDerived, Debug)]
struct UrlVerifiedStruct {
    #[schema(url)]
    website: String,
}

#[test]
fn test_url_macro_validator_success() {
    let json = r#"{"website": "https://muhammadfiaz.com"}"#;
    let res = UrlVerifiedStruct::from_json(json);
    assert!(res.is_ok(), "Expected valid URL to validate successfully");
}

#[test]
fn test_url_macro_validator_failure() {
    let json = r#"{"website": "not-a-valid-url"}"#;
    let res = UrlVerifiedStruct::from_json(json);
    assert!(res.is_err(), "Expected invalid URL to fail validation");
}

#[test]
fn test_programmatic_custom_validator_success() {
    use adapters::validator::{CustomValidator, ValidatorFn};
    let validator = CustomValidator {
        func: Box::new(|val, field| {
            if val.as_str() == Some("special-token") {
                Ok(())
            } else {
                Err(ValidationError::new(
                    field,
                    "invalid special token",
                    "token_mismatch",
                ))
            }
        }),
    };
    let pass = Value::String("special-token".into());
    let fail = Value::String("other-token".into());
    assert!(validator.validate(&pass, "token").is_ok());
    assert!(validator.validate(&fail, "token").is_err());
}
