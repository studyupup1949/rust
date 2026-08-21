# Generated Files

All generated files for the OpenDLT Accumulate Rust SDK. These files are automatically created from the Accumulate Go repository protocol definitions and should never be manually edited.

## ⚠️ Important

- **DO NOT** edit any files in this directory
- Files are regenerated from external sources (Accumulate Go repository)
- Manual changes will be overwritten during regeneration
- All files contain generation markers and timestamps

## Structure

```
generated/
├── enums.rs           # Protocol enum definitions
├── transactions.rs    # Transaction type definitions
├── tx_types.rs       # Transaction type mappings
├── tx_bodies.rs      # Transaction body structures
├── tx_envelope.rs    # Transaction envelope definitions
├── api_methods.rs    # API method definitions
└── *.json            # Generation metadata and manifests
```

## File Purposes

### Core Types
- **`enums.rs`** - All protocol enums (TransactionType, SignatureType, etc.)
- **`transactions.rs`** - Complete transaction type definitions
- **`tx_types.rs`** - Transaction type mappings and utilities
- **`tx_bodies.rs`** - Transaction body structures for all transaction types
- **`tx_envelope.rs`** - Transaction envelope and signature structures

### API Layer
- **`api_methods.rs`** - Generated API method definitions and JSON-RPC interfaces

### Metadata
- **`*_manifest.json`** - Generation metadata, type counts, and validation data
- Used for verification and regeneration tracking

## Regeneration Process

### Prerequisites
- Access to Accumulate Go repository
- Python 3.8+ with required dependencies
- Clean workspace (no uncommitted changes in generated files)

### Environment Variables
```bash
# Required: Path to Accumulate Go repository
export ACCUMULATE_REPO="/path/to/accumulate"

# Optional: Generation configuration
export RUST_SDK_ROOT="/path/to/opendlt-rust-v2v3-sdk/unified"
```

### Regeneration Steps

1. **Clean existing generated files**:
   ```bash
   rm -f src/generated/*.rs src/generated/*.json
   ```

2. **Run the generation pipeline**:
   ```bash
   # From the tooling directory
   cd tooling/backends/
   python rust_enum_generator.py
   python rust_transaction_generator.py
   python rust_api_generator.py
   ```

3. **Verify generation**:
   ```bash
   # Check all expected files are present
   ls src/generated/

   # Verify compilation
   cargo check
   ```

4. **Format generated code**:
   ```bash
   cargo fmt
   ```

### Generated File Markers

All generated files contain markers like:
```rust
// GENERATED FILE - DO NOT EDIT
// Generated from: /path/to/accumulate/protocol/definitions
// Generation time: 2024-01-01 12:00:00 UTC
// Generator: rust_enum_generator.py v1.0
```

## Integration with SDK

### Import Patterns
```rust
// From SDK code to generated types
use crate::generated::{
    TransactionType, SignatureType, AccountType,
    Transaction, TransactionBody, Envelope
};

// Generated enums are re-exported through lib.rs
use accumulate_client::{TransactionType, SignatureType};
```

### Usage Examples
```rust
use crate::generated::*;

// Enum usage
let tx_type = TransactionType::SendTokens;
assert_eq!(serde_json::to_string(&tx_type)?, "\"sendTokens\"");

// Transaction construction
let tx_body = TransactionBody::SendTokens {
    to: vec![TokenRecipient {
        url: "acc://bob.acme/tokens".to_string(),
        amount: "1000".to_string(),
    }],
};
```

## Type Safety and Validation

Generated types include:
- **Serde serialization/deserialization** with exact wire format compatibility
- **Validation constraints** from protocol specifications
- **Type-safe enums** with exhaustive pattern matching
- **Optional field handling** matching protocol requirements

## Cross-Language Compatibility

Generated code maintains compatibility with:
- **TypeScript SDK**: Identical JSON serialization
- **Go Protocol**: Source of truth for type definitions
- **Protocol Specifications**: Exact field names and constraints

## Generation Metadata

Manifest files track:
- **Type counts**: Number of enums, transactions, fields
- **Generation timestamps**: When files were last generated
- **Source hashes**: Verification of source protocol files
- **Validation data**: Test vectors and expected outputs

Example manifest structure:
```json
{
  "enums": [
    {
      "name": "TransactionType",
      "variants": ["unknown", "sendTokens", "createIdentity", ...],
      "variant_count": 42
    }
  ],
  "generation_time": "2024-01-01T12:00:00Z",
  "source_hash": "abc123...",
  "generator_version": "1.0"
}
```

## Troubleshooting

### Common Issues

1. **Compilation errors after regeneration**:
   - Run `cargo fmt` to fix formatting
   - Check for missing dependencies in Cargo.toml
   - Verify all files were generated successfully

2. **Import resolution failures**:
   - Ensure lib.rs exports are updated
   - Check for circular dependencies
   - Verify module declarations

3. **Serialization mismatches**:
   - Check source protocol definitions haven't changed
   - Verify serde attributes are correctly applied
   - Run conformance tests to validate compatibility

### Debugging Generation
```bash
# Verbose generation output
RUST_LOG=debug python rust_enum_generator.py

# Validate generated code structure
cargo expand --bin main > expanded.rs  # Requires cargo-expand

# Test generated types
cargo test generated_types
```

## Version Compatibility

Generated files track compatibility with:
- **Protocol Version**: Accumulate protocol version used for generation
- **Generator Version**: Version of generation scripts
- **SDK Version**: Target SDK version for generated code

Ensure all components are compatible before regenerating files.