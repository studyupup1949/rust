//! Example: schemas inside schemas, lists, optional fields, @derive with schema selection
//!
//! Demonstrates:
//! 1. Nested schemas — a field is typed by another schema (`Item` inside `Weapon`).
//! 2. Lists — `list<string>`, `list<i32>`, `list<Item>` with inline objects.
//! 3. Optional fields (`field*: type`) — can be omitted without error.
//! 4. `@derive path.aam` — full import of all schemas and values.
//! 5. `@derive path.aam::Schema1::Schema2` — import only selected schemas.
//!
//! Run with:
//! ```sh
//! cargo run --example advanced
//! ```

use aam_rs::aaml::AAML;
use aam_rs::error::AamlError;
use std::collections::HashMap;
use std::path::Path;

fn main() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    std::env::set_current_dir(&examples_dir).expect("Cannot change dir to examples/");

    header("AAM: schemas in schemas • lists • optional fields • @derive");

    demo_full_derive();
    demo_selective_derive();
    demo_nested_schema_validation();
    demo_list_of_schemas();
    demo_derive_two_schemas();
    demo_derive_nonexistent_schema();

    footer();
}

// ── Section 1: full @derive ───────────────────────────────────────────────────

fn demo_full_derive() {
    section("1. @derive advanced_base.aam  (full import: all schemas + values)");
    println!("   File contains schemas: Item, Weapon, Player\n");

    match AAML::load("advanced_base.aam") {
        Ok(cfg) => {
            println!("   ✔ Loaded successfully\n");
            print_all_schemas(&cfg, &["Item", "Weapon", "Player"]);
            divider();
            print_key(&cfg, "player_name");
            print_key(&cfg, "level");
            print_key(&cfg, "tags");
            print_key(&cfg, "equipped_ids");
            print_key(&cfg, "sword");
        }
        Err(e) => eprintln!("   ✘ {e}"),
    }
}

// ── Section 2: @derive with a single schema ───────────────────────────────────

fn demo_selective_derive() {
    section("2. @derive advanced_child.aam  (@derive base.aam::Player, score* not set)");
    println!("   Expected: Player ✔, Item ✗, Weapon ✗, score* absent — not an error\n");

    match AAML::load("advanced_child.aam") {
        Ok(cfg) => {
            println!("   ✔ Loaded successfully\n");
            println!("   Schema presence after @derive::Player:");
            println!("     Player : {}", schema_marker(&cfg, "Player", true));
            println!("     Item   : {}", schema_marker(&cfg, "Item",   false));
            println!("     Weapon : {}", schema_marker(&cfg, "Weapon", false));
            println!();
            print_all_schemas(&cfg, &["Player"]);
            divider();
            print_key(&cfg, "player_name");
            print_key(&cfg, "level");
            print_key(&cfg, "tags");
            print_key(&cfg, "equipped_ids");
            match cfg.find_obj("score") {
                Some(v) => println!("   {:>15} = {v}", "score"),
                None    => println!("   {:>15} = <not set — optional ✔>", "score"),
            }
        }
        Err(e) => eprintln!("   ✘ Unexpected error: {e}"),
    }
}

// ── Section 3: programmatic validation of nested schemas ─────────────────────

fn demo_nested_schema_validation() {
    section("3. Programmatic validation — Weapon { base: Item }  (no file)");

    let src = r#"
        @schema Item   { item_name: string, item_weight: f64, item_rare*: bool }
        @schema Weapon { base: Item, damage: i32, description*: string }
    "#;
    let cfg = AAML::parse(src).expect("Schema parse must succeed");

    let base_ok = "{ item_name = Axe, item_weight = 5.0 }";

    let mut w = make_weapon(base_ok, "80", None);
    validate(&cfg, "Weapon", &w, "Weapon without description*  → ✔");

    w.insert("description".into(), "Heavy two-handed axe".into());
    validate(&cfg, "Weapon", &w, "Weapon with description*     → ✔");

    let w_bad = make_weapon("{ item_name = Stick, item_weight = bad_num }", "5", None);
    validate(&cfg, "Weapon", &w_bad, "item_weight = bad_num        → ✘ expect error");

    let w_no_base: HashMap<String, String> = [("damage".into(), "10".into())].into();
    validate(&cfg, "Weapon", &w_no_base, "base missing                 → ✘ expect error");
}

fn make_weapon(base: &str, damage: &str, desc: Option<&str>) -> HashMap<String, String> {
    let mut m: HashMap<String, String> = [
        ("base".into(), base.into()),
        ("damage".into(), damage.into()),
    ].into();
    if let Some(d) = desc { m.insert("description".into(), d.into()); }
    m
}

fn validate(cfg: &AAML, schema: &str, data: &HashMap<String, String>, label: &str) {
    match cfg.apply_schema(schema, data) {
        Ok(()) => println!("   ✔ {label}"),
        Err(AamlError::SchemaValidationError { field, type_name, details, .. }) =>
            println!("   ✔ {label}\n       ↳ field '{field}' ({type_name}): {details}"),
        Err(e) => eprintln!("   ✘ {label} — unexpected error: {e}"),
    }
}

// ── Section 4: list<Schema> ───────────────────────────────────────────────────

fn demo_list_of_schemas() {
    section("4. list<Item> — list of inline objects of type Item");

    let src = r#"
        @schema Item  { item_name: string, item_weight: f64, item_rare*: bool }
        @schema Chest { title: string, loot: list<Item> }
    "#;
    let cfg = AAML::parse(src).expect("Schemas must parse");

    let mut chest_ok: HashMap<String, String> = HashMap::new();
    chest_ok.insert("title".into(), "Golden Chest".into());
    chest_ok.insert(
        "loot".into(),
        "[{ item_name = Gold Coin, item_weight = 0.1 }, \
          { item_name = Gem, item_weight = 0.3, item_rare = true }]".into(),
    );
    validate(&cfg, "Chest", &chest_ok, "two valid Items in loot   → ✔");

    let mut chest_bad = chest_ok.clone();
    chest_bad.insert(
        "loot".into(),
        "[{ item_name = Broken, item_weight = bad_weight }]".into(),
    );
    validate(&cfg, "Chest", &chest_bad, "item_weight = bad_weight  → ✘ expect error");
}

// ── Section 5: @derive with two schemas ──────────────────────────────────────

fn demo_derive_two_schemas() {
    section("5. @derive advanced_base.aam::Player::Item  (two :: selectors)");
    println!("   Syntax: @derive <file>::<Schema1>::<Schema2>");
    println!("   Expected: Player ✔, Item ✔, Weapon ✗\n");

    let content = "\
        @derive advanced_base.aam::Player::Item\n\
        player_name  = TwoSchemaHero\n\
        level        = 99\n\
        tags         = [master, dual]\n\
        equipped_ids = [300]\n\
    ";

    match AAML::parse(content) {
        Ok(cfg) => {
            println!("   ✔ Loaded successfully\n");
            println!("   Schema presence:");
            println!("     Player : {}", schema_marker(&cfg, "Player", true));
            println!("     Item   : {}", schema_marker(&cfg, "Item",   true));
            println!("     Weapon : {}", schema_marker(&cfg, "Weapon", false));
            println!();
            print_all_schemas(&cfg, &["Player", "Item"]);
            divider();
            print_key(&cfg, "player_name");
            print_key(&cfg, "level");
            print_key(&cfg, "tags");
            print_key(&cfg, "equipped_ids");
        }
        Err(e) => eprintln!("   ✘ {e}"),
    }
}

// ── Section 6: error — non-existent schema ────────────────────────────────────

fn demo_derive_nonexistent_schema() {
    section("6. @derive with a non-existent schema → DirectiveError");
    println!("   @derive advanced_base.aam::NonExistentSchema\n");

    match AAML::parse("@derive advanced_base.aam::NonExistentSchema\n") {
        Err(AamlError::DirectiveError(cmd, msg)) =>
            println!("   ✔ @{cmd}: {msg}"),
        other =>
            eprintln!("   ✘ Unexpected result: {other:?}"),
    }
}

// ── Helper functions ──────────────────────────────────────────────────────────

fn header(title: &str) {
    println!("\n{}", "═".repeat(62));
    println!("  {title}");
    println!("{}\n", "═".repeat(62));
}

fn section(title: &str) {
    println!("\n┌─{}─┐", "─".repeat(58));
    println!("│  {:<58}│", title);
    println!("└─{}─┘", "─".repeat(58));
}

fn divider() {
    println!("   {}", "─".repeat(52));
}

fn footer() {
    println!("\n{}", "═".repeat(62));
    println!("  Done.");
    println!("{}", "═".repeat(62));
}

/// Returns a display string showing whether a schema is present or absent,
/// and whether that matches the expectation (`expect_present`).
fn schema_marker(cfg: &AAML, name: &str, expect_present: bool) -> &'static str {
    match (cfg.get_schema(name).is_some(), expect_present) {
        (true,  true)  => "present  ✔",
        (false, false) => "absent   ✔",
        (true,  false) => "present  ✗ (unexpected!)",
        (false, true)  => "absent   ✗ (expected!)",
    }
}

fn print_key(cfg: &AAML, key: &str) {
    match cfg.find_obj(key) {
        Some(v) => println!("   {:>15} = {v}", key),
        None    => println!("   {:>15} = <not found>", key),
    }
}

fn print_all_schemas(cfg: &AAML, names: &[&str]) {
    for &name in names {
        match cfg.get_schema(name) {
            Some(schema) => {
                let mut fields: Vec<_> = schema.fields.iter().collect();
                fields.sort_by_key(|(k, _)| k.as_str());
                println!("   Schema '{name}':");
                for (field, ty) in &fields {
                    let opt = if schema.is_optional(field) { "*" } else { " " };
                    println!("     {opt} {field:<20} : {ty}");
                }
                println!();
            }
            None => println!("   Schema '{name}' not found\n"),
        }
    }
}
