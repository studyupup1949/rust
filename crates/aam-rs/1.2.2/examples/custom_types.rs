use aam_rs::aaml::AAML;
use aam_rs::builder::{AAMBuilder, SchemaField};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AAML Custom Types Example ===\n");

    // --- 1. Builtin primitive types ---
    println!("--- 1. Built-in primitive types ---");
    let mut b = AAMBuilder::new();
    b.schema("Primitives", [
        SchemaField::required("name",   "string"),
        SchemaField::required("age",    "i32"),
        SchemaField::required("score",  "f64"),
        SchemaField::required("active", "bool"),
        SchemaField::required("tint",   "color"),
    ]);
    b.add_line("name", "Alice");
    b.add_line("age", "30");
    b.add_line("score", "9.75");
    b.add_line("active", "true");
    b.add_line("tint", "#ff6600");

    match AAML::parse(&b.build()) {
        Ok(cfg) => {
            cfg.validate_schemas_completeness()?;
            println!("name   = {}", cfg.find_obj("name").unwrap().as_str());
            println!("age    = {}", cfg.find_obj("age").unwrap().as_str());
            println!("score  = {}", cfg.find_obj("score").unwrap().as_str());
            println!("active = {}", cfg.find_obj("active").unwrap().as_str());
            println!("tint   = {}", cfg.find_obj("tint").unwrap().as_str());
            println!("Primitives schema: OK\n");
        }
        Err(e) => eprintln!("Primitives error: {:?}\n", e),
    }

    // --- 2. @type aliases ---
    println!("--- 2. @type aliases (ipv4 -> string, port -> i32) ---");
    let mut b = AAMBuilder::new();
    b.type_alias("ipv4", "string");
    b.type_alias("port", "i32");
    b.schema("Network", [
        SchemaField::required("ip",   "ipv4"),
        SchemaField::required("port", "port"),
    ]);
    b.add_line("ip", "192.168.1.1");
    b.add_line("port", "8080");

    match AAML::parse(&b.build()) {
        Ok(cfg) => {
            cfg.validate_schemas_completeness()?;
            println!("ip   = {}", cfg.find_obj("ip").unwrap().as_str());
            println!("port = {}", cfg.find_obj("port").unwrap().as_str());
            println!("Network schema: OK\n");
        }
        Err(e) => eprintln!("Network error: {:?}\n", e),
    }

    // --- 3. Type validation errors ---
    println!("--- 3. Type validation errors ---");

    let mut b = AAMBuilder::new();
    b.schema("S", [SchemaField::required("val", "i32")]);
    b.add_line("val", "not_a_number");
    match AAML::parse(&b.build()) {
        Ok(_) => println!("(unexpected) bad i32 accepted"),
        Err(e) => println!("Correctly rejected bad i32: {}", e),
    }

    let mut b = AAMBuilder::new();
    b.schema("S", [SchemaField::required("flag", "bool")]);
    b.add_line("flag", "yes_please");
    match AAML::parse(&b.build()) {
        Ok(_) => println!("(unexpected) bad bool accepted"),
        Err(e) => println!("Correctly rejected bad bool: {}", e),
    }

    let mut b = AAMBuilder::new();
    b.schema("S", [SchemaField::required("c", "color")]);
    b.add_line("c", "notacolor");
    match AAML::parse(&b.build()) {
        Ok(_) => println!("(unexpected) bad color accepted"),
        Err(e) => println!("Correctly rejected bad color: {}", e),
    }

    // --- 4. apply_schema ---
    println!("\n--- 4. apply_schema ---");
    let mut b = AAMBuilder::new();
    b.schema("Player", [
        SchemaField::required("name",  "string"),
        SchemaField::required("score", "i32"),
    ]);
    let cfg = AAML::parse(&b.build())?;

    let mut data = std::collections::HashMap::new();
    data.insert("name".into(), "Bob".into());
    data.insert("score".into(), "42".into());
    match cfg.apply_schema("Player", &data) {
        Ok(_) => println!("Player schema valid: OK"),
        Err(e) => println!("Player schema error: {}", e),
    }

    data.insert("score".into(), "not_a_number".into());
    match cfg.apply_schema("Player", &data) {
        Ok(_) => println!("(unexpected) bad score accepted"),
        Err(e) => println!("Correctly rejected bad score: {}", e),
    }

    Ok(())
}
