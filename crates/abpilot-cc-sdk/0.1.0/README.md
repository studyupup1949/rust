# ABPilot CC Rust SDK Design Document

## Overview

This Rust SDK provides a type-safe, async client for interacting with the ABPilot CC platform, which consists of two Lambda function APIs:
- **MP API**: Management Platform for user authentication, API keys, apps, and worlds
- **APP API**: Application runtime for assets, world nodes, and device management

## Architecture

### Core Components

```
abpilot-cc-sdk/
├── src/
│   ├── lib.rs                 # Main library entry point
│   ├── client/
│   │   ├── mod.rs             # Client module
│   │   ├── mp.rs              # MP API client
│   │   └── app.rs             # APP API client
│   ├── auth/
│   │   ├── mod.rs             # Authentication module
│   │   ├── jwt.rs             # JWT token handling
│   │   ├── apikey.rs          # API key authentication
│   │   └── signature.rs       # HMAC-SHA256 signature generation
│   ├── models/
│   │   ├── mod.rs             # Data models
│   │   ├── user.rs            # User-related models
│   │   ├── app.rs             # App-related models
│   │   ├── world.rs           # World-related models
│   │   ├── asset.rs           # Asset-related models
│   │   └── device.rs          # Device-related models
│   ├── error.rs               # Error types
│   └── config.rs              # Configuration
├── examples/
│   ├── auth_flow.rs           # Authentication example
│   ├── app_management.rs      # App CRUD example
│   ├── world_management.rs    # World CRUD example
│   └── asset_operations.rs    # Asset operations example
└── tests/
    └── integration_tests.rs   # Integration tests
```

## Design Principles

1. **Type Safety**: Leverage Rust's type system to prevent runtime errors
2. **Async/Await**: Use tokio for async operations
3. **Builder Pattern**: Fluent API for constructing requests
4. **Error Handling**: Comprehensive error types with context
5. **Zero-Copy**: Minimize allocations where possible
6. **Testability**: Mock-friendly design for unit testing
7. **Modular Features**: Optional MP and APP clients via feature flags

## Core Types

### Client Structure

```rust
// Main SDK client
pub struct AbpilotClient {
    #[cfg(feature = "mp")]
    mp_client: MpClient,
    #[cfg(feature = "app")]
    app_client: AppClient,
    http_client: reqwest::Client,
}

// MP API client (enabled with "mp" feature)
#[cfg(feature = "mp")]
pub struct MpClient {
    base_url: String,
    auth: Option<AuthMethod>,
    http_client: reqwest::Client,
}

// APP API client (enabled with "app" feature)
#[cfg(feature = "app")]
pub struct AppClient {
    base_url: String,
    http_client: reqwest::Client,
}
```

### Authentication Types

```rust
pub enum AuthMethod {
    #[cfg(feature = "mp")]
    JwtToken(String),
    #[cfg(feature = "mp")]
    ApiKey(String),
    #[cfg(feature = "app")]
    AppSignature { app_id: String, secret: String },
    #[cfg(feature = "app")]
    WorldSignature { world_id: String, secret: String },
}

pub struct SignatureGenerator {
    secret: String,
}

impl SignatureGenerator {
    #[cfg(feature = "app")]
    pub fn generate_app_signature(&self, app_id: &str) -> (String, i64);
    #[cfg(feature = "app")]
    pub fn generate_world_signature(&self, world_id: &str) -> (String, i64);
}
```

### Model Types

```rust
// User models (MP feature)
#[cfg(feature = "mp")]
pub struct User {
    pub user_id: String,
    pub email: String,
    pub created_at: i64,
}

#[cfg(feature = "mp")]
pub struct AuthToken {
    pub token: String,
    pub user_id: String,
}

// API Key models (MP feature)
#[cfg(feature = "mp")]
pub struct ApiKey {
    pub apikey: String,
    pub name: String,
    pub created_at: Option<i64>,
}

// App models (MP feature)
#[cfg(feature = "mp")]
pub struct App {
    pub app_id: String,
    pub name: String,
    pub secret: Option<String>,
    pub created_at: Option<i64>,
}

// World models (MP feature)
#[cfg(feature = "mp")]
pub struct World {
    pub world_id: String,
    pub name: String,
    pub secret: Option<String>,
    pub created_at: Option<i64>,
}

// World Node models (APP feature)
#[cfg(feature = "app")]
pub struct WorldNode {
    pub world_id: String,
    pub base_url: String,
    pub tags: String,
}

// Asset models (APP feature)
#[cfg(feature = "app")]
pub struct Asset {
    pub r#type: String,
    pub id: String,
    pub value: i64,
}

// Device models (APP feature)
#[cfg(feature = "app")]
pub struct Device {
    pub device_id: String,
    pub world_id: String,
    pub info: serde_json::Value,
}

#[cfg(feature = "app")]
pub struct DeviceToken {
    pub token: String,
    pub items: Vec<WorldNodeInfo>,
}

#[cfg(feature = "app")]
pub struct WorldNodeInfo {
    pub base_url: String,
    pub tags: String,
}
```

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AbpilotError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Authentication failed: {0}")]
    AuthError(String),
    
    #[error("Invalid signature")]
    SignatureError,
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, AbpilotError>;
```

## API Design

### MP Client API

```rust
#[cfg(feature = "mp")]
impl MpClient {
    // Authentication
    pub async fn send_verification_code(&self, email: &str) -> Result<()>;
    pub async fn verify_code(&self, email: &str, code: &str) -> Result<AuthToken>;
    
    // API Key Management
    pub async fn create_api_key(&self, name: &str) -> Result<ApiKey>;
    pub async fn delete_api_key(&self, apikey: &str) -> Result<()>;
    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>>;
    
    // App Management
    pub async fn create_app(&self, name: &str) -> Result<App>;
    pub async fn delete_app(&self, app_id: &str) -> Result<()>;
    pub async fn list_apps(&self) -> Result<Vec<App>>;
    pub async fn reset_app_secret(&self, app_id: &str) -> Result<App>;
    pub async fn get_app_upload_urls(&self, app_id: &str, files: &[&str]) -> Result<Vec<String>>;
    pub async fn get_app_download_urls(&self, app_id: &str, files: &[&str]) -> Result<Vec<String>>;
    
    // World Management
    pub async fn create_world(&self, name: &str) -> Result<World>;
    pub async fn delete_world(&self, world_id: &str) -> Result<()>;
    pub async fn list_worlds(&self) -> Result<Vec<World>>;
    pub async fn get_world(&self, world_id: &str) -> Result<World>;
    pub async fn reset_world_secret(&self, world_id: &str) -> Result<World>;
    pub async fn get_world_upload_urls(&self, world_id: &str, files: &[&str]) -> Result<Vec<String>>;
    pub async fn get_world_download_urls(&self, world_id: &str, files: &[&str]) -> Result<Vec<String>>;
}
```

### APP Client API

```rust
#[cfg(feature = "app")]
impl AppClient {
    // Asset Management
    pub async fn list_assets(
        &self,
        app_id: &str,
        app_secret: &str,
        device_id: &str,
        world_id: &str,
    ) -> Result<Vec<Asset>>;
    
    pub async fn get_asset(
        &self,
        app_id: &str,
        app_secret: &str,
        device_id: &str,
        world_id: &str,
        asset_type: &str,
        asset_id: &str,
    ) -> Result<Asset>;
    
    pub async fn add_asset(
        &self,
        world_id: &str,
        world_secret: &str,
        device_id: &str,
        asset_type: &str,
        asset_id: &str,
        delta: i64,
    ) -> Result<Asset>;
    
    // World Node Management
    pub async fn update_world_node(
        &self,
        world_id: &str,
        world_secret: &str,
        base_url: &str,
        tags: &str,
    ) -> Result<WorldNode>;
    
    pub async fn delete_world_node(
        &self,
        world_id: &str,
        world_secret: &str,
        base_url: &str,
    ) -> Result<()>;
    
    // Device Management
    pub async fn create_device_token(
        &self,
        app_id: &str,
        app_secret: &str,
        world_id: &str,
        device_id: &str,
        info: serde_json::Value,
        ttl: u64,
    ) -> Result<DeviceToken>;
    
    pub async fn get_device_info(
        &self,
        world_id: &str,
        world_secret: &str,
        token: &str,
    ) -> Result<Device>;
}
```

### Builder Pattern for Complex Operations

```rust
#[cfg(feature = "app")]
pub struct AssetOperationBuilder<'a> {
    client: &'a AppClient,
    world_id: String,
    world_secret: String,
    device_id: String,
    asset_type: String,
    asset_id: String,
}

impl<'a> AssetOperationBuilder<'a> {
    pub fn new(client: &'a AppClient) -> Self;
    pub fn world(mut self, world_id: &str, world_secret: &str) -> Self;
    pub fn device(mut self, device_id: &str) -> Self;
    pub fn asset(mut self, asset_type: &str, asset_id: &str) -> Self;
    pub async fn add(self, delta: i64) -> Result<Asset>;
    pub async fn get(self) -> Result<Asset>;
}
```

## Usage Examples

### Authentication Flow

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
    
    // Create authenticated client
    let authed_client = client.with_auth(AuthMethod::JwtToken(auth_token.token));
    
    Ok(())
}
```

### App Management

```rust
// Create app
let app = authed_client.mp().create_app("My Game").await?;
println!("App ID: {}, Secret: {}", app.app_id, app.secret.unwrap());

// Upload files
let files = vec!["icon.png", "config.json"];
let upload_urls = authed_client.mp()
    .get_app_upload_urls(&app.app_id, &files)
    .await?;

// Upload file to S3
for (url, file) in upload_urls.iter().zip(files.iter()) {
    // Use reqwest to PUT file to presigned URL
}

// Download files (with app signature)
let download_urls = authed_client.mp()
    .get_app_download_urls(&app.app_id, &files)
    .await?;
```

### Asset Operations

```rust
// List assets
let assets = client.app()
    .list_assets(&app_id, &app_secret, "device_001", &world_id)
    .await?;

// Add gold
let updated_asset = client.app()
    .add_asset(&world_id, &world_secret, "device_001", "gold", "001", 100)
    .await?;

println!("New balance: {}", updated_asset.value);

// Using builder pattern
let asset = client.app()
    .asset_operation()
    .world(&world_id, &world_secret)
    .device("device_001")
    .asset("gold", "001")
    .add(100)
    .await?;
```

### Device Token Creation

```rust
use serde_json::json;

let device_info = json!({
    "platform": "ios",
    "version": "1.0",
    "device_model": "iPhone 14"
});

let device_token = client.app()
    .create_device_token(
        &app_id,
        &app_secret,
        &world_id,
        "device_001",
        device_info,
        3600, // 1 hour TTL
    )
    .await?;

println!("Token: {}", device_token.token);
for node in device_token.items {
    println!("Node: {} (tags: {})", node.base_url, node.tags);
}
```

## Feature Flags

The SDK supports optional features to reduce dependencies:

- **`mp`**: Enable MP API client (authentication, API keys, apps, worlds management)
- **`app`**: Enable APP API client (assets, world nodes, device management)
- **`full`**: Enable both MP and APP clients (default)

### Feature Dependencies

```toml
[features]
default = ["full"]
full = ["mp", "app"]
mp = []  # MP client feature
app = [] # APP client feature
```

### Usage Examples

```toml
# Use only MP client (for management operations)
[dependencies]
abpilot-cc-sdk = { version = "0.1", default-features = false, features = ["mp"] }

# Use only APP client (for runtime operations)
[dependencies]
abpilot-cc-sdk = { version = "0.1", default-features = false, features = ["app"] }

# Use both (default)
[dependencies]
abpilot-cc-sdk = "0.1"
```

## Dependencies

```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"
chrono = "0.4"

[dev-dependencies]
mockito = "1.2"
tokio-test = "0.4"
```

## Configuration

```rust
pub struct Config {
    #[cfg(feature = "mp")]
    pub mp_base_url: String,
    #[cfg(feature = "app")]
    pub app_base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            #[cfg(feature = "mp")]
            mp_base_url: "https://wpyi6ctkdvfcxbqtmy6d6tkesi0yzzid.lambda-url.us-east-1.on.aws".to_string(),
            #[cfg(feature = "app")]
            app_base_url: "https://opnqqwytt7sgobosrlk6kxp5de0rolbu.lambda-url.us-east-1.on.aws".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}
```

## Testing Strategy

1. **Unit Tests**: Test individual components (signature generation, serialization)
2. **Integration Tests**: Test against mock HTTP server (using mockito)
3. **Example Tests**: Ensure all examples compile and run
4. **Documentation Tests**: Test code snippets in documentation

## Security Considerations

1. **Secret Storage**: Never log or expose secrets in error messages
2. **Timestamp Validation**: Ensure signatures are generated with current timestamp
3. **HTTPS Only**: All requests must use HTTPS
4. **Token Expiry**: Handle token expiration gracefully
5. **Input Validation**: Validate all user inputs before sending to API

## Future Enhancements

1. **Retry Logic**: Automatic retry with exponential backoff
2. **Rate Limiting**: Client-side rate limiting to prevent API throttling
3. **Caching**: Cache frequently accessed data (apps, worlds)
4. **Streaming**: Support for large file uploads/downloads
5. **Webhooks**: Support for webhook signature verification
6. **Metrics**: Built-in metrics collection (request count, latency)
7. **Tracing**: OpenTelemetry integration for distributed tracing

## API Versioning

The SDK will follow semantic versioning:
- **Major**: Breaking API changes
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes

## License

MIT or Apache-2.0 (dual license)

## Contributing

Contributions welcome! Please follow Rust API guidelines and ensure all tests pass.
