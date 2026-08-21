# Changelog

All notable changes to the Accumulate Rust SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- N/A

### Changed
- N/A

### Fixed
- N/A

## [2.0.2] - 2026-02-07

### Fixed
- Verified all v3 examples against Kermit public testnet
- Removed duplicate custom tokens example (broken V2 faucet)

## [2.0.0] - 2024-12-30

### Added
- **QuickStart API**: Ultra-simple one-liner SDK usage with `QuickStart::kermit()`
- **SmartSigner**: Automatic signer version tracking and transaction management
- **TxBody Builders**: Complete transaction body builders for all operations
- **Key Management**: Full key page operations (add/remove keys, set thresholds)
- **Multi-Signature Support**: Complete multi-sig workflow with threshold management
- **Query Operations**: Comprehensive query support for accounts, transactions, and network status
- **Custom Tokens**: Token creation, issuance, and transfer support
- **Data Accounts**: Data account creation and entry writing
- **12 Complete Examples**: Production-ready examples covering all SDK features
- **Kermit Testnet Support**: Built-in constants for Kermit testnet endpoints
- **V3 Faucet Integration**: Working faucet support via V3 API
- **Polling Utilities**: `poll_for_balance()` and `poll_for_credits()` helpers

### Changed
- **Major Version Bump**: Version 2.0.0 reflects production-ready status
- **README Overhaul**: Clean, focused documentation matching Dart SDK style
- **Examples Reorganized**: Renamed to `example_NN_description.rs` format
- **All Examples Use Kermit**: Testnet-first approach for immediate usability

### Fixed
- **UpdateKeyPage Encoding**: Fixed binary encoding for key management operations
- **V3 Faucet in QuickStart**: Fixed `fund_wallet()` to use V3 API
- **Query Operation Timing**: Added proper polling for account availability
- **IssueTokens Transaction**: Removed deprecated fields from encoding

### Removed
- **Python Files**: Removed misplaced Python files from src directory
- **Disabled Binaries**: Cleaned up `.disabled` files
- **Emojis**: Removed all emojis from source code and tests

### Security
- **No Hardcoded Keys**: Verified no production keys in codebase
- **No Debug Prints**: Removed all debug print statements
- **Stub Audit**: Verified no security bypass stubs in signature verification

## [0.1.0] - Initial Development

### Added
- Initial Rust SDK implementation with V2/V3 unified support
- DevNet discovery binary for automatic configuration
- TypeScript SDK parity with byte-for-byte compatibility
- Canonical JSON encoding matching TS implementation
- Ed25519 cryptographic utilities for signing and verification
- Transaction envelope creation and verification
- Integration tests for DevNet compatibility
- Conformance tests against TypeScript SDK fixtures
- GitHub Actions CI/CD with multi-platform testing
- Code coverage reporting with cargo-llvm-cov
- Production-ready linting and formatting configuration

---

## Guidelines for Maintainers

When releasing a new version:

1. Update the version number in `Cargo.toml`
2. Move items from `[Unreleased]` to a new version section
3. Add a new empty `[Unreleased]` section
4. Include the release date in ISO format (YYYY-MM-DD)
5. Create a git tag with the version number (e.g., `v2.0.0`)

### Categories

- **Added** for new features
- **Changed** for changes in existing functionality
- **Deprecated** for soon-to-be removed features
- **Removed** for now removed features
- **Fixed** for any bug fixes
- **Security** in case of vulnerabilities
