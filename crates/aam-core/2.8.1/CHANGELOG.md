# Changelog

All notable changes to aam-core will be documented in this file.

## [2.8.1] - 2026-07-09

### Added

- Workspace split: core types and parsing engine extracted from aam-rs
- `FromAam` trait for AAM string deserialization
- `define_aam_loader!` declarative macro
- `AamDeserializer` — serde Deserialize for `.aam` content
- Schema reconstructer module (infers `@schema` from AAM instances)
- `rust-only` feature flag for Rust-exclusive builds

### Changed

- `legacy` feature now gates the deprecated AAML API and commands module
- `reconstructer` feature flag for schema reconstruction

