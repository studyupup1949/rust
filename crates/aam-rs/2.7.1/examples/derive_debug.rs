use aam_rs::aam::AAM;
use aam_rs::builder::AAMBuilder;
use std::fs;

fn main() {
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
            aam_rs::builder::SchemaField::required("id", "i32"),
            aam_rs::builder::SchemaField::required("name", "string"),
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

    println!(
        "Base file content:\n---\n{}\n---",
        fs::read_to_string(&file_path).unwrap()
    );
    println!(
        "Child file content:\n---\n{}\n---",
        fs::read_to_string(&child_path).unwrap()
    );

    let doc = AAM::load(&child_path);
    println!("AAM::load result: {:?}", doc);
}
