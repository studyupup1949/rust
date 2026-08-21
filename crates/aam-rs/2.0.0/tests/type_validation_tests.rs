use aam_rs::aam::AAM;

#[test]
fn parse_accepts_valid_builtin_types() {
    let content = "@schema Device { id: i32, active: bool, ratio: f64 }\nid = 42\nactive = true\nratio = 3.14";
    let parsed = AAM::parse(content);
    assert!(parsed.is_ok(), "Expected Ok, got: {:?}", parsed.err());
}

#[test]
fn parse_rejects_invalid_i32() {
    let content = "@schema Device { id: i32 }\nid = not_a_number";
    assert!(AAM::parse(content).is_err());
}

#[test]
fn parse_accepts_optional_schema_fields() {
    let content = "@schema Server { host: string, port*: i32 }\nhost = localhost";
    let parsed = AAM::parse(content);
    assert!(parsed.is_ok(), "Expected Ok, got: {:?}", parsed.err());
}

#[test]
fn parse_rejects_unknown_type_in_schema() {
    let content = "@schema Device { id: unknown_type }\nid = 42";
    assert!(AAM::parse(content).is_err());
}

#[test]
fn parse_rejects_unknown_type_in_type_alias() {
    let content = "@type DeviceId = unknown_type\n@schema Device { id: DeviceId }\nid = 42";
    assert!(AAM::parse(content).is_err());
}

#[test]
fn parse_rejects_missing_required_schema_field() {
    let content = "@schema Device { id: i32, name: string }\nid = 42";
    assert!(AAM::parse(content).is_err());
}

#[test]
fn parse_rejects_invalid_f64() {
    let content = "@schema Device { ratio: f64 }\nratio = not_a_float";
    assert!(AAM::parse(content).is_err());
}
