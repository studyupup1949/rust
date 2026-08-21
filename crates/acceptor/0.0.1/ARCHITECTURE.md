# acceptor architecture

`acceptor` is the no_std acceptor bundle built on top of the [`accepts`](https://crates.io/crates/accepts) core
traits. This document captures design and layout details that go beyond the
README.

## Goals

- Provide thin, composable acceptors that implement `Accepts` / `AsyncAccepts`
  without adding heavy dependencies.
- Keep a clear split: no_std only; consumers opt into std support in their own
  crate if needed.
- Offer both sync and async variants when the structure is the same; split
  types when requirements diverge.

## Dependency policy

This crate is always `no_std` and exposes no feature flags. Heap-backed helpers
can live in [`acceptor-alloc`](https://crates.io/crates/acceptor-alloc); std-dependent
acceptors live in sibling crates such as [`acceptor-std`](https://crates.io/crates/acceptor-std).

## Naming (series guidelines)

- Core traits live in `accepts`.
- no_std bundle: `acceptor` (this crate).
- Dep-specific bundles: `acceptor-alloc`, `acceptor-std`, etc., as siblings (not feature-gated subcrates).
- Official bundles use the `acceptor-*` prefix so it’s clear what we maintain. If you publish `*-acceptor` reusing someone else’s crate name (e.g., `foo-acceptor`), please do it with the owner’s OK to avoid confusion.
- Keep these bundles focused on `Accepts`/`AsyncAccepts` implementations; macros, test helpers, or other tooling belong in separate crates.

## Version map

| acceptor | accepts |
| --- | --- |
| 0.0.1 | 0.0.2 |

## Layout (modules)

- `around/`: before/after hooks (`Around`, `AsyncAround`).
- `batch/`: buffered forwarding (`Batch`).
- `stateful_callback/`: stateful callbacks that own shared context (`StatefulCallback`, `AsyncStatefulCallback`).
- `deref_forwarder/`: delegate through deref targets (`DerefForwarder`).
- `filter/`: predicate-based forwarding (`Filter`, `AsyncFilter`).
- `branch/`: conditional routing (`Branch`, `AsyncBranch`).
- `inspect/`: tap/inspect before forwarding (`Inspect`, `AsyncInspect`).
- `iterator/`: iterate and forward items (`ForEach`, `AsyncForEach`).
- `map/`: map values before forwarding (`Map`, `AsyncMap`).
- `once/`: single-shot forwarding (`Once`, `AsyncOnce`).
- `repeat/`: repeat forwarding (`Repeat`, `AsyncRepeat`).
- `result_router/`: route `Result` to ok/err acceptors (`ResultRouter`, `AsyncResultRouter`).
- `router/`: multi-route dispatch (`Router`, `AsyncRouter`).

All acceptors are implemented in terms of `accepts::{Accepts, AsyncAccepts}` and
require no other crates. The crate is always `no_std`; add std-dependent
acceptors in a sibling crate if you need them.
