# ABPilot CC Rust SDK

A type-safe, async Rust SDK for the ABPilot CC platform.

## Features

- 🔐 **Multiple Authentication Methods**: JWT, API Key, App/World Signatures
- 🚀 **Async/Await**: Built on tokio and reqwest
- 🎯 **Type-Safe**: Leverage Rust's type system
- 🔧 **Modular**: Optional MP and APP features
- 📦 **Zero-Copy**: Minimal allocations
- ✅ **Well-Tested**: Comprehensive test coverage

## Installation

Add to your `Cargo.toml`:

```toml
# Full SDK (both MP and APP)
[dependencies]
abpilot-cc-sdk = "0.1"

# MP only (management operations)
[dependencies]
abpilot-cc-sdk = { version = "0.1", default-features = false, features = ["mp"] }

# APP only (runtime operations)
[dependencies]
abpilot-cc-sdk = { version = "0.1", default-features = false, features = ["app"] }
```

## Quick Start

### Authentication Flow (MP Feature)

```rust
use abpilot_cc_sdk::{AbpilotClient, AuthMethod};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AbpilotClient::new();
    
    // Send verification code
    client.mp().send_verification_code("user@example.com").await?;
    
    // Verify code and get token
    let auth_token = client.mp()
        .verify_code("user@example.com", "123456")
        .await?;
    
    // Use the token for authenticated requests
    let mut authed_client = client.clone();
    authed_client.mp_mut().set_auth(AuthMethod::jwt(auth_token.token));
    
    Ok(())
}
```

### App Management (MP Feature)

```rust
// Create an app
let app = client.mp().create_app("My Game").await?;
println!("App ID: {}", app.app_id);
println!("Secret: {}", app.secret.unwrap());

// List all apps
let apps = client.mp().list_apps().await?;

// Get upload URLs
let files = vec!["icon.png", "config.json"];
let urls = client.mp().get_app_upload_urls(&app.app_id, &files).await?;
```

### World Management (MP Feature)

```rust
// Create a world
let world = client.mp().create_world("My World").await?;
println!("World ID: {}", world.world_id);
println!("Secret: {}", world.secret.unwrap());

// List all worlds
let worlds = client.mp().list_worlds().await?;

// Get world details
let world_details = client.mp().get_world(&world.world_id).await?;
```

### Asset Operations (APP Feature)

```rust
use serde_json::json;

let client = AbpilotClient::new();

// Create device token
let device_info = json!({
    "platform": "ios",
    "version": "1.0"
});

let token = client.app()
    .create_device_token(
        &app_id,
        &app_secret,
        &world_id,
        "device_001",
        device_info,
        3600, // TTL in seconds
    )
    .await?;

// List assets
let assets = client.app()
    .list_assets(&app_id, &app_secret, "device_001", &world_id)
    .await?;

// Add gold
let asset = client.app()
    .add_asset(&world_id, &world_secret, "device_001", "gold", "001", 100)
    .await?;
println!("New balance: {}", asset.value);
```

### World Node Management (APP Feature)

```rust
// Update world node
let node = client.app()
    .update_world_node(
        &world_id,
        &world_secret,
        "https://node1.example.com",
        "cn|us"
    )
    .await?;

// Delete world node
client.app()
    .delete_world_node(&world_id, &world_secret, "https://node1.example.com")
    .await?;
```

## API Reference

### MP Client (Management Platform)

#### Authentication
- `send_verification_code(email)` - Send verification code
- `verify_code(email, code)` - Verify code and get JWT token

#### API Keys
- `create_api_key(name)` - Create new API key
- `delete_api_key(apikey)` - Delete API key
- `list_api_keys()` - List all API keys

#### Apps
- `create_app(name)` - Create new app
- `delete_app(app_id)` - Delete app
- `list_apps()` - List all apps
- `reset_app_secret(app_id)` - Reset app secret
- `get_app_upload_urls(app_id, files)` - Get S3 upload URLs
- `get_app_download_urls(app_id, files)` - Get S3 download URLs

#### Worlds
- `create_world(name)` - Create new world
- `delete_world(world_id)` - Delete world
- `list_worlds()` - List all worlds
- `get_world(world_id)` - Get world details
- `reset_world_secret(world_id)` - Reset world secret
- `get_world_upload_urls(world_id, files)` - Get S3 upload URLs
- `get_world_download_urls(world_id, files)` - Get S3 download URLs

### APP Client (Application Runtime)

#### Assets
- `list_assets(app_id, app_secret, device_id, world_id)` - List all assets
- `get_asset(app_id, app_secret, device_id, world_id, type, id)` - Get specific asset
- `add_asset(world_id, world_secret, device_id, type, id, delta)` - Add/deduct asset

#### World Nodes
- `update_world_node(world_id, world_secret, base_url, tags)` - Update node
- `delete_world_node(world_id, world_secret, base_url)` - Delete node

#### Devices
- `create_device_token(app_id, app_secret, world_id, device_id, info, ttl)` - Create token
- `get_device_info(world_id, world_secret, token)` - Get device info

## Configuration

```rust
use abpilot_cc_sdk::Config;
use std::time::Duration;

let config = Config::new()
    .with_mp_base_url("https://custom-mp.example.com")
    .with_app_base_url("https://custom-app.example.com")
    .with_timeout(Duration::from_secs(60))
    .with_max_retries(5);

let client = AbpilotClient::with_config(config);
```

## Error Handling

```rust
use abpilot_cc_sdk::AbpilotError;

match client.mp().create_app("My App").await {
    Ok(app) => println!("Created: {}", app.app_id),
    Err(AbpilotError::AuthError(msg)) => eprintln!("Auth failed: {}", msg),
    Err(AbpilotError::ApiError { status, message }) => {
        eprintln!("API error {}: {}", status, message)
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Examples

Run examples with:

```bash
# Authentication flow
cargo run --example auth_flow --features mp

# App management
ABPILOT_TOKEN=your_token cargo run --example app_management --features mp

# World management
ABPILOT_TOKEN=your_token cargo run --example world_management --features mp

# Asset operations
APP_ID=xxx APP_SECRET=xxx WORLD_ID=xxx WORLD_SECRET=xxx \
  cargo run --example asset_operations --features app
```

## Testing

```bash
# Run all tests
cargo test --all-features

# Test MP feature only
cargo test --no-default-features --features mp

# Test APP feature only
cargo test --no-default-features --features app
```

## Feature Flags

- `mp` - Management Platform client (authentication, apps, worlds)
- `app` - Application runtime client (assets, nodes, devices)
- `full` - Both MP and APP (default)

## License

MIT OR Apache-2.0

## Contributing

Contributions welcome! Please ensure all tests pass before submitting PRs.
