# TypeScript Fixture Exporter for Rust Fuzzing

This directory contains tools for generating deterministic test vectors from TypeScript that are used to verify byte-for-byte compatibility between the TypeScript and Rust SDKs.

## Files

- `export-random-vectors.js` - Main fuzzing vector generator
- `export-fixtures.js` - Static fixture generator
- `package.json` - Node.js dependencies

## Usage

### Generate Random Vectors for Fuzzing

```bash
# Generate default 1000 vectors
node export-random-vectors.js > ../../tests/golden/ts_rand_vectors.jsonl

# Generate custom count via environment variable
TS_FUZZ_N=200 node export-random-vectors.js > ../../tests/golden/ts_rand_vectors.jsonl

# CI environment (200 vectors)
TS_FUZZ_N=200 node export-random-vectors.js > ../../tests/golden/ts_rand_vectors.jsonl

# Local development (up to 2000 vectors)
TS_FUZZ_N=2000 node export-random-vectors.js > ../../tests/golden/ts_rand_vectors.jsonl
```

### Run Rust Fuzz Tests

```bash
# Basic roundtrip test (works without vectors file)
cargo test --test ts_fuzz_roundtrip test_fallback_basic_roundtrip

# Full fuzz test suite (requires vectors file)
cargo test --test ts_fuzz_roundtrip

# Specific test categories
cargo test --test ts_fuzz_roundtrip test_ts_fuzz_roundtrip_binary
cargo test --test ts_fuzz_roundtrip test_ts_fuzz_roundtrip_canonical_json
cargo test --test ts_fuzz_roundtrip test_ts_fuzz_roundtrip_hashes
```

## Output Format

Each line in the JSON Lines output contains:

```json
{
  "hexBin": "4143435500...",
  "canonicalJson": "{\"body\":{...}}",
  "txHashHex": "4be49c59c717...",
  "meta": {
    "index": 0,
    "txType": "send-tokens",
    "numSignatures": 1,
    "binarySize": 445,
    "canonicalSize": 161
  }
}
```

### Fields

- `hexBin` - Binary envelope encoded as hex string
- `canonicalJson` - Canonical JSON of the transaction
- `txHashHex` - SHA-256 hash of canonical JSON (hex)
- `meta.index` - Vector sequence number
- `meta.txType` - Transaction type
- `meta.numSignatures` - Number of signatures in envelope
- `meta.binarySize` - Size of binary data in bytes
- `meta.canonicalSize` - Size of canonical JSON in bytes

## Verification Process

The Rust fuzz tests verify:

1. **Binary Roundtrip**: `decode(hexBin) → structs → encode() == hexBin`
2. **Canonical JSON**: `dumps_canonical(transaction) == canonicalJson`
3. **Hash Verification**: `sha256_json(transaction) == txHashHex`

## Deterministic Generation

Uses a fixed seed PRNG (`accumulate-rust-ts-fuzz-seed-2024`) to ensure:
- Reproducible test vectors across runs
- Consistent CI/CD testing
- Deterministic debugging

## Transaction Types Generated

- `send-tokens` - Token transfers
- `create-identity` - Identity creation
- `create-token-account` - Token account creation
- `create-data-account` - Data account creation
- `write-data` - Data writing
- `create-key-page` - Key page creation
- `create-key-book` - Key book creation
- `add-credits` - Credit additions
- `update-key` - Key updates
- `system-genesis` - Genesis transactions

## Environment Variables

- `TS_FUZZ_N` - Number of vectors to generate (default: 1000, max: 2000)

## CI Configuration

For CI environments, use `TS_FUZZ_N=200` to balance test coverage with execution time.