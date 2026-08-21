# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is a Rust library (`adsbx_json`) for parsing JSON from the ADS-B Exchange API. It provides type-safe deserialization of aircraft tracking data using serde. Currently supports v2 of the API only.

## Build & Test Commands

```bash
cargo build                           # Build the library
cargo test                            # Run unit and integration tests
cargo bench                           # Run benchmarks
cargo build --release --examples      # Build examples in release mode

# Run the fetch example (requires API key)
ADSBX_API_KEY=xxx cargo run --example fetch -- --url https://adsbexchange.com/api/aircraft/v2/all
```

## Architecture

- **`src/lib.rs`** - Crate root with `ParseError` type
- **`src/v2.rs`** - V2 API types including `Response`, `Aircraft`, and supporting enums/structs. Uses `#[serde(deny_unknown_fields)]` to catch API changes
- **`tests/integration_test.rs`** - Integration tests using specimen JSON files in `tests/`
- **`benches/parser.rs`** - Criterion benchmarks for parsing performance

## Key Design Decisions

- Uses strict serde parsing (`deny_unknown_fields`) to error on unexpected JSON fields. This helps detect ADS-B Exchange API changes early.
- Deprecated fields like `frame` and `reg` are captured but marked with `skip_serializing` (use `aircraft_type` and `registration` instead).
- The `aircraft` array can be null in the API response; this is handled by `empty_aircraft_vec` deserializer.
