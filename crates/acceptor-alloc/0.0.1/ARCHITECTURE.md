# acceptor-alloc architecture

`acceptor-alloc` is the alloc-powered bundle of acceptors built on [`accepts`](https://crates.io/crates/accepts). This document captures design and layout details that go beyond the README.

## Goals

- Provide acceptors that need heap-backed collections without pulling in `std`.
- Keep shapes consistent with the `acceptor` series (sync/async variants where appropriate).
- Minimize dependencies to `alloc` + `accepts`.

## Dependency policy

- Requires `alloc` but not `std`.
- Depends on `accepts` (no extra features enabled).
- No feature flags are exposed.

## Naming (series guidelines)

- Core traits live in `accepts`.
- no_std bundle: [`acceptor`](https://crates.io/crates/acceptor).
- Sibling bundles:  `acceptor-alloc`, [`acceptor-std`](https://crates.io/crates/acceptor-std), etc., as separate crates (not feature-gated).
- Official bundles use the `acceptor-*` prefix so it’s clear what we maintain.

## Version map

| acceptor-alloc | accepts |
| --- | --- |
| 0.0.1 | 0.0.2 |

## Layout (modules)

- `btree_kv_router/`: `BTreeKVRouter` routes `(Key, Value)` pairs via `BTreeMap`, with optional fallback.
- `support/`: test helpers (`block_on`, `Recorder`).

All acceptors implement `accepts::{Accepts, AsyncAccepts}`.

## Testing / CI

- Unit tests live alongside modules; `cargo test` is the primary check.
- No doctests currently.
