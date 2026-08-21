use aam_rs::aam::AAM;
use aam_rs::builder::{AAMBuilder, InlineObject, SchemaField};
use aam_rs::found_value::FoundValue;
use std::collections::HashMap;

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

#[test]
fn test_inline_schema_validation() {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let base_path = dir.path();

    let file_path = base_path.join("config.aam");
    let child_path = base_path.join("re2.aam");

    let source = InlineObject::new()
        .with_field("dir", r#""/hyprland/build/re2""#)
        .with_field("branch", "main")
        .with_field("url", r#""https://github.com/google/re2.git""#);
    let build = InlineObject::new()
        .with_field("system", "cmake")
        .with_field("command", "cmake")
        .with_field(
            "args",
            r#"["-G", "Ninja", "-DCMAKE_INSTALL_PREFIX=/hyprland", "-DCMAKE_INSTALL_LIBDIR=lib64", "-DCMAKE_BUILD_TYPE=Release", "-DCMAKE_LINKER=mold", "-DBUILD_SHARED_LIBS=ON", "-B", "build"]"#,
        );
    let install = InlineObject::new()
        .with_field("commands", r#"["make install"]"#)
        .with_field("prefix", r#""/usr/local""#);
    let hooks = InlineObject::new()
        .with_field("pre_build", r#""""#)
        .with_field("post_build", r#""""#);

    let mut b = AAMBuilder::new();
    b.schema_multiline(
        "SourceInfo",
        vec![
            SchemaField::required("dir", "string"),
            SchemaField::required("branch", "string"),
            SchemaField::required("url", "string"),
        ],
    )
    .schema_multiline(
        "BuildInfo",
        vec![
            SchemaField::required("system", "string"),
            SchemaField::required("command", "string"),
            SchemaField::required("args", "list<string>"),
        ],
    )
    .schema_multiline(
        "InstallInfo",
        vec![
            SchemaField::required("commands", "list<string>"),
            SchemaField::required("prefix", "string"),
        ],
    )
    .schema_multiline(
        "Hooks",
        vec![
            SchemaField::required("pre_build", "string"),
            SchemaField::required("post_build", "string"),
        ],
    )
    .schema_multiline(
        "Project",
        vec![
            SchemaField::required("id", "string"),
            SchemaField::required("name", "string"),
            SchemaField::required("type", "string"),
            SchemaField::required("version", "string"),
            SchemaField::required("source", "SourceInfo"),
            SchemaField::required("build", "BuildInfo"),
            SchemaField::required("install", "InstallInfo"),
            SchemaField::required("hooks", "Hooks"),
            SchemaField::required("build_deps", "list<string>"),
            SchemaField::required("run_deps", "list<string>"),
        ],
    )
    .to_file(file_path.clone())
    .expect("Failed to write AAM file");
    let mut b = AAMBuilder::new();
    b.derive(file_path.to_str().unwrap(), vec!["Project"])
        .import(file_path.to_str().unwrap())
        .add_line("id", "re2")
        .add_line("name", "RE2")
        .add_line("type", "core")
        .add_line("version", "2025-11-05")
        .add_inline("source", &source)
        .add_inline("build", &build)
        .add_inline("install", &install)
        .add_inline("hooks", &hooks)
        .add_line("build_deps", "[ninja, cmake, mold, abseil-cpp]")
        .add_line("run_deps", "[abseil-cpp]")
        .to_file(child_path.clone())
        .expect("Failed to write AAM file");

    println!("{}", b.build());

    let doc = AAM::parse(child_path.to_str().unwrap());
    if let Err(ref errors) = doc {
        for err in errors {
            eprintln!("Parse error: {}", err);
        }
    }
    assert!(doc.is_ok(), "Expected successful parse with inline schemas");

    let aam = doc.unwrap();

    // Simple value assertions
    assert_eq!(aam.get("id"), Some("re2"));
    assert_eq!(aam.get("name"), Some("RE2"));
    assert_eq!(aam.get("type"), Some("core"));
    assert_eq!(aam.get("version"), Some("2025-11-05"));

    // Parse inline object values back into HashMap via FoundValue
    let source_map: HashMap<String, String> = FoundValue::new(aam.get("source").unwrap())
        .as_object()
        .expect("source should be a parseable inline object");
    assert_eq!(source_map.get("dir").unwrap(), "/hyprland/build/re2");
    assert_eq!(source_map.get("branch").unwrap(), "main");
    assert_eq!(
        source_map.get("url").unwrap(),
        "https://github.com/google/re2.git"
    );

    let build_map = FoundValue::new(aam.get("build").unwrap())
        .as_object()
        .expect("build should be a parseable inline object");
    assert_eq!(build_map.get("system").unwrap(), "cmake");
    assert_eq!(build_map.get("command").unwrap(), "cmake");

    let install_map = FoundValue::new(aam.get("install").unwrap())
        .as_object()
        .expect("install should be a parseable inline object");
    assert_eq!(install_map.get("prefix").unwrap(), "/usr/local");

    let hooks_map = FoundValue::new(aam.get("hooks").unwrap())
        .as_object()
        .expect("hooks should be a parseable inline object");
    assert_eq!(hooks_map.get("pre_build").unwrap(), "");

    // Verify schemas are registered
    assert!(aam.get_schema("Project").is_some());
    assert!(aam.get_schema("SourceInfo").is_some());
    assert!(aam.get_schema("BuildInfo").is_some());
    assert!(aam.get_schema("InstallInfo").is_some());
    assert!(aam.get_schema("Hooks").is_some());
}
