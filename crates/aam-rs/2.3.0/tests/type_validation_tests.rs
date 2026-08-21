use aam_rs::aam::AAM;
use aam_rs::builder::{AAMBuilder, SchemaField};

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

#[test]
fn derive_after_variable() {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let base_path = dir.path();

    let file_path = base_path.join("schema.aam");
    let child_path = base_path.join("derive.aam");

    let mut builder = AAMBuilder::new();
    builder.schema(
        "Device",
        vec![
            SchemaField::required("id", "i32"),
            SchemaField::required("name", "string"),
        ],
    );
    builder
        .to_file(file_path.clone())
        .expect("Failed to write AAM file");
    let mut builder = AAMBuilder::new();
    builder.add_line("id", "42");
    builder.derive(file_path.to_str().unwrap(), vec!["Device"]);
    builder
        .to_file(child_path.clone())
        .expect("Failed to write AAM file");
    let doc = AAM::load(&child_path);
    if doc.is_err() {
        println!("{:?}", doc.err());
    } else {
        assert!(doc.is_err());
    }
}

#[test]
fn schema_validation_in_one_file() {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let base_path = dir.path();

    let file_path = base_path.join("schema_validation.aam");
    let mut builder = AAMBuilder::new();

    builder.schema(
        "Device",
        vec![
            SchemaField::required("id", "i32"),
            SchemaField::required("name", "string"),
        ],
    );
    builder
        .to_file(file_path.clone())
        .expect("Failed to write AAM file");
    let doc = AAM::load(&file_path);
    println!("Error details: {:?}", doc);
    assert!(doc.is_ok());
}

#[test]
fn schema_invalid_syntax() {
    let content = r#"
    @schema Device {
        id: i32
        name: string
        ls
    }
    "#;
    assert!(AAM::parse(content).is_err());
}

#[test]
fn schema_fields_are_not_provided() {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let base_path = dir.path();

    let file_path = base_path.join("schema_validation_2.aam");
    let second_file_path = base_path.join("schema_validation_2_child.aam");

    let mut builder = AAMBuilder::new();
    builder.schema_multiline(
        "Device",
        vec![
            SchemaField::required("id", "i32"),
            SchemaField::required("name", "string"),
        ],
    );
    builder
        .to_file(file_path.clone())
        .expect("Failed to write AAM file");
    let mut builder = AAMBuilder::new();
    builder.derive(file_path.to_str().unwrap(), vec!["Device"]);
    builder.add_line("id", "42");
    builder
        .to_file(second_file_path.clone())
        .expect("Failed to write AAM file");
    let doc = AAM::load(&second_file_path);
    if doc.is_err() {
        println!("{:?}", doc.err());
    } else {
        assert!(doc.is_err());
    }
}

#[test]
fn schema_fields_are_not_provided_in_child_but_provided_in_parent() {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let base_path = dir.path();

    let parent_file_path = base_path.join("schema_validation_3.aam");
    let child_file_path = base_path.join("schema_validation_3_child.aam");

    let mut builder = AAMBuilder::new();
    builder.schema_multiline(
        "Device",
        vec![
            SchemaField::required("id", "i32"),
            SchemaField::required("name", "string"),
        ],
    );
    builder.add_line("name", "OK");
    builder
        .to_file(parent_file_path.clone())
        .expect("Failed to write AAM file");
    let mut builder = AAMBuilder::new();
    builder.derive(parent_file_path.to_str().unwrap(), vec!["Device"]);
    builder.add_line("id", "42");
    builder
        .to_file(child_file_path.clone())
        .expect("Failed to write AAM file");
    let doc = AAM::load(&child_file_path);
    if doc.is_err() {
        println!("{:?}", doc.err());
    } else {
        assert!(doc.is_err());
    }
}

#[test]
fn derive_not_imports() {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let base_path = dir.path();

    let file_path = base_path.join("derive_4.aam");
    let child_path = base_path.join("derive_4_child.aam");

    let mut builder = AAMBuilder::new();
    builder.add_line("id", "42");
    builder.add_line("name", "OK");
    builder.schema_multiline(
        "Device",
        vec![
            SchemaField::required("id", "i32"),
            SchemaField::required("name", "string"),
        ],
    );
    builder
        .to_file(file_path.clone())
        .expect("Failed to write AAM file");
    let mut builder = AAMBuilder::new();
    builder.derive(file_path.to_str().unwrap(), vec!["Device"]);
    builder
        .to_file(child_path.clone())
        .expect("Failed to write AAM file");
    let doc = AAM::load(&child_path);
    if doc.is_err() {
        println!("{:?}", doc.err());
    } else {
        assert!(doc.is_err());
    }
}
