# Changelog

All notable changes to aam-core will be documented in this file.

## [2.9.0](https://github.com/INiNiDS/aam-rs/compare/2.8.1...2.9.0) - 2026-07-16

### Added

- *(bindings)* expose AAM::update reload API across all bindings
- *(aam)* add AAM::update and update_from_text reload methods

### Fixed

- *(ci)* add C# SafeAamHandle helpers for aam_update/aam_reload and rustfmt
- re-export anyhow from aam-core/aam-rs so define_aam_loader! works without downstream dep

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

