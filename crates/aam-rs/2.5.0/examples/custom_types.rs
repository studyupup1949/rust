use aam_rs::aam::AAM;
use aam_rs::builder::{AAMBuilder, SchemaField};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AAM Custom Types Example ===\n");

    // --- 1. Builtin primitive types ---
    println!("--- 1. Built-in primitive types ---");
    let mut b = AAMBuilder::new();
    b.schema(
        "Primitives",
        [
            SchemaField::required("name", "string"),
            SchemaField::required("age", "i32"),
            SchemaField::required("score", "f64"),
            SchemaField::required("active", "bool"),
            SchemaField::required("tint", "color"),
        ],
    );
    b.add_line("name", "Alice");
    b.add_line("age", "30");
    b.add_line("score", "9.75");
    b.add_line("active", "true");
    b.add_line("tint", "#ff6600");

    match AAM::parse(&b.build()) {
        Ok(cfg) => {
            println!("name   = {}", cfg.get("name").unwrap_or("<missing>"));
            println!("age    = {}", cfg.get("age").unwrap_or("<missing>"));
            println!("score  = {}", cfg.get("score").unwrap_or("<missing>"));
            println!("active = {}", cfg.get("active").unwrap_or("<missing>"));
            println!("tint   = {}", cfg.get("tint").unwrap_or("<missing>"));
            println!("Primitives schema: OK\n");
        }
        Err(e) => eprintln!("Primitives error: {:?}\n", e),
    }

    // --- 2. @type aliases ---
    println!("--- 2. @type aliases (ipv4 -> string, port -> i32) ---");
    let mut b = AAMBuilder::new();
    b.type_alias("ipv4", "string");
    b.type_alias("port", "i32");
    b.schema(
        "Network",
        [
            SchemaField::required("ip", "ipv4"),
            SchemaField::required("port", "port"),
        ],
    );
    b.add_line("ip", "192.168.1.1");
    b.add_line("port", "8080");

    match AAM::parse(&b.build()) {
        Ok(cfg) => {
            println!("ip   = {}", cfg.get("ip").unwrap_or("<missing>"));
            println!("port = {}", cfg.get("port").unwrap_or("<missing>"));
            println!("Network schema: OK\n");
        }
        Err(e) => eprintln!("Network error: {:?}\n", e),
    }

    // --- 3. Type validation errors ---
    println!("--- 3. Type validation errors ---");

    let mut b = AAMBuilder::new();
    b.schema("S", [SchemaField::required("val", "i32")]);
    b.add_line("val", "not_a_number");
    match AAM::parse(&b.build()) {
        Ok(_) => println!("(unexpected) bad i32 accepted"),
        Err(e) => println!("Correctly rejected bad i32: {:?}", e),
    }

    let mut b = AAMBuilder::new();
    b.schema("S", [SchemaField::required("flag", "bool")]);
    b.add_line("flag", "yes_please");
    match AAM::parse(&b.build()) {
        Ok(_) => println!("(unexpected) bad bool accepted"),
        Err(e) => println!("Correctly rejected bad bool: {:?}", e),
    }

    let mut b = AAMBuilder::new();
    b.schema("S", [SchemaField::required("c", "color")]);
    b.add_line("c", "notacolor");
    match AAM::parse(&b.build()) {
        Ok(_) => println!("(unexpected) bad color accepted"),
        Err(e) => println!("Correctly rejected bad color: {:?}", e),
    }

    // --- 4. runtime apply_schema is intentionally not in AAM ---
    println!("\n--- 4. runtime apply_schema ---");
    let mut b = AAMBuilder::new();
    b.schema(
        "Player",
        [
            SchemaField::required("name", "string"),
            SchemaField::required("score", "i32"),
        ],
    );
    if let Err(err) = AAM::parse(&b.build()).map_err(|errs| errs.into_iter().next().unwrap()) {
        println!(
            "Runtime apply_schema demo skipped on this parser build: {}",
            err
        );
        return Ok(());
    }

    let mut data: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    data.insert("name".into(), "Bob".into());
    data.insert("score".into(), "42".into());
    println!("Player data sample: {:?}", data);
    println!("Runtime apply_schema is not part of the new AAM API.");

    data.insert("score".into(), "not_a_number".into());
    println!("Bad Player data sample: {:?}", data);
    println!("Use parse-time validation or external validator integration for ad-hoc maps.");

    Ok(())
}
