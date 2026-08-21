//! Example: `builder` — constructing AAML documents programmatically
//!
//! Shows how to use `AAMBuilder` to generate AAML content in code, then
//! parse it with `AAML::parse` and validate it against schemas.
//!
//! Sections:
//! 1. Simple key-value document.
//! 2. Document with a schema — required + optional fields.
//! 3. Document with a list field.
//! 4. Document with an inline nested object.
//! 5. Writing a generated document to a temporary file and loading it back.
//!
//! Run with:
//! ```sh
//! cargo run --example builder
//! ```

use aam_rs::aaml::AAML;
use aam_rs::builder::{AAMBuilder, SchemaField};
use aam_rs::error::AamlError;

fn main() {
    header("AAML Builder — programmatic document construction");

    section_1_simple_kv();
    section_2_schema_required_optional();
    section_3_list_field();
    section_4_inline_object();
    section_5_roundtrip_file();

    footer();
}

// ── 1. Simple key-value document ─────────────────────────────────────────────

fn section_1_simple_kv() {
    section("1. Simple key-value document");

    let mut b = AAMBuilder::new();
    b.add_line("host",    "localhost");
    b.add_line("port",    "5432");
    b.add_line("db_name", "my_database");
    b.add_line("user",    "admin");

    let content = b.build();
    println!("   Generated AAML:\n");
    for line in content.lines() {
        println!("     {line}");
    }
    println!();

    let cfg = AAML::parse(&content).expect("Parse failed");
    println!("   Parsed values:");
    for key in &["host", "port", "db_name", "user"] {
        print_key(&cfg, key);
    }
}

// ── 2. Schema with required and optional fields ───────────────────────────────

fn section_2_schema_required_optional() {
    section("2. Schema — required + optional fields");

    let mut b = AAMBuilder::new();
    b.schema_multiline("AppConfig", [
        SchemaField::required("app_name",    "string"),
        SchemaField::required("max_retries", "i32"),
        SchemaField::required("timeout",     "f64"),
        SchemaField::optional("log_level",   "string"),
        SchemaField::optional("debug",       "bool"),
    ]);
    // Required fields only (optional ones deliberately omitted)
    b.add_line("app_name",    "MyService");
    b.add_line("max_retries", "3");
    b.add_line("timeout",     "30.0");

    let content = b.build();
    println!("   Generated AAML:\n");
    for line in content.lines() {
        println!("     {line}");
    }
    println!();

    match AAML::parse(&content) {
        Ok(cfg) => {
            println!("   ✔ Parsed — optional fields absent, no error\n");
            println!("   Schema 'AppConfig':");
            print_schema(&cfg, "AppConfig");
            println!("   Values:");
            for key in &["app_name", "max_retries", "timeout", "log_level", "debug"] {
                print_key(&cfg, key);
            }
        }
        Err(e) => eprintln!("   ✘ Unexpected error: {e}"),
    }
}

// ── 3. Document with a list field ─────────────────────────────────────────────

fn section_3_list_field() {
    section("3. list<string> and list<i32> fields");

    let mut b = AAMBuilder::new();
    b.schema_multiline("Route", [
        SchemaField::required("path",    "string"),
        SchemaField::required("methods", "list<string>"),
        SchemaField::required("codes",   "list<i32>"),
        SchemaField::optional("tags",    "list<string>"),
    ]);
    b.add_line("path",    "/api/users");
    b.add_line("methods", "[GET, POST, DELETE]");
    b.add_line("codes",   "[200, 201, 204, 400, 404]");
    // tags* omitted

    let content = b.build();
    match AAML::parse(&content) {
        Ok(cfg) => {
            println!("   ✔ Parsed\n");
            println!("   Schema 'Route':");
            print_schema(&cfg, "Route");
            println!("   Values:");
            for key in &["path", "methods", "codes", "tags"] {
                print_key(&cfg, key);
            }
        }
        Err(e) => eprintln!("   ✘ {e}"),
    }
}

// ── 4. Inline nested object ───────────────────────────────────────────────────

fn section_4_inline_object() {
    section("4. Inline nested object — Address inside Server");

    let mut b = AAMBuilder::new();
    b.schema("Address", [
        SchemaField::required("host", "string"),
        SchemaField::required("port", "i32"),
        SchemaField::optional("tls",  "bool"),
    ]);
    b.schema("Server", [
        SchemaField::required("name",    "string"),
        SchemaField::required("address", "Address"),
        SchemaField::required("workers", "i32"),
    ]);
    b.add_line("name",    "ApiGateway");
    b.add_line("workers", "8");
    // Inline object for 'address' field — validated against Address schema
    b.add_line("address", "{ host = gateway.example.com, port = 8443, tls = true }");

    let content = b.build();
    match AAML::parse(&content) {
        Ok(cfg) => {
            println!("   ✔ Parsed\n");
            println!("   Schemas:");
            print_schema(&cfg, "Address");
            print_schema(&cfg, "Server");
            println!("   Values:");
            for key in &["name", "workers", "address"] {
                print_key(&cfg, key);
            }
        }
        Err(e) => eprintln!("   ✘ {e}"),
    }
}

// ── 5. Roundtrip: generate → write file → load file ──────────────────────────

fn section_5_roundtrip_file() {
    section("5. Roundtrip — write AAML to file, then load with AAML::load");

    let tmp_path = "tmp_builder_roundtrip.aam";

    let mut b = AAMBuilder::new();
    b.schema("Plugin", [
        SchemaField::required("plugin_name", "string"),
        SchemaField::required("enabled",     "bool"),
        SchemaField::optional("priority",    "i32"),
    ]);
    b.add_line("plugin_name", "AuthPlugin");
    b.add_line("enabled",     "true");
    // priority* omitted

    match b.to_file(tmp_path) {
        Ok(()) => println!("   ✔ Written to {tmp_path}"),
        Err(e) => { eprintln!("   ✘ Write error: {e}"); return; }
    }

    match AAML::load(tmp_path) {
        Ok(cfg) => {
            println!("   ✔ Loaded back from {tmp_path}\n");
            println!("   Schema 'Plugin':");
            print_schema(&cfg, "Plugin");
            println!("   Values:");
            for key in &["plugin_name", "enabled", "priority"] {
                print_key(&cfg, key);
            }
        }
        Err(e) => eprintln!("   ✘ Load error: {e}"),
    }

    // Clean up
    let _ = std::fs::remove_file(tmp_path);
    println!("\n   ✔ Temporary file removed");

    // 5b. Intentional error: required field missing — detected via completeness check
    println!("\n   5b. Missing required 'enabled' → expect SchemaValidationError (completeness check)");
    let mut b2 = AAMBuilder::new();
    b2.schema("Plugin", [
        SchemaField::required("plugin_name", "string"),
        SchemaField::required("enabled",     "bool"),
        SchemaField::optional("priority",    "i32"),
    ]);
    b2.add_line("plugin_name", "BrokenPlugin");
    // enabled is intentionally omitted

    match AAML::parse(&b2.build()).and_then(|cfg| { cfg.validate_schemas_completeness()?; Ok(cfg) }) {
        Err(AamlError::SchemaValidationError { schema, field, details, .. }) =>
            println!("   ✔ Schema '{schema}', field '{field}': {details}"),
        other => eprintln!("   ✘ Unexpected result: {other:?}"),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn print_key(cfg: &AAML, key: &str) {
    match cfg.find_obj(key) {
        Some(v) => println!("   {key:>15} = {v}"),
        None    => println!("   {key:>15} = <not set>"),
    }
}

fn print_schema(cfg: &AAML, name: &str) {
    match cfg.get_schema(name) {
        Some(s) => {
            let mut fields: Vec<_> = s.fields.iter().collect();
            fields.sort_by_key(|(k, _)| k.as_str());
            for (field, ty) in &fields {
                let opt = if s.is_optional(field) { "*" } else { " " };
                println!("     {opt} {field:<20} : {ty}");
            }
            println!();
        }
        None => println!("   Schema '{name}' not found\n"),
    }
}

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

