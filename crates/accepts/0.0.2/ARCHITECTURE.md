# accepts architecture

This repository currently contains the minimal **core** of the `accepts` project:
three traits plus a handful of helper traits and blanket implementations. There
are no bundled acceptors, macros, or code generation in this crate.

## Goals

- Provide a tiny, stable surface for synchronous and asynchronous "acceptor"
  abstractions.
- Favor composability: acceptors take `&self` and return `()`, making them easy
  to share without mutability and signaling that each stage forwards or
  side-effects; richer control flow can be layered externally.
- Stay `no_std`-first; opt into heap-backed helpers with `alloc`. Std-dependent
  conveniences live outside this crate.
- Keep dependencies zero so the core can be embedded in other crates without
  pull-in.

## Feature flags

- `alloc` (default): unlocks heap-backed blanket impls (`Box`, `Rc`, `Arc`)
  and the `DynAsyncAccepts` trait. Disable with `default-features = false` for
  core-only `no_std` builds.

## Crate layout

- `lib.rs`: re-exports the public surface.
- `Accepts<Value>`: synchronous acceptor (`&self`, fire-and-forget).
- `AsyncAccepts<Value>`: asynchronous acceptor returning an owned future tied
  to `&self`.
- `DynAsyncAccepts<Value>`: boxed-future variant for trait objects (auto
  implemented when `alloc` is enabled).
- `implementations/core/*`: blanket impls that work in `no_std`:
  - `unit`: `()` accepts any value and does nothing.
  - `tuple`: tuples up to 12 elements fan out to multiple acceptors (clones the
    value for all but the last).
  - `option` / `result`: forward when `Some`/`Ok`, no-op otherwise.
  - `collections/{slice,array}`: fan out across slices/arrays, cloning for each
    element; last element gets the original to avoid an extra clone.
  - `pointers`: forwards through references and, when `alloc` is on, through
    `Box`, `Rc`, and `Arc`.
- `implementations/alloc/*`: heap-backed impls (`Vec`, `VecDeque`, pointer
  wrappers).

## Naming

- Any type that implements `Accepts`/`AsyncAccepts` is an *acceptor*.
- This crate defines only the traits. Official acceptor bundles live in the `acceptor` series, including the no_std core ([`acceptor`](https://crates.io/crates/acceptor)) and siblings like [`acceptor-alloc`](https://crates.io/crates/acceptor-alloc)/[`acceptor-std`](https://crates.io/crates/acceptor-std). See the `acceptor` crate for naming/etiquette details.
- Keep these bundles focused on `Accepts`/`AsyncAccepts` implementations; macros,
  test helpers, or other tooling live in separate crates.

## Data flow and cloning

Fan-out impls (`tuple`, `slice`, `array`, `Vec`) call acceptors in order,
cloning the incoming value for every element except the last, which receives the
original. Consumers should pass `Clone`/`Copy` values or use cheap clones when
chaining multiple acceptors.

## Testing

There is no dedicated test crate. Behavior is primarily enforced through
doc-tests on the traits; fan-out impls are intentionally straightforward to keep
the surface predictable.
