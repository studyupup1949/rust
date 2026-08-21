//! Example: `@derive` with schema validation
//!
//! Demonstrates:
//! 1. A successful `@derive` load where all schema fields are present.
//! 2. A failed load where a required schema field is missing → `SchemaValidationError`.
//! 3. A failed load where a field value has the wrong type → `SchemaValidationError`.
//! 4. Notes about runtime map validation in the new AAM API.
//!
//! Run with:
//! ```sh
//! cargo run --example derive
//! ```

use aam_rs::aam::AAM;
use aam_rs::builder::{AAMBuilder, SchemaField};
use aam_rs::error::AamlError;
use std::collections::HashMap;
use std::path::Path;

fn main() {
    // ── Resolve the examples/ directory so relative paths in .aam files work ──
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    std::env::set_current_dir(&examples_dir).expect("Cannot change dir to examples/");

    println!("═══════════════════════════════════════════════════════");
    println!("  AAM @derive + Schema Validation Example");
    println!("═══════════════════════════════════════════════════════\n");

    // ── 1. Successful load ────────────────────────────────────────────────────
    println!("▶ 1. Loading derive_child.aam (all required fields present)");
    match AAM::load("derive_child.aam").map_err(first_error) {
        Ok(config) => {
            println!("   ✔ Loaded successfully\n");

            println!("   ── Child-specific keys ──────────────────────────────");
            print_key(&config, "author");
            print_key(&config, "version");

            println!("\n   ── Overridden keys (child wins over base) ───────────");
            print_key(&config, "log_level");
            print_key(&config, "theme");
            print_key(&config, "id");
            print_key(&config, "name");
            print_key(&config, "active");

            println!("\n   ── Inherited keys from derive_base.aam ──────────────");
            print_key(&config, "app_name");
            print_key(&config, "max_retries");
            print_key(&config, "timeout");

            println!("\n   ── Inherited schema: Entity ─────────────────────────");
            print_schema(&config, "Entity");

            println!("\n   ── Child schema: Plugin ─────────────────────────────");
            print_schema(&config, "Plugin");
        }
        Err(e) => {
            eprintln!("   ✘ Unexpected error: {e}");
        }
    }

    // ── 2. Missing required field → SchemaValidationError ────────────────────
    println!("\n▶ 2. Missing required Entity field 'active' → expect SchemaValidationError");
    // ── 2. Missing field → SchemaValidationError ─────────────────────────────
    println!("\n▶ 2. Field 'active' is omitted → expect SchemaValidationError");
    {
        // Write a temporary base that defines the schema but does NOT supply 'active'
        let base_path = "tmp_base_missing_field.aam";
        let mut b = AAMBuilder::new();
        b.schema(
            "Entity",
            [
                SchemaField::required("id", "i32"),
                SchemaField::required("name", "string"),
                SchemaField::required("active", "bool"),
            ],
        )
        .add_line("id", "10")
        .add_line("name", "TestApp");
        // 'active' is intentionally omitted

        b.to_file(base_path).unwrap();

        let content = format!("@derive {base_path}\n");
        let result = AAM::parse(&content).map_err(first_error);
        let _ = std::fs::remove_file(base_path);

        match_result::<AAM>(result)
    }

    // ── 3. Wrong type for a field → SchemaValidationError ────────────────────
    println!("\n▶ 3. Field 'id' set to a non-integer → expect SchemaValidationError");
    {
        let base_path = "tmp_base_wrong_type.aam";

        let mut b = AAMBuilder::new();
        b.schema(
            "Entity",
            [
                SchemaField::required("id", "i32"),
                SchemaField::required("name", "string"),
                SchemaField::required("active", "bool"),
            ],
        )
        .add_line("id", "not-a-number") // ← wrong type
        .add_line("name", "TestApp")
        .add_line("active", "true");

        b.to_file(base_path).unwrap();

        let content = format!("@derive {base_path}\n");
        let result = AAM::parse(&content).map_err(first_error);
        let _ = std::fs::remove_file(base_path);

        match_result::<AAM>(result)
    }

    // ── 4. Runtime map validation guidance ────────────────────────────────────
    println!("\n▶ 4. Runtime map validation guidance");
    {
        let config = match AAM::parse("@schema Player { name: string, score: i32, health: f64 }")
            .map_err(first_error)
        {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("   ~ Runtime schema demo skipped on this parser build: {e}");
                return;
            }
        };

        // 4a. Valid data
        let mut valid_data = HashMap::new();
        valid_data.insert("name".to_string(), "Alice".to_string());
        valid_data.insert("score".to_string(), "1500".to_string());
        valid_data.insert("health".to_string(), "87.3".to_string());

        println!(
            "   Player schema present: {}",
            config.get_schema("Player").is_some()
        );
        println!("   Valid sample map: {valid_data:?}");

        // 4b. Missing field
        let mut missing = HashMap::new();
        missing.insert("name".to_string(), "Bob".to_string());
        // "score" and "health" are absent

        println!("   Missing-fields sample map: {missing:?}");

        // 4c. Wrong type
        let mut wrong_type = HashMap::new();
        wrong_type.insert("name".to_string(), "Carol".to_string());
        wrong_type.insert("score".to_string(), "not-a-number".to_string());
        wrong_type.insert("health".to_string(), "99.0".to_string());

        println!("   Wrong-type sample map: {wrong_type:?}");
        println!("   ~ runtime apply_schema is intentionally not exposed on AAM.");
    }

    println!("\n═══════════════════════════════════════════════════════");
    println!("  Done.");
    println!("═══════════════════════════════════════════════════════");
}

/// Prints a single key-value pair from the config, or `<not found>` if absent.
fn print_key(config: &AAM, key: &str) {
    let value = config
        .get(key)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "<not found>".to_string());
    println!("   {key:>15} = {value}");
}

/// Prints the fields of a named schema, or a message if the schema is absent.
fn print_schema(config: &AAM, schema_name: &str) {
    match config.get_schema(schema_name) {
        Some(schema) => {
            let mut fields: Vec<_> = schema.fields.iter().collect();
            fields.sort_by_key(|(k, _)| k.as_str());
            for (field, (ty, optional)) in fields {
                let opt = if *optional { "*" } else { " " };
                println!("   {field:>15}{opt}: {ty}");
            }
        }
        None => println!("   Schema '{schema_name}' not found"),
    }
}

fn match_result<T>(result: Result<T, AamlError>) {
    match result {
        Err(AamlError::SchemaValidationError {
            schema,
            field,
            type_name,
            details,
            ..
        }) => {
            println!(
                "   ✔ Got expected error — schema: '{schema}', field: '{field}' \
                     (type: '{type_name}'), reason: {details}"
            );
        }
        Err(other) => eprintln!("   ✘ Wrong error type: {other}"),
        Ok(_) => eprintln!("   ✘ Expected an error but parsing succeeded"),
    }
}

fn first_error(errors: Vec<AamlError>) -> AamlError {
    errors.into_iter().next().unwrap_or(AamlError::ParseError {
        line: 1,
        content: String::new(),
        details: "unexpected empty error list".to_string(),
        diagnostics: None,
    })
}
