# adapters-macros

Procedural macros for the `adapters` library.

This crate provides macro-derive implementations for the `adapters` data validation, serialization, and transformation framework.

## Usage

This crate is a dependency of the main `adapters` crate and should not be used directly. Instead, add `adapters` to your `Cargo.toml`:

```toml
[dependencies]
adapters = "0.0.0"
```

Then, you can use the `Schema` derive macro:

```rust
use adapters::prelude::*;

#[derive(Schema)]
struct User {
    #[schema(min_length = 3)]
    username: String,
}
```
