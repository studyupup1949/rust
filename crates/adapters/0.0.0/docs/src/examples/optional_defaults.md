# Optional Fields & Default Values

Demonstrates how to handle missing structural values in fields using `Option<T>` or define fallbacks dynamically via declarative metadata attributes.

### Compilable Example

```rust
//! optional_defaults example — Option<T> fields and default values.

use adapters::{Adapter, Validate};
use adapters_macros::Schema;

#[derive(Schema, Debug)]
struct Profile {
    name: String,
    bio: Option<String>,
    #[schema(default = "India")]
    country: String,
    #[schema(default = 0)]
    score: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== optional_defaults example ===\n");

    // All fields present
    let full = Profile::from_json(
        r#"{
        "name": "Alice",
        "bio": "Rust enthusiast",
        "country": "USA",
        "score": 42
    }"#,
    )?;
    println!("Full: {:?}", full);

    // bio absent → None, country absent → default "India", score absent → default 0
    let minimal = Profile::from_json(r#"{"name": "Bob"}"#)?;
    println!("Minimal: {:?}", minimal);
    assert!(minimal.bio.is_none());
    assert_eq!(minimal.country, "India");
    assert_eq!(minimal.score, 0);
    println!("bio is None: {}", minimal.bio.is_none());
    println!("country default: {}", minimal.country);
    println!("score default: {}", minimal.score);

    // Explicit null for bio → None
    let with_null = Profile::from_json(r#"{"name": "Carol", "bio": null}"#)?;
    println!("\nWith explicit null bio: {:?}", with_null);
    assert!(with_null.bio.is_none());

    // Validate
    full.validate()?;
    minimal.validate()?;
    println!("\nAll valid!");

    Ok(())
}
```
