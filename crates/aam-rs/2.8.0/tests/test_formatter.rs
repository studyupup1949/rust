use aam_rs::aam::AAM;
use aam_rs::builder::AAMBuilder;
use aam_rs::pipeline::FormattingOptions;
use tempfile::tempdir;

#[test]
fn test_formatter() {
    let dir = tempdir().expect("Failed to create temp dir");
    let base_path = dir.path();

    let file_name = "file.aam";
    let file_path = base_path.join(file_name);

    let mut b = AAMBuilder::new();
    b.add_line("b", "c");
    b.to_file(&file_path).expect("Failed to write AAM file");

    let source = format!(
        r#"
        a        = b
        @schema Device {{
        ok: string
        not_ok: string
        }}
        @type port = i32
        @import {}
    "#,
        file_path.display()
    );

    let doc = AAM::parse(&source).expect("Failed to parse AAM content");
    let formatted = doc
        .format(&source, &FormattingOptions::default())
        .expect("Failed to format");

    let expected = format!(
        r#"@import {}

a = b
@schema Device {{ ok: string, not_ok: string }}
@type port = i32
"#,
        file_path.display()
    );

    assert_eq!(
        formatted, expected,
        "Formatted output does not match expected"
    );
}
