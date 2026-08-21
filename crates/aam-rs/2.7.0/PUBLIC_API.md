# Public API (Rust)

This document is a short map of the public Rust API exported by this repository.

## Crate Entry

- Crate: `aam-rs`
- Main exports are re-exported from `src/lib.rs`.

## Main Public Modules

- `aaml` (legacy and widely used API)
- `aam` (new pipeline-backed API)
- `aam_value`
- `builder`
- `error`
- `found_value`
- `pipeline`

Feature-gated modules:

- `aot` (`aot` feature)
- `ffi` (`ffi` feature)
- `python` (`python` feature)
- `jni` (`jni` feature)

## Core Types

- `AAML` (legacy parser API): parse/load/merge, lookups, schema/type validation.
- `AAM` (new API): parse/load, structured querying, formatter and pipeline utilities.
- `AAMBuilder`: fluent builder for generating `.aam` content.
- `FoundValue`: lookup result wrapper (`as_str`, `as_list`, `as_object`).
- `AamlError`: typed error enum used across parsing/validation/runtime paths.

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
