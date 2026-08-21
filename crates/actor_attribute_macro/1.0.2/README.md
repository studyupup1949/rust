# Actor Attribute Macro

This crate provides the `#[actor]` procedural macro for the `simple_json_server` library. The macro automatically implements the `Actor` trait for structs by generating a JSON-based dispatch method.

## How it works

The `#[actor]` macro:

1. **Analyzes public async methods** in an `impl` block
2. **Generates message structs** for each method's parameters
3. **Implements the Actor trait** with a `dispatch` method that:
   - Deserializes JSON messages in the format `{"method": "method_name", "params": {...}}`
   - Matches method names and calls the appropriate async method
   - Serializes the return value back to JSON

## Usage

```rust
use simple_json_server::{Actor, actor};

#[derive(Debug, Clone)]
struct Calculator {
    memory: f64,
}

#[actor]
impl Calculator {
    pub async fn add(&self, a: f64, b: f64) -> f64 {
        a + b
    }
    
    pub async fn get_memory(&self) -> f64 {
        self.memory
    }
    
    pub async fn info(&self) -> String {
        "Calculator v1.0".to_string()
    }
}
```

## Generated Code

For the above example, the macro generates:

1. **Message structs** for each method:
```rust
#[derive(serde::Deserialize)]
struct AddMessage {
    a: f64,
    b: f64,
}

#[derive(serde::Deserialize)]
struct GetMemoryMessage {}

#[derive(serde::Deserialize)]
struct InfoMessage {}
```

2. **Actor trait implementation**:
```rust
impl crate::Actor for Calculator {
    fn dispatch(&self, msg: &str) -> String {
        // JSON parsing and method dispatch logic
    }
}
```

## Message Format

The macro expects JSON messages in this format:

```json
{
    "method": "method_name",
    "params": {
        "param1": "value1",
        "param2": "value2"
    }
}
```

For methods with no parameters, use an empty params object:
```json
{
    "method": "info",
    "params": {}
}
```

## Method Requirements

The macro only processes methods that are:
- **Public** (`pub`)
- **Async** (`async fn`)

Private methods and synchronous methods are ignored.

## Return Values

All return values are automatically serialized to JSON. This includes:
- Primitive types: `42` → `"42"`
- Strings: `"hello"` → `"\"hello\""`
- Complex types: `Result<T, E>` → `{"Ok": value}` or `{"Err": error}`
- Custom types (if they implement `Serialize`)

## Error Handling

The macro handles various error cases:
- **Invalid JSON**: Returns error message as JSON string
- **Missing method field**: Returns error message
- **Unknown method**: Returns "Unknown method" error
- **Parameter deserialization errors**: Returns detailed error message
- **Result serialization errors**: Returns error message

## Dependencies

This macro requires:
- `syn` - For parsing Rust syntax
- `quote` - For generating Rust code
- `proc-macro2` - For procedural macro support
- `serde` and `serde_json` - For JSON serialization/deserialization
- `tokio` - For async runtime support
