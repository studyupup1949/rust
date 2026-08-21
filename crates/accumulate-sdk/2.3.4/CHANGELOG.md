# Changelog

All notable changes to the Accumulate Rust SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.3.4] - 2026-07-30

### Added
- **Structured CLI** (`accumulate`): 13 verbs, `--json` emits exactly one envelope
  object on stdout, canonical `ACC_*` error codes with a `retryable` flag, and exit
  codes 0/1/2/3 an agent can branch on without parsing. `accumulate --help --json`
  returns the whole command tree. Defaults to testnet; mainnet requires both
  `--network mainnet` and `ACCUMULATE_ALLOW_MAINNET=1`.
  Conforms to `CLI-SPEC.md`; verified by a shared conformance suite across all five SDKs.
- `llms.txt` and `AGENTS.md` now document the CLI.

## [2.3.3] - 2026-07-29

### Added
- Canonical Accumulate error catalog in `llms-full.txt`: every error code with its
  category, whether a retry is productive (`retryable`), likely causes, the concrete
  fix, and the Rust type to catch. Each operation now lists the errors it can raise.
- `.devcontainer/devcontainer.json` pinning this repo's toolchain, defaulting to the
  Kermit testnet and carrying no credentials.

### Fixed
- `AGENTS.md` setup, test and layout paths now match this repository's actual root.
  They previously instructed agents to `cd` into a subdirectory that does not exist
  in a fresh clone, so the very first setup command failed.

## [2.3.2] - 2026-07-29

### Fixed
- **Submit-time rejections were silently swallowed.** `sign_submit_and_wait` extracted the transaction id from the `submit` response and went straight to polling, never inspecting the per-submission `status` for an error. A network rejection (e.g. `unauthorized`) therefore surfaced only as a timeout after `max_attempts` — 60 x 2s — with no reason attached, discarding the actionable error at the moment it arrived. Both submit paths now return it immediately.

### Added
- `llms.txt` documents that custom tokens carry their own precision, configured when the token issuer is created. It is not 1e8, and issuing `1000` against a precision-8 token mints `0.00001` tokens while the transaction still succeeds.

## [2.3.1] - 2026-07-28

### Fixed
- **Library helpers no longer print to stdout.** `poll_for_balance` and the faucet helpers wrote progress with `println!`, which corrupts any caller parsing stdout as data. They now emit through `tracing`, so consumers decide whether to surface them.
- `build.rs` propagates errors instead of `expect()`. `[lints.clippy] expect_used = "deny"` applies to every target, so clippy failed on the build script and never reached the library — meaning `make lint` and `make ci-check` had never actually run against library code.
- Replaced two `unwrap()` calls in the Merkle anchor fold with explicit matches. Signing bytes are unchanged (`golden_bytes_stable` passes).

### Added
- `tracing` as a runtime dependency (previously only `tracing-subscriber`, and only for dev).

## [2.3.0] - 2026-07-28

### Added
- `Amount::token(whole_tokens, precision)` and `Amount::to_token(precision)` for **custom tokens**. Custom tokens declare their own precision at creation; the wire format is always base units. Previously `Amount` covered only ACME (`acme`, `base_units`, `credits`), so issuing a custom token meant hand-computing a power of ten.

### Fixed
- `examples/v3/example_06_custom_tokens.rs` labelled a base-unit literal as "tokens" (`Issuing 10000 tokens (100.00 RUST)`). The arithmetic was right but the wording taught the wrong model: an AI agent following this example issued `1000` against a precision-8 token and minted `0.00001` tokens instead of 1000, and the transaction succeeded. The example now uses `Amount::token` and prints both readings.

## [2.1.0] - 2026-02-27

### Added
- Binary encoding for CreateKeyPage, BurnTokens, CreateKeyBook, UpdateKey, BurnCredits, TransferCredits, WriteDataTo, LockAccount, and UpdateAccountAuth transaction types
- `compute_write_data_to_body_hash` for correct WriteDataTo transaction hashing
- `TxBody::write_data_to_hex` builder method
- `marshal_body_to_binary` dispatch for all newly encoded transaction types
- `account_auth_op_types` constants matching Go protocol

### Fixed
- Corrected tx_type constant values (LockAccount, BurnCredits, TransferCredits, UpdateAccountAuth, UpdateKey) to match Go protocol
- Fixed `create_key_page` field name from `publicKeyHash` to `keyHash`
- SmartSigner now handles WriteDataTo separately from WriteData for correct hashing

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
