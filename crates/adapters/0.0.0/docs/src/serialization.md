# Serialization & Deserialization

Adapters features highly optimized and fully type-safe serialization and deserialization traits. These traits define how memory structures are converted to and from intermediate `Value` dynamic trees.

---

## The `Serialize` Trait

The `Serialize` trait defines how a typed Rust instance maps into a dynamic `Value`.

```rust
pub trait Serialize {
    fn serialize(&self) -> Value;
}
```

Every standard primitive type implements this out of the box:
- Booleans (`bool`)
- All integers (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `usize`)
- All floats (`f32`, `f64`)
- Text types (`String`, `&str`)
- Container types (`Option<T>`, `Vec<T>`, `BTreeMap<String, T>`)

---

## The `Deserialize` Trait

The `Deserialize` trait is the inverse operation, decoding a `Value` tree back into a typed Rust instance while auditing numeric bounds and ranges:

```rust
pub trait Deserialize: Sized {
    fn deserialize(value: Value) -> Result<Self, Error>;
}
```

### Safety and Strict Numeric Bounds
Unlike naive decoders that might cause silent overflows, Adapters performs type-safe checks during deserialization:
- If you deserialize a value of `300` into a `u8` field, it will return a clean `DeserializationError` explaining that `300` overflows the bounds of `u8`.
- If an unsigned integer type (e.g. `u32`) receives a negative value (e.g. `-10`), it is caught and rejected immediately.

---

## Custom Manual Implementations

While using the `#[derive(Schema)]` macro covers 99% of use cases, you can implement these traits manually for total control:

```rust
use adapters::{Serialize, Deserialize, Value, Error, error::DeserializationError};

struct Point {
    x: i32,
    y: i32,
}

impl Serialize for Point {
    fn serialize(&self) -> Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert("x".to_string(), Value::Int(self.x as i64));
        map.insert("y".to_string(), Value::Int(self.y as i64));
        Value::Object(map)
    }
}

impl Deserialize for Point {
    fn deserialize(value: Value) -> Result<Self, Error> {
        let obj = value.as_object().ok_or_else(|| {
            DeserializationError::new("expected an object representation")
        })?;
        
        let x = obj.get("x")
            .and_then(|v| v.as_int())
            .ok_or_else(|| DeserializationError::new("missing coordinate x"))? as i32;
            
        let y = obj.get("y")
            .and_then(|v| v.as_int())
            .ok_or_else(|| DeserializationError::new("missing coordinate y"))? as i32;

        Ok(Point { x, y })
    }
}
```

---

## Nested/Recursive Serialization & Deserialization

Adapters fully supports deep nested models under both programmatic use and standard macro derivation. 

When you define a structure that references another structure as a field (both having derived `Schema`), the serialization and deserialization routines operate recursively:
1. **Nested Serialization**: Translates the complex model tree from the nested structures all the way down into deep hierarchical key-value JSON trees (`Value::Object`).
2. **Nested Deserialization**: Reconstructs custom nested Rust models dynamically from the intermediate hierarchy, handling all type verification and bounds checking nested deep inside the child structures.

```rust
use adapters::prelude::*;

#[derive(Schema, Debug)]
struct Metadata {
    created_at: String,
    version: String,
}

#[derive(Schema, Debug)]
struct Post {
    title: String,
    metadata: Metadata, // Automatically maps, validates, and serializes recursively!
}
```
