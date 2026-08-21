//! basic_schema example — demonstrates #[derive(Schema)], from_json, validate, to_json.

use adapters::{Adapter, SchemaProvider, Validate};
use adapters_macros::Schema;

#[derive(Schema, Debug)]
struct User {
    #[schema(min_length = 3, max_length = 32)]
    username: String,
    #[schema(email)]
    email: String,
    #[schema(min = 18, max = 120)]
    age: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== basic_schema example ===\n");

    // Valid user
    let json = r#"{"username": "alice", "email": "alice@example.com", "age": 25}"#;
    let user = User::from_json(json)?;
    println!("Parsed: {:?}", user);

    user.validate()?;
    println!("Valid!");

    let out = user.to_json()?;
    println!("Serialized: {}", out);

    let schema = User::schema();
    println!("Schema type: {:?}", schema);

    // Validation failure
    println!("\n--- Testing bad input ---");
    let bad_result = User::from_json(r#"{"username": "ab", "email": "not-an-email", "age": 10}"#);
    match bad_result {
        Ok(_) => println!("Unexpectedly parsed"),
        Err(e) => println!("Validation error (expected): {}", e),
    }

    Ok(())
}
