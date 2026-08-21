//! Advanced validation rules example showcasing `non_empty`, `alphanumeric`,
//! `positive`, `negative`, and `non_zero` validation rules both via declarative
//! macros and programmatic builders.

use adapters::SchemaValidator;
use adapters::prelude::*;
use adapters::schema::{FloatSchema, IntegerSchema, ObjectSchema, StringSchema};

/// A financial invoice transaction derived using macro constraints.
#[derive(Schema, Debug)]
struct TransactionInvoice {
    /// Reference code must be alphanumeric and non-empty.
    #[schema(non_empty, alphanumeric)]
    reference_code: String,

    /// Number of items must be strictly positive (> 0).
    #[schema(positive)]
    quantity: i32,

    /// Adjustments or discounts must be strictly negative (< 0.0).
    #[schema(negative)]
    discount: f32,

    /// Balance cannot be zero (!= 0).
    #[schema(non_zero)]
    balance: i64,
}

fn main() -> Result<(), Error> {
    println!("=== 1. Declarative Macro Validation ===");

    // Valid Payload
    let valid_json = r#"{
        "reference_code": "INV2026TX",
        "quantity": 5,
        "discount": -15.50,
        "balance": 250
    }"#;
    let invoice = TransactionInvoice::from_json(valid_json)?;
    println!(
        "Successfully parsed and validated valid invoice:\n{:#?}\n",
        invoice
    );

    // Invalid Payload with multiple constraint violations
    let invalid_json = r#"{
        "reference_code": "INV-2026-TX!",
        "quantity": 0,
        "discount": 5.50,
        "balance": 0
    }"#;

    match TransactionInvoice::from_json(invalid_json) {
        Ok(_) => println!("Error: Expected validation failure but succeeded!"),
        Err(e) => {
            println!("Validation failed as expected! Errors:");
            println!("{}", e);
        }
    }

    println!("\n=== 2. Programmatic Builder Validation ===");

    // Build the identical validation logic programmatically
    let invoice_schema = ObjectSchema::new()
        .field(
            "reference_code",
            StringSchema::new().required().non_empty().alphanumeric(),
        )
        .field("quantity", IntegerSchema::new().required().positive())
        .field("discount", FloatSchema::new().required().negative())
        .field("balance", IntegerSchema::new().required().non_zero());

    // Validate a dynamic Value payload using our schema
    let dynamic_invalid_payload = Value::Object(
        [
            ("reference_code".to_string(), Value::String("".to_string())), // Fails non_empty
            ("quantity".to_string(), Value::Int(-10)),                     // Fails positive
            ("discount".to_string(), Value::Float(-5.0)),                  // Passes negative
            ("balance".to_string(), Value::Int(0)),                        // Fails non_zero
        ]
        .into_iter()
        .collect(),
    );

    match invoice_schema.validate(&dynamic_invalid_payload, "invoice") {
        Ok(_) => println!("Error: Expected programmatic validation failure!"),
        Err(err) => {
            println!("Programmatic validation detected constraint violations successfully:");
            println!(
                "Field: {}, Code: {}, Message: {}",
                err.field, err.code, err.message
            );
        }
    }

    Ok(())
}
