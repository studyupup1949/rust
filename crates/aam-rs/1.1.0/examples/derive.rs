//! Example: `@derive` with schema validation
//!
//! Demonstrates:
//! 1. A successful `@derive` load where all schema fields are present.
//! 2. A failed load where a required schema field is missing → `SchemaValidationError`.
//! 3. A failed load where a field value has the wrong type → `SchemaValidationError`.
//! 4. Using `apply_schema` to validate an arbitrary data map at runtime.
//!
//! Run with:
//! ```sh
//! cargo run --example derive
//! ```

use aam_rs::aaml::AAML;
use aam_rs::builder::AAMBuilder;
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
    match AAML::load("derive_child.aam") {
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
    {
        // Write a temporary base that defines the schema but does NOT supply 'active'
        let base_path = "tmp_base_missing_field.aam";
        let mut b = AAMBuilder::new();
        b.add_raw("@schema Entity { id: i32, name: string, active: bool }");
        b.add_line("id", "10");
        b.add_line("name", "TestApp");
        // 'active' is intentionally omitted
        b.to_file(base_path).unwrap();

        let content = format!("@derive {base_path}\n");
        let result = AAML::parse(&content);
        let _ = std::fs::remove_file(base_path);

        match result {
            Err(AamlError::SchemaValidationError { schema, field, type_name, details }) => {
                println!(
                    "   ✔ Got expected error — schema: '{schema}', field: '{field}' \
                     (type: '{type_name}'), reason: {details}"
                );
            }
            Err(other) => eprintln!("   ✘ Wrong error type: {other}"),
            Ok(_) => eprintln!("   ✘ Expected an error but parsing succeeded"),
        }
    }

    // ── 3. Wrong type for a field → SchemaValidationError ────────────────────
    println!("\n▶ 3. Field 'id' set to a non-integer → expect SchemaValidationError");
    {
        let base_path = "tmp_base_wrong_type.aam";
        let mut b = AAMBuilder::new();
        b.add_raw("@schema Entity { id: i32, name: string, active: bool }");
        b.add_line("id", "not-a-number");   // ← wrong type
        b.add_line("name", "TestApp");
        b.add_line("active", "true");
        b.to_file(base_path).unwrap();

        let content = format!("@derive {base_path}\n");
        let result = AAML::parse(&content);
        let _ = std::fs::remove_file(base_path);

        match result {
            Err(AamlError::SchemaValidationError { schema, field, type_name, details }) => {
                println!(
                    "   ✔ Got expected error — schema: '{schema}', field: '{field}' \
                     (type: '{type_name}'), reason: {details}"
                );
            }
            Err(other) => eprintln!("   ✘ Wrong error type: {other}"),
            Ok(_) => eprintln!("   ✘ Expected an error but parsing succeeded"),
        }
    }

    // ── 4. apply_schema — validate an arbitrary data map ─────────────────────
    println!("\n▶ 4. apply_schema — explicit data-map validation");
    {
        let config = AAML::parse("@schema Player { name: string, score: i32, health: f64 }")
            .expect("Schema parse must succeed");

        // 4a. Valid data
        let mut valid_data = HashMap::new();
        valid_data.insert("name".to_string(), "Alice".to_string());
        valid_data.insert("score".to_string(), "1500".to_string());
        valid_data.insert("health".to_string(), "87.3".to_string());

        match config.apply_schema("Player", &valid_data) {
            Ok(()) => println!("   ✔ Valid data accepted by 'Player' schema"),
            Err(e) => eprintln!("   ✘ Unexpected rejection: {e}"),
        }

        // 4b. Missing field
        let mut missing = HashMap::new();
        missing.insert("name".to_string(), "Bob".to_string());
        // "score" and "health" are absent

        match config.apply_schema("Player", &missing) {
            Err(AamlError::SchemaValidationError { field, details, .. }) => {
                println!("   ✔ Missing field '{field}' caught — {details}");
            }
            other => eprintln!("   ✘ Unexpected result: {other:?}"),
        }

        // 4c. Wrong type
        let mut wrong_type = HashMap::new();
        wrong_type.insert("name".to_string(), "Carol".to_string());
        wrong_type.insert("score".to_string(), "not-a-number".to_string());
        wrong_type.insert("health".to_string(), "99.0".to_string());

        match config.apply_schema("Player", &wrong_type) {
            Err(AamlError::SchemaValidationError { field, type_name, details, .. }) => {
                println!("   ✔ Wrong type for '{field}' (expected {type_name}) caught — {details}");
            }
            other => eprintln!("   ✘ Unexpected result: {other:?}"),
        }
    }

    println!("\n═══════════════════════════════════════════════════════");
    println!("  Done.");
    println!("═══════════════════════════════════════════════════════");
}

/// Prints a single key-value pair from the config, or `<not found>` if absent.
fn print_key(config: &AAML, key: &str) {
    let value = config
        .find_obj(key)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "<not found>".to_string());
    println!("   {key:>15} = {value}");
}

/// Prints the fields of a named schema, or a message if the schema is absent.
fn print_schema(config: &AAML, schema_name: &str) {
    match config.get_schema(schema_name) {
        Some(schema) => {
            let mut fields: Vec<_> = schema.fields.iter().collect();
            fields.sort_by_key(|(k, _)| k.as_str());
            for (field, ty) in fields {
                println!("   {field:>15} : {ty}");
            }
        }
        None => println!("   Schema '{schema_name}' not found"),
    }
}
