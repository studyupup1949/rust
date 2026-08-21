# acceptor-std architecture

`acceptor-std` is the std-powered bundle of acceptors built on [`accepts`](https://crates.io/crates/accepts). This document captures design and layout details that go beyond the README.

## Goals

- Provide std-dependent acceptors (e.g., `std::sync::mpsc` integration) while keeping the core no_std crate slim.
- Mirror the sync/async shapes used in `acceptor` for familiarity.
- Keep dependencies minimal; only `std` beyond `accepts`.

## Dependency policy

- Requires `std`.
- Depends on `accepts` (no extra features enabled).
- No feature flags are exposed.

## Naming (series guidelines)

- Core traits live in `accepts`.
- no_std bundle: [`acceptor`](https://crates.io/crates/acceptor).
- Sibling bundles: [`acceptor-alloc`](https://crates.io/crates/acceptor-alloc), `acceptor-std`, etc., as separate crates (not feature-gated).
- Official bundles use the `acceptor-*` prefix so it’s clear what we maintain.

## Version map

| acceptor-std | accepts |
| --- | --- |
| 0.0.1 | 0.0.2 |

## Layout (modules)

- `mpsc/`: `MpscSender` forwards via `std::sync::mpsc::Sender`.
- `print/`: `DisplayPrinter`, `DebugPrinter` for simple stdout output.
- `support/`: test helpers (`block_on`, `Recorder`).

All acceptors implement `accepts::{Accepts, AsyncAccepts}`.

## Testing / CI

- Unit tests live alongside modules; `cargo test` is the primary check.
- No doctests currently.
