# Data Transformation

Data models often look completely different depending on the layer they occupy (e.g. database schema vs. public HTTP response payload). Adapters provides functional, highly efficient tools to transform data dynamically.

---

## The `Adapt` Trait

The `Adapt` trait represents a standard pattern for mapping one typed model structure into another:

```rust
pub trait Adapt<T>: Sized {
    fn adapt(value: T) -> Result<Self, Error>;
}
```

### Typical Use Case: DB Model -> API Response

```rust
use adapters::prelude::*;

struct UserRow {
    id: i64,
    db_hash: String,
    username: String,
}

#[derive(Debug)]
struct UserAPIResponse {
    id: i64,
    username: String,
}

impl Adapt<UserRow> for UserAPIResponse {
    fn adapt(row: UserRow) -> Result<Self, Error> {
        Ok(UserAPIResponse {
            id: row.id,
            username: row.username,
        })
    }
}
```

---

## Transformation Pipelines

The `Pipeline` builder lets you sequence separate, dynamic mapping operations over `Value` trees. If any step fails, the entire chain short-circuits safely.

```rust
use adapters::prelude::*;

let transform_pipeline = Pipeline::new()
    // Step 1: Multiply integers by 2
    .step(|val| match val {
        Value::Int(n) => Ok(Value::Int(n * 2)),
        other => Ok(other),
    })
    // Step 2: Add 1 to the result
    .step(|val| match val {
        Value::Int(n) => Ok(Value::Int(n + 1)),
        other => Ok(other),
    });

let result = transform_pipeline.run(Value::Int(5))?;
assert_eq!(result, Value::Int(11)); // 5 * 2 + 1 = 11
```

---

## Field Renaming (`FieldMapper`)

When interacting with external services, key casings often differ (e.g. snake_case vs. camelCase). The `FieldMapper` renames fields inside nested object values dynamically.

```rust
use adapters::prelude::*;

let mapper = FieldMapper::new()
    .map("first_name", "firstName")
    .map("last_name", "lastName");

let mut input_map = std::collections::BTreeMap::new();
input_map.insert("first_name".to_string(), Value::String("Alice".into()));
input_map.insert("last_name".to_string(), Value::String("Smith".into()));
input_map.insert("age".to_string(), Value::Int(30));

let input = Value::Object(input_map);
let output = mapper.apply(&input)?;

// output key structure is now:
// { "firstName": "Alice", "lastName": "Smith", "age": 30 }
```
