# Schema Validation

Validation is the core pillar of Adapters. You can build validation schemas programmatically using builder types, or let the procedural macros generate them automatically.

---

## Declarative Attributes (`#[derive(Schema)]`)

When deriving `Schema` on your structures, you can use the `#[schema(...)]` attribute to specify validation constraints. The macro supports a rich set of rules:

### String Constraints
- `min_length = <usize>`: The string must contain at least N characters.
- `max_length = <usize>`: The string must contain at most M characters.
- `non_empty`: The string cannot be empty (equivalent to `min_length = 1`).
- `email`: Enforces a standard RFC 5322-compliant email format check.
- `url`: Enforces a valid absolute URL format check.
- `regex = "<pattern>"`: Matches the string against a custom regular expression.

### Numeric Constraints
- `min = <number>`: Value must be greater than or equal to N.
- `max = <number>`: Value must be less than or equal to M.
- `positive`: Enforces that the number is strictly greater than zero (> 0).
- `negative`: Enforces that the number is strictly less than zero (< 0).
- `non_zero`: Fails validation if the number is exactly 0.

### Structural Controls
- `strict`: Enforces strict type checking. When true, types like numbers will not be coerced from strings.
- `optional`: Declares the field as optional (permits Null values).
- `default = <expression>`: Applies a default value if the key is missing in the source payload.

---

## Programmatic Schema Building

If you need to construct schemas dynamically at runtime, use our highly expressive builder APIs:

```rust
use adapters::prelude::*;
use adapters::schema::{ObjectSchema, StringSchema, IntegerSchema};

let schema = ObjectSchema::new()
    .field("username", StringSchema::new().required().non_empty().alphanumeric())
    .field("age", IntegerSchema::new().required().min(18).max(99).positive())
    .field("balance", IntegerSchema::new().required().non_zero())
    .strict(); // Rejects unknown object keys

// Validate a dynamic Value representation
let payload = Value::Null; // or Value::Object(...)
let result = schema.validate(&payload, "root");
```

---

## Nested Schema Validation

Adapters natively supports recursive validation of complex nested structures. When a structure derives `Schema`, its schema definition incorporates the schema of any sub-structures that also implement `SchemaProvider`.

For example, when validating a parent struct like `User`, any nested objects (e.g. `Address`) will be fully validated against their own schemas. Any validation failures in the nested child are reported with correct dot-notation paths (e.g., `address.city` or `address.zip_code`).

```rust
use adapters::prelude::*;

#[derive(Schema, Debug)]
struct Address {
    #[schema(min_length = 3)]
    city: String,
    country: String,
}

#[derive(Schema, Debug)]
struct User {
    name: String,
    address: Address, // Automatically delegates validation to Address::schema()!
}
```

If you attempt to parse a payload where `address.city` is only two characters long, the validation engine will fail and report `address.city` as the failing field path.
