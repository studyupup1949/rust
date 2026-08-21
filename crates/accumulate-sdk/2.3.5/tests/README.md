# Test Suite

Comprehensive test suite for the Accumulate Rust SDK, organized by functional concern and test type.

## Test Structure

```
tests/
├── unit/               # Unit tests for individual components
│   ├── runtime/       # Runtime helper tests
│   └── protocol/      # Protocol functionality tests
├── conformance/       # Protocol conformance tests
│   ├── hash_vectors_test.rs     # Hash conformance with golden vectors
│   └── canonical_json_test.rs   # JSON canonicalization conformance
├── integration/       # End-to-end integration tests
│   └── zero_to_hero_devnet_test.rs  # Complete workflow testing
├── coverage/          # Coverage-specific tests
├── fuzz/              # Fuzzing and property-based tests
├── quarantine/        # Experimental or broken tests (excluded by default)
├── specialized/       # Specialized test categories
├── support/           # Test utilities and helpers
├── golden/            # Golden master test vectors
└── builders/          # Transaction builder tests
```

## Running Tests

### All Tests
```bash
cargo test
```

### By Category
```bash
# Unit tests only
cargo test --test "unit/*"

# Conformance tests only
cargo test conformance

# Integration tests only (requires DevNet)
cargo test integration

# Specific test patterns
cargo test hash_vectors
cargo test canonical_json
```

### With Coverage
```bash
# Generate coverage report
cargo llvm-cov --all-features --html

# Coverage with specific threshold
cargo llvm-cov --all-features --summary-only
```

### DevNet Integration Tests
```bash
# Note: These require a running DevNet instance
cargo test integration --ignored

# Or run specific DevNet tests
cargo test zero_to_hero_devnet_test --ignored
```

## Test Categories

### Unit Tests (`unit/`)
Fast, isolated tests for individual components:

- **Runtime (`runtime/`)**: Helper functions, validation utilities, and basic functionality
- **Protocol (`protocol/`)**: Enum tests, edge cases, performance, stability, and roundtrip validation

Key test files:
- `enum_edge_case_tests.rs` (403 lines) - Edge case validation
- `enum_integration_tests.rs` (466 lines) - Integration scenarios
- `enum_performance_tests.rs` (454 lines) - Performance benchmarks
- `enum_property_tests.rs` (454 lines) - Property-based testing
- `enum_stability_tests.rs` (449 lines) - Stability validation
- `enum_roundtrip_tests.rs` (210 lines) - Roundtrip verification

### Conformance Tests (`conformance/`)
Tests against protocol specifications and cross-language compatibility:

- **Hash Vectors**: SHA-256 hash conformance with TypeScript golden files
- **Canonical JSON**: Deterministic JSON encoding matches TypeScript exactly
- **Binary Encoding**: Protocol buffer encoding/decoding verification
- **Transaction Envelopes**: Complete envelope structure validation

### Integration Tests (`integration/`)
End-to-end tests with external dependencies:

- **DevNet E2E**: Full workflow testing against local DevNet
- **Zero-to-Hero**: Complete user journey from key generation to transactions
- **Network Connectivity**: Basic endpoint validation and error handling

### Specialized Tests

- **Fuzz (`fuzz/`)**: Property-based testing and randomized input validation
- **Coverage (`coverage/`)**: Tests designed specifically for coverage analysis
- **Golden (`golden/`)**: Reference test vectors and golden master comparisons

## Test Data and Fixtures

### Golden Files (`golden/`)
Reference test vectors for conformance testing:
- TypeScript compatibility vectors (JSONL format)
- Protocol-specific test data
- Hash and signature verification vectors

### Test Utilities (`support/`)
- Path resolution helpers
- Test data generators
- Common test patterns and macros

## Configuration and Environment

### Environment Variables
- `RUST_LOG` - Logging level (debug, info, warn, error)
- `ACC_RPC_URL_V2` - V2 API endpoint override
- `ACC_RPC_URL_V3` - V3 API endpoint override
- `ACC_DEVNET_DIR` - DevNet directory for integration tests

### Test Tags and Filtering
```bash
# Skip integration tests (default behavior)
cargo test

# Run only ignored tests (DevNet required)
cargo test -- --ignored

# Run with specific threads
cargo test -- --test-threads=1

# Verbose output
cargo test -- --nocapture
```

## Cross-Language Compatibility

### TypeScript Parity Testing
The SDK maintains byte-for-byte compatibility with TypeScript SDK:

```bash
# Generate TypeScript test vectors
cd tooling/ts-fixture-exporter/
TS_FUZZ_N=1000 node export-random-vectors.js > ../../tests/golden/ts_rand_vectors.jsonl

# Run parity tests
cargo test ts_fuzz_roundtrip
```

Verification includes:
- **Binary Roundtrip**: Identical encoding/decoding
- **Canonical JSON**: Deterministic serialization
- **Hash Calculations**: SHA-256 hash parity
- **Signature Verification**: Ed25519 signature compatibility

### Parity Gate Pipeline
```bash
# Run complete parity validation
scripts/run_parity_gate.ps1

# Custom configuration
scripts/run_parity_gate.ps1 -FuzzCount 2000 -CoverageThreshold 80
```

## Writing Tests

### Test Placement Guidelines
- **Unit tests**: Test single functions/structs in isolation
- **Conformance**: Test against external specifications or golden files
- **Integration**: Test complete workflows requiring external services

### Test Patterns
```rust
use accumulate_client::*;

#[test]
fn test_component_behavior() {
    // Arrange
    let input = create_test_data();

    // Act
    let result = function_under_test(input);

    // Assert
    assert_eq!(result.unwrap(), expected_output);
}

#[tokio::test]
async fn test_async_functionality() {
    let client = create_test_client().await;
    let result = client.some_operation().await;
    assert!(result.is_ok());
}
```

### Golden File Tests
```rust
#[test]
fn test_golden_compatibility() {
    let test_vectors = load_golden_file("test_vectors.jsonl");
    for vector in test_vectors {
        let result = process_test_vector(&vector);
        assert_eq!(result.hash, vector.expected_hash);
    }
}
```

## Test Quality Standards

- **Coverage**: Aim for high coverage of critical paths (>70% overall, >85% for critical modules)
- **Isolation**: Unit tests should not depend on external services
- **Speed**: Unit and conformance tests should run quickly (<1s each)
- **Reliability**: Integration tests should be resilient to network issues
- **Clarity**: Test names should clearly describe what is being tested
- **Determinism**: Tests should produce consistent results across runs

## DevNet Test Requirements

Integration tests marked with `#[ignore]` require a running DevNet instance:

1. **Start DevNet**:
   ```bash
   cd /path/to/devnet-accumulate-instance
   ./start-devnet.sh
   ```

2. **Verify connectivity**:
   ```bash
   curl http://localhost:26660/v2/status
   curl http://localhost:26661/v3/status
   ```

3. **Run integration tests**:
   ```bash
   cargo test integration --ignored
   ```

These tests validate:
- Complete transaction workflows
- Network connectivity and error handling
- Faucet integration
- Multi-step operations (key generation → funding → transactions)

## Debugging Tests

### Detailed Output
```bash
# Show test output
cargo test -- --nocapture

# Show debug logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Single threaded execution
cargo test -- --test-threads=1
```

### Test-Specific Debugging
```rust
#[test]
fn debug_test() {
    env_logger::init(); // Enable logging in test
    let result = function_under_test();
    dbg!(&result); // Debug print
    assert!(result.is_ok());
}
```