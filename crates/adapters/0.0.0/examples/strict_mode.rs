//! strict_mode example — demonstrates strict type validation.

use adapters::{Adapter, Schema as SchemaApi, SchemaValidator, Value};
use adapters_macros::Schema;

#[derive(Schema, Debug)]
struct Payment {
    #[schema(strict)]
    amount: f64,
    #[schema(strict)]
    currency: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== strict_mode example ===\n");

    // Valid strict input
    let valid = Payment::from_json(r#"{"amount": 18.5, "currency": "USD"}"#)?;
    println!("Valid payment: {:?}", valid);

    // Strict: string "18.5" should fail for f64 amount
    println!("\n--- Testing strict rejections ---");
    let bad1 = Payment::from_json(r#"{"amount": "18.5", "currency": "USD"}"#);
    match bad1 {
        Err(e) => println!("String for amount rejected (expected): {}", e),
        Ok(p) => println!("String for amount accepted (unexpected): {:?}", p),
    }

    // Strict: integer 42 should fail for string currency
    let bad2 = Payment::from_json(r#"{"amount": 10.0, "currency": 42}"#);
    match bad2 {
        Err(e) => println!("Int for currency rejected (expected): {}", e),
        Ok(p) => println!("Int for currency accepted (unexpected): {:?}", p),
    }

    // Manual schema: non-strict (default)
    println!("\n--- Non-strict schema (coercion allowed) ---");
    let loose_schema = SchemaApi::object()
        .field("amount", SchemaApi::float())
        .field("currency", SchemaApi::string());

    let coerced = Value::Object({
        let mut m = std::collections::BTreeMap::new();
        m.insert("amount".into(), Value::String("99.9".into()));
        m.insert("currency".into(), Value::Int(1));
        m
    });

    match loose_schema.validate(&coerced, "root") {
        Ok(()) => println!("Non-strict accepts coercible types (expected)"),
        Err(e) => println!("Non-strict failed unexpectedly: {}", e),
    }

    Ok(())
}
