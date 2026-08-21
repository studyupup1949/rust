# Abineo Messaging

Shared structs and enums for internal messaging service.

## Installation

### Using cargo

```sh
cargo add abineo-messaging
```

### Add to Cargo.toml

```toml
[dependencies]
abineo-messaging = "1"
```

## Usage

```rust
use abineo_messaging::*;

let result = Message::email_builder()
    .subject("The Email")
    .recipient("info@abineo.com")
    .body(Content::builder()
        .title("Hello, world!")
        .subtitle("Lorem ipsum dolor")
        .text("Now that we know who you are, I know who I am")
        .secret("42"))
    .build();

assert!(result.is_ok());
```
