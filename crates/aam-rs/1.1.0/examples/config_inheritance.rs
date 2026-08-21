//! Example: `config_inheritance` — real-world config inheritance via `@derive`
//!
//! Simulates the pattern of a shared production base config that a staging
//! environment overrides using `@derive file::Schema1::Schema2`.
//!
//! What you will see in the console:
//! - Base config loaded → all keys and schemas printed.
//! - Child config loaded → overridden keys, inherited keys, selective schemas.
//! - Inline validation of a partial data map against an inherited schema.
//!
//! Run with:
//! ```sh
//! cargo run --example config_inheritance
//! ```

use aam_rs::aaml::AAML;
use aam_rs::error::AamlError;
use std::collections::HashMap;
use std::path::Path;

fn main() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    std::env::set_current_dir(&examples_dir).expect("Cannot change dir to examples/");

    header("AAML — Config Inheritance with @derive");

    section_1_base();
    section_2_child();
    section_3_selective_two_schemas();
    section_4_apply_schema_runtime();
    section_5_error_cases();

    footer();
}

// ── 1. Base config ────────────────────────────────────────────────────────────

fn section_1_base() {
    section("1. Load config_base.aam — production base (all schemas)");

    match AAML::load("config_base.aam") {
        Ok(cfg) => {
            println!("   ✔ Loaded config_base.aam\n");
            for name in &["Address", "Database", "Server"] {
                print_schema_compact(&cfg, name);
            }
            divider();
            for key in &["app_env", "build_id", "tags", "feature_flags", "server", "database"] {
                print_key(&cfg, key);
            }
        }
        Err(e) => eprintln!("   ✘ {e}"),
    }
}

// ── 2. Child config — @derive base::Server::Database ─────────────────────────

fn section_2_child() {
    section("2. Load config_child.aam — @derive config_base.aam::Server::Database");
    println!("   Expected: Server ✔, Database ✔, Address ✗ (not imported)\n");

    match AAML::load("config_child.aam") {
        Ok(cfg) => {
            println!("   ✔ Loaded config_child.aam\n");
            println!("   Schema presence:");
            for (name, expected) in &[("Server", true), ("Database", true), ("Address", false)] {
                println!("     {name:>10} : {}", schema_marker(&cfg, name, *expected));
            }
            println!();
            divider();
            println!("   Override & staging-specific keys:");
            for key in &["app_env", "build_id", "deployed_by", "server", "database"] {
                print_key(&cfg, key);
            }
        }
        Err(e) => eprintln!("   ✘ Unexpected error: {e}"),
    }
}

// ── 3. Inline @derive selecting two schemas ───────────────────────────────────

fn section_3_selective_two_schemas() {
    section("3. Inline @derive config_base.aam::Server::Database  (no file)");
    println!("   Syntax: @derive <file>::<Schema1>::<Schema2>");
    println!("   Values must satisfy both schemas.\n");

    let content = concat!(
        "@derive config_base.aam::Server::Database\n",
        "server   = { name = InlineApp, version = 0.1, address = { host = localhost, port = 9000 }, allowed_ips = [127.0.0.1] }\n",
        "database = { driver = mysql, url = mysql://localhost/test, pool_min = 1, pool_max = 5 }\n",
        "app_env  = test\n",
    );

    match AAML::parse(content) {
        Ok(cfg) => {
            println!("   ✔ Parsed successfully\n");
            println!("   Schemas present after selective derive:");
            for (name, exp) in &[("Server", true), ("Database", true), ("Address", false)] {
                println!("     {name:>10} : {}", schema_marker(&cfg, name, *exp));
            }
            println!();
            for key in &["app_env", "server", "database"] {
                print_key(&cfg, key);
            }
        }
        Err(e) => eprintln!("   ✘ {e}"),
    }
}

// ── 4. apply_schema at runtime ────────────────────────────────────────────────

fn section_4_apply_schema_runtime() {
    section("4. apply_schema — validate arbitrary map at runtime");

    let cfg = AAML::parse(concat!(
        "@schema Server { name: string, version: string, address: Address, ",
        "allowed_ips: list<string>, debug*: bool }\n",
        "@schema Address { host: string, port: i32, tls*: bool }\n",
    )).expect("Schema parse failed");

    // 4a. Valid complete map
    let valid: HashMap<String, String> = [
        ("name".into(),        "ProdServer".into()),
        ("version".into(),     "2.0.0".into()),
        ("address".into(),     "{ host = prod.example.com, port = 443, tls = true }".into()),
        ("allowed_ips".into(), "[10.0.0.1, 10.0.0.2]".into()),
    ].into();
    validate_map(&cfg, "Server", &valid, "valid server map (no debug*) → ✔");

    // 4b. debug* included
    let mut with_debug = valid.clone();
    with_debug.insert("debug".into(), "false".into());
    validate_map(&cfg, "Server", &with_debug, "debug* = false               → ✔");

    // 4c. port wrong type
    let bad_port: HashMap<String, String> = [
        ("name".into(),        "BadServer".into()),
        ("version".into(),     "1.0".into()),
        ("address".into(),     "{ host = x, port = eighty }".into()),
        ("allowed_ips".into(), "[1.2.3.4]".into()),
    ].into();
    validate_map(&cfg, "Server", &bad_port, "port = 'eighty'              → ✘");

    // 4d. required field missing
    let missing_name: HashMap<String, String> = [
        ("version".into(),     "1.0".into()),
        ("address".into(),     "{ host = x, port = 80 }".into()),
        ("allowed_ips".into(), "[]".into()),
    ].into();
    validate_map(&cfg, "Server", &missing_name, "name missing                 → ✘");
}

// ── 5. Error cases ────────────────────────────────────────────────────────────

fn section_5_error_cases() {
    section("5. Error cases");

    // 5a. Derive non-existent schema
    println!("   5a. @derive config_base.aam::PhantomSchema → DirectiveError");
    match AAML::parse("@derive config_base.aam::PhantomSchema\n") {
        Err(AamlError::DirectiveError(cmd, msg)) =>
            println!("       ✔ @{cmd}: {msg}"),
        other => eprintln!("       ✘ Unexpected: {other:?}"),
    }

    // 5b. Schema field type mismatch at parse time
    println!("\n   5b. build_id = not-a-number  in @schema with i32 → SchemaValidationError");
    let src = "@schema Build { build_id: i32, env: string }\nbuild_id = not-a-number\nenv = prod\n";
    match AAML::parse(src) {
        Err(AamlError::SchemaValidationError { schema, field, type_name, details }) =>
            println!(
                "       ✔ schema '{schema}', field '{field}' ({type_name}): {details}"
            ),
        other => eprintln!("       ✘ Unexpected: {other:?}"),
    }

    // 5c. Required field missing — detected via validate_schemas_completeness()
    println!("\n   5c. Required field 'env' absent → SchemaValidationError (via completeness check)");
    let src2 = "@schema Build { build_id: i32, env: string }\nbuild_id = 7\n";
    match AAML::parse(src2).and_then(|cfg| { cfg.validate_schemas_completeness()?; Ok(cfg) }) {
        Err(AamlError::SchemaValidationError { schema, field, details, .. }) =>
            println!("       ✔ schema '{schema}', field '{field}': {details}"),
        other => eprintln!("       ✘ Unexpected: {other:?}"),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn validate_map(cfg: &AAML, schema: &str, data: &HashMap<String, String>, label: &str) {
    match cfg.apply_schema(schema, data) {
        Ok(()) =>
            println!("   ✔ {label}"),
        Err(AamlError::SchemaValidationError { field, type_name, details, .. }) =>
            println!("   ✔ {label}\n       ↳ '{field}' ({type_name}): {details}"),
        Err(e) =>
            eprintln!("   ✘ {label} — unexpected: {e}"),
    }
}

fn print_key(cfg: &AAML, key: &str) {
    match cfg.find_obj(key) {
        Some(v) => println!("   {key:>15} = {v}"),
        None    => println!("   {key:>15} = <not found>"),
    }
}

fn print_schema_compact(cfg: &AAML, name: &str) {
    match cfg.get_schema(name) {
        Some(s) => {
            let mut fields: Vec<_> = s.fields.iter().collect();
            fields.sort_by_key(|(k, _)| k.as_str());
            println!("   Schema '{name}':");
            for (field, ty) in &fields {
                let opt = if s.is_optional(field) { "*" } else { " " };
                println!("     {opt} {field:<20} : {ty}");
            }
            println!();
        }
        None => println!("   Schema '{name}' not found\n"),
    }
}

fn schema_marker(cfg: &AAML, name: &str, expect_present: bool) -> &'static str {
    match (cfg.get_schema(name).is_some(), expect_present) {
        (true,  true)  => "present  ✔",
        (false, false) => "absent   ✔",
        (true,  false) => "present  ✗ (unexpected!)",
        (false, true)  => "absent   ✗ (expected!)",
    }
}

fn divider() { println!("   {}", "─".repeat(52)); }

fn header(title: &str) {
    println!("\n{}", "═".repeat(64));
    println!("  {title}");
    println!("{}\n", "═".repeat(64));
}

fn section(title: &str) {
    println!("\n┌─{}─┐", "─".repeat(60));
    println!("│  {:<60}│", title);
    println!("└─{}─┘", "─".repeat(60));
}

fn footer() {
    println!("\n{}", "═".repeat(64));
    println!("  Done.");
    println!("{}", "═".repeat(64));
}

