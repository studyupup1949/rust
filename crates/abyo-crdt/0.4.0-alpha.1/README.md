# abyo-crdt

[![Crates.io](https://img.shields.io/crates/v/abyo-crdt.svg)](https://crates.io/crates/abyo-crdt)
[![Documentation](https://docs.rs/abyo-crdt/badge.svg)](https://docs.rs/abyo-crdt)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

> Pure Rust CRDT library — Eg-walker event log over Fugue-Maximal list, with Peritext rich text planned.

`abyo-crdt` is a Conflict-free Replicated Data Type library written from scratch
in pure Rust, designed for native Rust applications (Tauri, Bevy, embedded,
local-first SaaS backends) that don't want to round-trip through WebAssembly or
JavaScript.

## Status

**v0.4.0-alpha** — comprehensive feature set, production-grade
engineering practices, alpha API. Check the [CHANGELOG](CHANGELOG.md)
for the full v0.1 → v0.4 history.

| Component                       | Status |
|---------------------------------|--------|
| List CRDT (Fugue-Maximal)       | ✅ AVL OST, `O(log N)` for all ops |
| Map / Counter / Set CRDTs       | ✅ |
| Rich text (Peritext)            | ✅ valued + boolean marks, expand rules |
| Quill / Yjs Delta JSON interop  | ✅ `to_delta` / `from_delta` |
| Yjs `lib0` + StateVector        | ✅ byte-identical primitives |
| Yjs `Y.Update v1` snapshot encoder | ✅ minimal — covers single-client Y.Text bootstrap |
| Anchor-based cursors / selections | ✅ |
| Undo / redo (inverse-op API)    | ✅ |
| Tombstone GC + log compaction   | ✅ |
| Grapheme cluster handling       | ✅ UAX #29 via `unicode-segmentation` |
| Auto-generated replica IDs      | ✅ `new_replica_id()`, `Text::new_random()` |
| Lamport overflow safety         | ✅ `checked_add` |
| Persistence layer (`Storage` trait, file-backed) | ✅ append-only log + atomic snapshots |
| WASM bindings                   | ✅ `bindings/wasm/` |
| Python bindings                 | ✅ `bindings/py/` (PyO3 abi3) |
| stateright model checker        | ✅ exhaustive interleaving search |
| cargo-fuzz harness              | ✅ 4 targets, **3.3M+ runs in CI** |
| Comparison bench vs `yrs`       | ✅ 17× faster on append/5000 |
| Memory bench                    | ✅ |
| crates.io publish prep          | ✅ `cargo package` clean |

## Why?

The Rust CRDT landscape currently looks like this:

- **Yjs** — JavaScript only; using it from Rust means WASM round-trip.
- **Automerge** — Rust core, but the JS API is the priority. Performance trails Yjs.
- **Loro** — modern, fast, but Rust is a build target for its JS bindings, not a first-class API.
- **diamond-types** — Joseph Gentle's Eg-walker reference; explicitly a research prototype.

`abyo-crdt` aims to be the **Pure Rust first** library: idiomatic Rust APIs,
no JS interop assumptions, and built on the most recent algorithmic research
(Eg-walker 2024, Fugue 2023, Peritext 2022).

## Quick start

```rust
use abyo_crdt::List;

// Each replica has a unique 64-bit ID. Pick a random u64 at first launch.
let mut alice = List::<char>::new(1);
let mut bob = List::<char>::new(2);

// Alice types "Hi"
alice.insert(0, 'H');
alice.insert(1, 'i');

// Bob syncs from Alice
bob.merge(&alice);
assert_eq!(bob.to_vec(), vec!['H', 'i']);

// Bob types "!" while offline
bob.insert(2, '!');

// Alice and Bob meet again
alice.merge(&bob);
assert_eq!(alice.to_vec(), vec!['H', 'i', '!']);
assert_eq!(bob.to_vec(), vec!['H', 'i', '!']);
```

## Design

### Eg-walker style event log

All operations are recorded in a causal log (event graph). The "current
state" is a function of the log: given the same log, every replica computes
the same visible sequence. This makes serialization trivial (just persist
the log) and incremental sync efficient (send only the ops the peer hasn't
seen).

### Fugue-Maximal positioning

Inserts are positioned in a tree structure where each node has a `parent`
and a `side` (Left or Right). The
[Fugue paper (Weidner et al., 2023)](https://arxiv.org/abs/2305.00583)
proves this layout has the **non-interleaving property**: when two users
concurrently type "Hello" and "World" at the same position, the merged
result is `HelloWorld` or `WorldHello`, never `HWeolrllod`.

This is a meaningful upgrade over the YATA algorithm used by Yjs, which
admits some interleaving in pathological concurrent edits.

### What you get

- **Deterministic convergence** — verified by `proptest`. See
  `tests/convergence.rs`.
- **Causal idempotency** — applying the same op twice is a no-op.
- **Pure Rust, no `unsafe`** — `#![forbid(unsafe_code)]`.
- **Optional `serde`** — on by default; disable with `default-features = false`.

## Examples

```bash
cargo run --example two_replicas
cargo run --example concurrent_burst
```

## Benchmarks

```bash
cargo bench
```

Comparison data against `diamond-types` and `loro` is forthcoming.

## Property tests

```bash
cargo test --release
```

The convergence suite generates random op sequences across N replicas and
verifies all replicas reach the same state regardless of merge order.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

## References

- Gentle, *Collaborative Text Editing with Eg-walker* (2024) —
  [paper](https://arxiv.org/abs/2409.14252)
- Weidner et al., *The Art of the Fugue: Minimizing Interleaving in
  Collaborative Text Editing* (2023) —
  [paper](https://arxiv.org/abs/2305.00583)
- Litt, Lim, Kleppmann, van Hardenberg, *Peritext: A CRDT for Collaborative
  Rich Text Editing* (Ink & Switch, 2022) —
  [paper](https://www.inkandswitch.com/peritext/)
- Kleppmann, *Local-first software* (Ink & Switch, 2019) —
  [essay](https://www.inkandswitch.com/local-first/)
