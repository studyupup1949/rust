//! explicit_schema example — demonstrates ObjectSchema builder without derive macro.

use adapters::json::parse;
use adapters::{Schema, SchemaValidator, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== explicit_schema example ===\n");

    // Build a schema for a user object without any derive macros
    let user_schema = Schema::object()
        .field(
            "username",
            Schema::string().required().min_length(3).max_length(32),
        )
        .field("email", Schema::string().required().email())
        .field("age", Schema::integer().required().min(18).max(120))
        .field("bio", Schema::string().optional());

    // Validate a valid JSON value
    let json = r#"{"username":"bob","email":"bob@example.com","age":30}"#;
    let value = parse(json)?;
    match user_schema.validate(&value, "root") {
        Ok(()) => println!("Valid user object!"),
        Err(e) => println!("Validation error: {}", e),
    }

    // Validate an invalid value
    let bad_json = r#"{"username":"x","email":"notanemail","age":15}"#;
    let bad_value = parse(bad_json)?;
    match user_schema.validate(&bad_value, "root") {
        Ok(()) => println!("Valid (unexpected)"),
        Err(e) => println!("Error (expected): {}", e),
    }

    // Build a standalone string schema
    let email_schema = Schema::string().required().email().min_length(5);
    let email_val = Value::String("test@example.com".into());
    let bad_email = Value::String("notvalid".into());

    println!(
        "\nEmail '{}': {:?}",
        email_val.as_str().unwrap(),
        Schema::String(email_schema)
            .validate(&email_val, "email")
            .map(|_| "valid")
    );
    let email_schema2 = Schema::string().required().email().min_length(5);
    println!(
        "Email 'notvalid': {:?}",
        Schema::String(email_schema2)
            .validate(&bad_email, "email")
            .err()
            .map(|e| e.to_string())
    );

    Ok(())
}
