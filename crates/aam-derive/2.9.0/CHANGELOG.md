# Changelog

All notable changes to aam-derive will be documented in this file.

## [2.9.0](https://github.com/INiNiDS/aam-rs/compare/2.8.1...2.9.0) - 2026-07-16

### Added

- *(aam-derive)* support `#[aam(default)]` and `#[aam(default = "expr")]` on FromAam fields

### Fixed

- *(aam-derive)* make #[aam(default)] parsing MSRV-safe (no let-chain match guards)

## [2.8.1] - 2026-07-09

### Added

- `#[derive(FromAam)]` proc-macro with `rename`, `default`, `skip` attributes
- `schema_to_struct!` proc-macro — generate struct + `FromAam` impl from inline `@schema`

