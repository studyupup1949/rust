# Nested Models

This example outlines recursive struct configurations. Sub-elements are fully validated, and dynamic errors accumulate precise dot-notation paths (e.g. `address.city`).

### Compilable Example

```rust
//! nested_models example — nested structs with validation error path reporting.

use adapters::{Adapter, Validate};
use adapters_macros::Schema;

#[derive(Schema, Debug)]
struct Address {
    city: String,
    country: String,
}

#[derive(Schema, Debug)]
struct User {
    username: String,
    address: Address,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== nested_models example ===\n");

    // Full round-trip
    let json = r#"{
        "username": "alice",
        "address": {"city": "Berlin", "country": "Germany"}
    }"#;

    let user = User::from_json(json)?;
    println!("Parsed user: {:?}", user);

    let out = user.to_json()?;
    println!("Re-serialized: {}", out);

    // Parse back
    let user2 = User::from_json(&out)?;
    println!("Round-trip OK: username={}", user2.username);

    // Validation
    user.validate()?;
    println!("Valid!");

    Ok(())
}
```
