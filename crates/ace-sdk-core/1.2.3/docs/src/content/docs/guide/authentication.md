---
title: Authentication
description: Device code OAuth and token management
---

## Device Code Flow (RFC 8628)

```rust
use ace_sdk_core::auth::{login, LoginOptions};

let options = LoginOptions {
    server_url: "https://ace-api.code-engine.app".to_string(),
    client_type: "rust-app".to_string(),
    ..Default::default()
};

let result = login(options, |user_code, verification_uri| {
    println!("Go to: {}", verification_uri);
    println!("Enter code: {}", user_code);
}).await?;
```

## Token Management

Tokens are automatically refreshed when within 5 minutes of expiry.

```rust
use ace_sdk_core::auth::{is_token_locally_expired, get_effective_token};
use ace_sdk_core::types::AceConfig;

let config = AceConfig::default();
let expired = is_token_locally_expired(&config);
let token = get_effective_token(&config);
```

## Token Types

| Prefix | Type | Description |
|--------|------|-------------|
| `ace_user_` | User | Personal token from device code flow |
| `ace_` | Org | Organization API key |
