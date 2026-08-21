# Examples

Complete working examples for the Accumulate Rust SDK. All examples run against the Kermit testnet.

## Directory Layout

```
examples/
├── v3/          # Core V3 API examples (start here)
├── misc/        # Additional / legacy examples
└── README.md
```

## Quick Start

```bash
# Run the QuickStart demo (simplest introduction)
cargo run --example example_11_quickstart_demo

# Run lite identity example
cargo run --example example_01_lite_identities
```

## V3 Examples (`examples/v3/`)

| # | Example | Description |
|---|---------|-------------|
| 01 | `example_01_lite_identities` | Generate keys, derive lite URLs, fund via faucet |
| 02 | `example_02_adi_creation` | Create ADI with key book and key page |
| 03 | `example_03_token_accounts` | Create and manage token accounts |
| 04 | `example_04_data_accounts` | Create data accounts and write entries |
| 05 | `example_05_adi_to_adi_transfer` | Transfer tokens between ADI accounts |
| 06 | `example_06_custom_tokens` | Create custom tokens with issuance |
| 07 | `example_07_query_operations` | Query accounts, transactions, key books |
| 08 | `example_08_query_transactions` | Transaction querying patterns |
| 09 | `example_09_key_management` | Add/remove keys from key pages |
| 10 | `example_10_threshold_updates` | Update multi-sig thresholds |
| 11 | `example_11_quickstart_demo` | Ultra-simple QuickStart API demo |
| 12 | `example_12_multi_signature_workflow` | Complete multi-sig transaction flow |
| 13 | `example_13_adi_to_adi_transfer_with_header_options` | ADI transfers with memo, metadata, expire, hold_until, authorities |

### Example Categories

#### Beginner
Start here to learn the SDK basics:
- `example_01_lite_identities` - Key generation and lite accounts
- `example_11_quickstart_demo` - Simplest possible SDK usage

#### Identity & Accounts
- `example_02_adi_creation` - Create Accumulate Digital Identifiers
- `example_03_token_accounts` - Token account management
- `example_04_data_accounts` - Data storage on-chain

#### Token Operations
- `example_05_adi_to_adi_transfer` - Token transfers
- `example_06_custom_tokens` - Custom token creation

#### Queries
- `example_07_query_operations` - Network and account queries
- `example_08_query_transactions` - Transaction lookups

#### Key Management (Advanced)
- `example_09_key_management` - Key page operations
- `example_10_threshold_updates` - Multi-sig configuration

## Misc Examples (`examples/misc/`)

| Example | Description |
|---------|-------------|
| `100_keygen_lite_urls` | Key generation and lite URL derivation |
| `120_faucet_local_devnet` | Faucet usage on local DevNet |
| `210_buy_credits_lite` | Buy credits with a lite account |
| `220_create_adi_v3` | ADI creation via V3 API |
| `270_write_data` | Write data entries |
| `280_send_tokens_lta_to_adi` | Send tokens from lite to ADI |
| `999_zero_to_hero` | End-to-end walkthrough |
| `basic_usage` | Basic SDK usage patterns |
| `transaction_demo` | Transaction submission demo |

## Network Configuration

All examples use the **Kermit testnet** by default:
- V2 API: `https://kermit.accumulatenetwork.io/v2`
- V3 API: `https://kermit.accumulatenetwork.io/v3`

For local DevNet, modify the endpoint URLs in the example code:
```rust
let v2_url = Url::parse("http://localhost:26660/v2")?;
let v3_url = Url::parse("http://localhost:26661/v3")?;
```

## Running Examples

```bash
# Run a specific example
cargo run --example example_01_lite_identities

# Build all examples (check for compilation errors)
cargo build --examples

# Run with debug output
RUST_LOG=debug cargo run --example example_07_query_operations
```

## Example Output

Each example prints progress and results to stdout. Example output from `example_11_quickstart_demo`:

```
============================================================
  QuickStart Demo - Ultra-Simple Accumulate SDK Usage
============================================================

>>> Step 1: Connect to Kermit Testnet
    Connected to Kermit Testnet

>>> Step 2: Create Wallet
    Lite Identity:      acc://abc123.../
    Lite Token Account: acc://abc123.../ACME

>>> Step 3: Fund Wallet (faucet x5, wait 15s)
    Balance: Some(5000000000) ACME tokens

>>> Step 4: Create ADI (one call does everything!)
    ADI URL:      acc://quickstart-1234567890.acme
    Key Book:     acc://quickstart-1234567890.acme/book
    Key Page:     acc://quickstart-1234567890.acme/book/1
...
```

## Contributing Examples

When adding new examples:

1. Place V3 API examples in `examples/v3/` following naming: `example_NN_description.rs`
2. Place other/experimental examples in `examples/misc/`
3. Use SmartSigner or QuickStart APIs
4. Include progress output with `println!`
5. Handle errors gracefully
6. Test against Kermit testnet
7. Update this README and add a `[[example]]` entry in `Cargo.toml`
