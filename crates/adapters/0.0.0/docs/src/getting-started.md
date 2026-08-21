# Getting Started

Learn how to install, configure, and run your first schema validation and deserialization flow using Adapters.

## Installation

### Method 1: Reference latest Git repository (Recommended)

Since this project is under active development, referencing the official Git repository ensures you always have the latest performance improvements and features:

```toml
[dependencies]
adapters = { git = "https://github.com/muhammad-fiaz/adapters.git" }
```

### Method 2: Manual Version Entry

Alternatively, add `adapters` directly as a version dependency to your `Cargo.toml`:

```toml
[dependencies]
adapters = "0.0.0"
```

---

## Your First Schema Model

With Adapters, you define your data structures using standard Rust structs and derive their schema and validation rules directly using the `#[derive(Schema)]` macro.

Here is a complete, compilable example:

```rust
use adapters::prelude::*;

#[derive(Schema, Debug)]
struct UserProfile {
    #[schema(min_length = 3, max_length = 32)]
    username: String,
    
    #[schema(email)]
    email: String,
    
    #[schema(min = 18, max = 120)]
    age: u8,
    
    #[schema(optional)]
    website: Option<String>,
}

fn main() -> Result<(), adapters::Error> {
    // 1. A valid JSON payload
    let json_data = r#"{
        "username": "supercoder",
        "email": "contact@example.com",
        "age": 28,
        "website": "https://muhammad-fiaz.github.io"
    }"#;

    // 2. Parse, validate, and deserialize in a single operation!
    let user = UserProfile::from_json(json_data)?;
    println!("Successfully parsed and validated user: {:?}", user);

    // 3. Serializing a struct instance back to JSON
    let serialized_json = user.to_json()?;
    println!("Serialized JSON: {}", serialized_json);

    Ok(())
}
```

---

## How It Works Under the Hood

When you call `UserProfile::from_json(json_data)`:

1. **JSON Tokenization & Parsing**: The native JSON engine parses the string into a structured `Value` tree.
2. **Schema Compilation**: The derived `SchemaProvider` implementation yields the structural definition of your struct.
3. **Dynamic Validation**: The `SchemaValidator` runs validation rules over the dynamic fields (e.g., confirming `age` is between 18 and 120, and `email` is correctly formatted).
4. **Strong Typing Deserialization**: If validation passes, the deserializer converts the checked `Value` directly into your `UserProfile` Rust struct.
