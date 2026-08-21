# Public API (Rust)

This document is a short map of the public Rust API exported by this repository.

## Crate Entry

- Crate: `aam-rs` (facade)
- Real implementation lives in `aam-core`; proc macros live in `aam-derive`.
- Main exports are re-exported from `aam-rs/src/lib.rs`.

## Main Public Modules

- `aaml` (legacy API, gated by the `legacy` feature, enabled by default)
- `aam` (new pipeline-backed API)
- `aam_value`
- `builder`
- `error`
- `found_value`
- `from_aam` (serde-like deserialization from AAM strings)
- `pipeline`
- `macros` (`define_aam_loader!` and related helpers)

Feature-gated modules:

- `aot` (`aot` feature, enabled by default)
- `ffi` (`ffi` feature)
- `python` (`python` feature)
- `jni` (`jni` feature)
- `commands` (`legacy` feature)
- `types` (`legacy` feature)

## Core Types

- `AAML` (legacy parser API): parse/load/merge, lookups, schema/type validation (`legacy` feature).
- `AAM` (new API): parse/load, structured querying, formatter and pipeline utilities.
- `AAMBuilder`: fluent builder for generating `.aam` content.
- `FoundValue`: lookup result wrapper (`as_str`, `as_list`, `as_object`).
- `AamlError`: typed error enum used across parsing/validation/runtime paths.
- `FromAam`: derive macro for auto-generating AAM → struct deserialization (`#[derive(FromAam)]`).
- `from_aam::FromAam`: trait for manual AAM deserialization (`fn from_aam_str(value: &str) -> Result<Self, AamlError>`).
- `from_aam::get_aam` / `from_aam::get_opt_aam`: extract typed values from an `AAM` instance.
- `from_aam::parse_object_fields` / `from_aam::parse_list_items`: low-level helpers.
- `schema_to_struct!`: proc macro that generates a Rust struct + `FromAam` impl from an AAM `@schema` definition.

## Stable Lookup Model

- Forward lookup by key.
- Reverse lookup by value (for selected API surfaces).
- Deep lookup/reference resolution with loop-safe behavior.

## Directive and Type System Surface

- Directives: `@import`, `@derive`, `@schema`, `@type`.
- Built-in type families include primitives and domain types (`math`, `physics`, `time`, `list<T>`).

## Public API Change Policy

- Keep signatures and behavior backward-compatible unless explicitly marked as breaking.
- For any public API change, update this file and relevant binding `PUBLIC_API.md` files.
