# actix-web-request-uuid Usage Guide

## ID Length Customization

The `with_id_length()` method allows you to customize the length of generated request IDs. This is useful when you need shorter IDs for performance reasons or specific length requirements.

### Basic Usage

```rust
use actix_web::{App, HttpServer, web};
use actix_web_request_uuid::RequestIDMiddleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // Default: 36-character UUID (with hyphens)
            .wrap(RequestIDMiddleware::new())

            // Custom: 16-character UUID (truncated)
            .wrap(RequestIDMiddleware::new().with_id_length(16))

            // Custom: 8-character UUID (very short)
            .wrap(RequestIDMiddleware::new().with_id_length(8))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### How It Works

When you specify a custom length:
- If the length is **less than 36**, the UUID is truncated to the specified length
- If the length is **36 or more**, the full UUID (36 characters) is used
- The method **panics** if you specify a length of 0

### Examples

```rust
// 36 characters (default)
RequestIDMiddleware::new()
// Output: "550e8400-e29b-41d4-a716-446655440000"

// 16 characters
RequestIDMiddleware::new().with_id_length(16)
// Output: "550e8400-e29b-41"

// 8 characters
RequestIDMiddleware::new().with_id_length(8)
// Output: "550e8400"

// 4 characters (very short, might not be unique enough)
RequestIDMiddleware::new().with_id_length(4)
// Output: "550e"
```

### Combining with Other Methods

The `with_id_length()` method can be combined with other configuration methods:

```rust
// Short IDs with custom header
RequestIDMiddleware::new()
    .with_id_length(12)
    .header_name("X-Trace-ID")

// Note: with_full_uuid() and with_simple_uuid() will override the length setting
RequestIDMiddleware::new()
    .with_id_length(16)      // This will be overridden
    .with_full_uuid()        // Forces 36 characters with hyphens

// Custom generator ignores length setting
RequestIDMiddleware::new()
    .with_id_length(16)      // This will be ignored
    .generator(|| "custom-id-12345".to_string())
```

### Best Practices

1. **Uniqueness vs Performance Trade-off**
   - Shorter IDs are faster to generate and transmit
   - But they have higher collision probability
   - Recommended minimum: 8-12 characters for most applications

2. **Common Length Choices**
   - **36 characters**: Full UUID, maximum uniqueness
   - **32 characters**: UUID without hyphens (use `with_simple_uuid()`)
   - **16 characters**: Good balance of uniqueness and size
   - **8 characters**: Minimum recommended for production

3. **Testing Different Lengths**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_lengths() {
        // Test that generated IDs match expected lengths
        let lengths = vec![8, 12, 16, 24, 32, 36, 50];

        for &len in &lengths {
            let middleware = RequestIDMiddleware::new().with_id_length(len);
            let expected_len = len.min(36); // Max length is 36
            assert_eq!(middleware.get_id_length(), len);
        }
    }
}
```

### Error Handling

```rust
// This will panic!
let middleware = RequestIDMiddleware::new().with_id_length(0);
// Error: "Request ID length must be greater than 0"

// Safe way to handle dynamic lengths
fn create_middleware(length: usize) -> Result<RequestIDMiddleware, &'static str> {
    if length == 0 {
        Err("ID length must be greater than 0")
    } else {
        Ok(RequestIDMiddleware::new().with_id_length(length))
    }
}
```

### Performance Considerations

Shorter IDs provide several benefits:
- Less memory usage
- Faster string operations
- Smaller HTTP headers
- Reduced log file sizes

However, ensure your chosen length provides sufficient uniqueness for your use case.
