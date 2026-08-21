# OpenDLT Accumulate Rust SDK

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/accumulate-client.svg)](https://crates.io/crates/accumulate-client)

Production-ready Rust SDK for the Accumulate blockchain protocol. Supports all signature types, V2/V3 API endpoints, and provides a high-level signing API with automatic version tracking.

## Features

- **Multi-Signature Support**: Ed25519, RCD1, BTC, ETH, RSA-SHA256, ECDSA-SHA256
- **Smart Signing**: Automatic signer version tracking with `SmartSigner`
- **Complete Protocol**: All transaction types and account operations
- **Async/Await**: Modern async Rust with tokio runtime
- **Network Ready**: Mainnet, Testnet (Kermit), and local DevNet support

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
accumulate-client = "2.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

```rust
use accumulate_client::{
    AccumulateClient, AccOptions, derive_lite_identity_url,
    KERMIT_V2, KERMIT_V3,
};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Kermit testnet
    let v2_url = Url::parse(KERMIT_V2)?;
    let v3_url = Url::parse(KERMIT_V3)?;
    let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default()).await?;

    // Generate key pair and derive lite account URLs
    let keypair = AccumulateClient::generate_keypair();
    let public_key = keypair.verifying_key().to_bytes();
    let lite_identity = derive_lite_identity_url(&public_key);
    let lite_token_account = format!("{}/ACME", lite_identity);

    println!("Lite Identity: {}", lite_identity);
    println!("Lite Token Account: {}", lite_token_account);

    Ok(())
}
```

## Smart Signing API

The `SmartSigner` class handles version tracking automatically:

```rust
use accumulate_client::{
    AccumulateClient, AccOptions, SmartSigner, TxBody,
    derive_lite_identity_url, KERMIT_V2, KERMIT_V3,
};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let v2_url = Url::parse(KERMIT_V2)?;
    let v3_url = Url::parse(KERMIT_V3)?;
    let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default()).await?;

    let keypair = AccumulateClient::generate_keypair();
    let public_key = keypair.verifying_key().to_bytes();
    let lite_identity = derive_lite_identity_url(&public_key);
    let lite_token_account = format!("{}/ACME", lite_identity);

    // Create SmartSigner - automatically queries and tracks signer version
    let mut signer = SmartSigner::new(&client, keypair, &lite_identity);

    // Sign, submit, and wait for delivery in one call
    let result = signer.sign_submit_and_wait(
        &lite_token_account,
        &TxBody::send_tokens_single("acc://recipient.acme/tokens", "100000000"),
        Some("Payment"),
        30, // timeout seconds
    ).await;

    if result.success {
        println!("Transaction delivered: {:?}", result.txid);
    }

    Ok(())
}
```

## QuickStart API

For the simplest possible experience, use `QuickStart`:

```rust
use accumulate_client::QuickStart;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Kermit testnet (one line!)
    let acc = QuickStart::kermit().await?;

    // Create a wallet (one line!)
    let wallet = acc.create_wallet();
    println!("Lite Token Account: {}", wallet.lite_token_account);

    // Fund from faucet (one line!)
    acc.fund_wallet(&wallet, 5).await?;

    // Create ADI with automatic credit purchase (one line!)
    let adi = acc.setup_adi(&wallet, "my-adi").await?;
    println!("ADI Created: {}", adi.url);

    acc.close();
    Ok(())
}
```

## Supported Signature Types

| Type | Description | Use Case |
|------|-------------|----------|
| Ed25519 | Default signature type | Recommended for all new accounts |
| LegacyED25519 | Legacy Ed25519 format | Backward compatibility |
| RCD1 | Factom RCD1 signature | Factom ecosystem compatibility |
| BTC | Bitcoin secp256k1 | Bitcoin ecosystem integration |
| ETH | Ethereum secp256k1 + keccak256 | Ethereum ecosystem integration |
| RSA-SHA256 | RSA with SHA-256 | Enterprise/legacy systems |
| ECDSA-SHA256 | ECDSA P-256 curve | Standard ECDSA operations |

## Transaction Builders

Build transactions using the `TxBody` struct:

```rust
use accumulate_client::TxBody;

// Send tokens
TxBody::send_tokens_single("acc://recipient.acme/tokens", "100000000");

// Add credits
TxBody::add_credits("acc://my-identity.acme", "1000000", oracle_price);

// Create ADI
TxBody::create_identity("acc://my-adi.acme", "acc://my-adi.acme/book", &key_hash_hex);

// Create token account
TxBody::create_token_account("acc://my-adi.acme/tokens", "acc://ACME");

// Create custom token
TxBody::create_token("acc://my-adi.acme/mytoken", "MTK", 8, None);

// Write data
TxBody::write_data(&["entry1_hex", "entry2_hex"]);

// Key management
TxBody::update_key_page_add_key(&key_hash_bytes);
TxBody::update_key_page_set_threshold(2);
```

## Network Endpoints

```rust
use accumulate_client::{AccumulateClient, AccOptions, KERMIT_V2, KERMIT_V3};
use url::Url;

// Kermit Testnet (recommended for development)
let client = AccumulateClient::new_with_options(
    Url::parse(KERMIT_V2)?,
    Url::parse(KERMIT_V3)?,
    AccOptions::default(),
).await?;

// Local DevNet
let client = AccumulateClient::new_with_options(
    Url::parse("http://localhost:26660/v2")?,
    Url::parse("http://localhost:26661/v3")?,
    AccOptions::default(),
).await?;

// Custom endpoint
let client = AccumulateClient::new_with_options(
    Url::parse("https://your-node.com/v2")?,
    Url::parse("https://your-node.com/v3")?,
    AccOptions::default(),
).await?;
```

## Examples

See [`examples/`](examples/) for complete working examples:

| Example | Description |
|---------|-------------|
| `example_01_lite_identities` | Lite identity and token account operations |
| `example_02_adi_creation` | ADI creation with key books |
| `example_03_token_accounts` | Token account management |
| `example_04_data_accounts` | Data account operations |
| `example_05_adi_to_adi_transfer` | ADI-to-ADI token transfers |
| `example_06_custom_tokens` | Custom token creation and issuance |
| `example_07_query_operations` | Query accounts, transactions, and network status |
| `example_08_query_transactions` | Transaction querying patterns |
| `example_09_key_management` | Key page operations (add/remove keys) |
| `example_10_threshold_updates` | Multi-sig threshold management |
| `example_11_quickstart_demo` | Ultra-simple QuickStart API demo |
| `example_12_multi_signature_workflow` | Complete multi-sig workflow |

Run any example:
```bash
cargo run --example example_01_lite_identities
cargo run --example example_11_quickstart_demo
```

## Project Structure

```
src/
├── lib.rs              # Public API facade
├── client.rs           # AccumulateClient implementation
├── helpers.rs          # SmartSigner, TxBody, QuickStart, utilities
├── json_rpc_client.rs  # V2/V3 JSON-RPC client
├── codec/              # Binary encoding (TLV format)
├── crypto/             # Ed25519 and signature implementations
├── generated/          # Protocol types from YAML definitions
└── protocol/           # Transaction and envelope builders
examples/               # Complete working examples
tests/
├── unit/               # Unit tests
├── integration/        # Network integration tests
└── conformance/        # Cross-implementation compatibility
```

## Development

### Running Tests
```bash
cargo test                           # All tests
cargo test --lib                     # Library tests only
cargo test --test integration_tests  # Integration tests (requires network)
```

### Code Quality
```bash
cargo fmt                            # Format code
cargo clippy                         # Run linter
cargo doc --no-deps --open           # Generate and view docs
```

### Building Examples
```bash
cargo build --examples               # Build all examples
cargo run --example example_11_quickstart_demo
```

## Error Handling

All API methods return `Result<T, JsonRpcError>`:

```rust
use accumulate_client::JsonRpcError;

match client.v3_client.call_v3::<Value>("query", params).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(JsonRpcError::Http(e)) => eprintln!("HTTP error: {}", e),
    Err(JsonRpcError::Rpc { code, message }) => {
        eprintln!("RPC error {}: {}", code, message)
    },
    Err(e) => eprintln!("Other error: {}", e),
}
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Links

- [Accumulate Protocol](https://accumulatenetwork.io/)
- [API Documentation](https://docs.accumulatenetwork.io/)
- [Kermit Testnet Explorer](https://kermit.explorer.accumulatenetwork.io/)
- [Crates.io Package](https://crates.io/crates/accumulate-client)
