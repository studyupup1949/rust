//! Example: `schema_in_schema` — nested schemas, optional fields, lists
//!
//! Demonstrates every nested-schema feature in a single, self-contained run:
//!
//! 1. **Inline definition** — schemas declared directly in Rust source via `AAML::parse`.
//! 2. **Nested schema field** — `Address` used as a type inside `Server`.
//! 3. **Optional field** (`*`) — `debug*: bool` in `Server` may be omitted.
//! 4. **list\<string\>** — `allowed_ips` is a list of string values.
//! 5. **list\<Schema\>** — `list<Item>` for a loot array where each element
//!    is an inline object validated against the `Item` schema.
//! 6. **File-based load** — reading `config_base.aam` which mixes all the above.
//!
//! Run with:
//! ```sh
//! cargo run --example schema_in_schema
//! ```

use aam_rs::aaml::AAML;
use aam_rs::error::AamlError;
use std::collections::HashMap;
use std::path::Path;

fn main() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    std::env::set_current_dir(&examples_dir).expect("Cannot change dir to examples/");

    header("AAML — Nested Schemas, Optional Fields & Lists");

    section_1_inline_nested();
    section_2_optional_fields();
    section_3_list_of_primitives();
    section_4_list_of_schemas();
    section_5_file_based();

    footer();
}

// ── 1. Inline nested schema ───────────────────────────────────────────────────

fn section_1_inline_nested() {
    section("1. Nested schema: Server { address: Address }");
    
    let src = "
        @schema Address {
            host: string
            port: i32
            tls*: bool
        }
        @schema Server {
            name: string
            address: Address
            debug*: bool
        }";
    let cfg = AAML::parse(src).expect("Schema parse failed");

    print_schema(&cfg, "Address");
    print_schema(&cfg, "Server");

    // Valid — address matches Address schema, debug* is absent (optional)
    let mut ok: HashMap<String, String> = HashMap::new();
    ok.insert("name".into(), "WebServer".into());
    ok.insert("address".into(), "{ host = localhost, port = 8080 }".into());
    check(&cfg, "Server", &ok, "no debug* (optional)   → ✔");

    // Valid — debug* is provided
    ok.insert("debug".into(), "true".into());
    check(&cfg, "Server", &ok, "debug* = true          → ✔");

    // Invalid — port is a string instead of i32
    let mut bad: HashMap<String, String> = HashMap::new();
    bad.insert("name".into(), "BadServer".into());
    bad.insert("address".into(), "{ host = x, port = not-a-port }".into());
    check(&cfg, "Server", &bad, "port = not-a-port      → ✘");
}

// ── 2. Optional fields ────────────────────────────────────────────────────────

fn section_2_optional_fields() {
    section("2. Optional fields marked with *");


    let src = "
        @schema Config {
            app_name: string
            max_retries: i32
            log_level*: string
            debug*: bool
            timeout*: f64
        }";
    let cfg = AAML::parse(src).expect("Parse failed");
    print_schema(&cfg, "Config");

    // All required fields present, all optional absent
    let minimal: HashMap<String, String> = [
        ("app_name".into(),    "MyApp".into()),
        ("max_retries".into(), "3".into()),
    ].into();
    check(&cfg, "Config", &minimal, "only required fields    → ✔");

    // All fields present
    let full: HashMap<String, String> = [
        ("app_name".into(),    "MyApp".into()),
        ("max_retries".into(), "5".into()),
        ("log_level".into(),   "debug".into()),
        ("debug".into(),       "true".into()),
        ("timeout".into(),     "30.0".into()),
    ].into();
    check(&cfg, "Config", &full, "all fields present      → ✔");

    // Missing required field
    let missing: HashMap<String, String> = [
        ("app_name".into(), "Partial".into()),
        // max_retries intentionally omitted
    ].into();
    check(&cfg, "Config", &missing, "max_retries missing     → ✘");
}

// ── 3. list<primitive> ────────────────────────────────────────────────────────

fn section_3_list_of_primitives() {
    section("3. list<string> and list<i32>");


    let src = "
        @schema Profile {
            username: string
            tags: list<string>  
            scores: list<i32>
            nicknames*: list<string>
        }";
    let cfg = AAML::parse(src).expect("Parse failed");
    print_schema(&cfg, "Profile");

    let ok: HashMap<String, String> = [
        ("username".into(), "alice".into()),
        ("tags".into(),     "[rust, systems, embedded]".into()),
        ("scores".into(),   "[100, 250, 375]".into()),
        // nicknames* omitted
    ].into();
    check(&cfg, "Profile", &ok, "tags + scores, no nicknames* → ✔");

    let with_opt: HashMap<String, String> = [
        ("username".into(),  "bob".into()),
        ("tags".into(),      "[gamedev, audio]".into()),
        ("scores".into(),    "[42, 77]".into()),
        ("nicknames".into(), "[Bobby, B-man]".into()),
    ].into();
    check(&cfg, "Profile", &with_opt, "all including nicknames*     → ✔");

    let bad_score: HashMap<String, String> = [
        ("username".into(), "carol".into()),
        ("tags".into(),     "[tag]".into()),
        ("scores".into(),   "[1, two, 3]".into()),  // "two" is not i32
    ].into();
    check(&cfg, "Profile", &bad_score, "scores contains 'two'        → ✘");
}

// ── 4. list<Schema> ──────────────────────────────────────────────────────────

fn section_4_list_of_schemas() {
    section("4. list<Item> — each list element is validated as an Item object");

    // Inline schemas — comma separators are correct for single-line form
    let src = "
        @schema Item { item_name: string, item_weight: f64, item_rare*: bool }
        @schema Chest { chest_name: string, gold: i32, loot: list<Item>, owner*: string }";
    let cfg = AAML::parse(src).expect("Parse failed");
    print_schema(&cfg, "Item");
    print_schema(&cfg, "Chest");

    let chest_ok: HashMap<String, String> = [
        ("chest_name".into(), "Ancient Chest".into()),
        ("gold".into(),       "250".into()),
        ("loot".into(),
         "[{ item_name = Iron Sword, item_weight = 3.2 }, \
           { item_name = Magic Gem, item_weight = 0.1, item_rare = true }]".into()),
    ].into();
    check(&cfg, "Chest", &chest_ok, "2 valid Items, no owner*   → ✔");

    let chest_full: HashMap<String, String> = [
        ("chest_name".into(), "Boss Chest".into()),
        ("gold".into(),       "999".into()),
        ("loot".into(),
         "[{ item_name = Dragon Scale, item_weight = 5.0, item_rare = true }]".into()),
        ("owner".into(),      "DragonKing".into()),
    ].into();
    check(&cfg, "Chest", &chest_full, "1 Item + owner*            → ✔");

    let chest_bad: HashMap<String, String> = [
        ("chest_name".into(), "Broken Chest".into()),
        ("gold".into(),       "10".into()),
        ("loot".into(), "[{ item_name = Junk, item_weight = heavy }]".into()),
    ].into();
    check(&cfg, "Chest", &chest_bad, "item_weight = 'heavy'      → ✘");

    let chest_no_gold: HashMap<String, String> = [
        ("chest_name".into(), "Empty Chest".into()),
        ("loot".into(),       "[]".into()),
    ].into();
    check(&cfg, "Chest", &chest_no_gold, "gold field missing         → ✘");
}

// ── 5. File-based load ────────────────────────────────────────────────────────

fn section_5_file_based() {
    section("5. Load config_base.aam — file with nested schemas & lists");

    match AAML::load("config_base.aam") {
        Ok(cfg) => {
            println!("   ✔ Loaded config_base.aam\n");
            println!("   Schemas in file:");
            for name in &["Address", "Database", "Server"] {
                let marker = if cfg.get_schema(name).is_some() { "✔" } else { "✗" };
                println!("     {marker} {name}");
            }
            println!();
            println!("   Values:");
            for key in &["app_env", "build_id", "description", "tags", "feature_flags"] {
                print_key(&cfg, key);
            }
            println!();
            println!("   Inline objects:");
            for key in &["address", "database", "server"] {
                print_key(&cfg, key);
            }
        }
        Err(e) => eprintln!("   ✘ Load error: {e}"),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn check(cfg: &AAML, schema: &str, data: &HashMap<String, String>, label: &str) {
    match cfg.apply_schema(schema, data) {
        Ok(()) =>
            println!("   ✔ {label}"),
        Err(AamlError::SchemaValidationError { field, type_name, details, .. }) =>
            println!("   ✔ {label}\n       ↳ field '{field}' ({type_name}): {details}"),
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

fn print_schema(cfg: &AAML, name: &str) {
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
