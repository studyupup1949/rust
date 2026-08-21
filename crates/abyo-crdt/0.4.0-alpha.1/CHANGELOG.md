# Changelog

All notable changes to `abyo-crdt` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — v0.4 part 3 (production-grade extension push)

#### Robustness

- Found and fixed an AVL deletion bug: the two-children detach path
  set successor's parent to `NIL` instead of the deleted node's
  original parent. Caught by a new `adversarial_inserts_then_remove_root_repeatedly`
  test.
- New OST adversarial unit tests: alternating-end inserts, zigzag
  inserts, set_visible thrashing.
- 100K-char append + 100K random-position insert stress tests
  (`#[ignore]`-flagged for CI release runs).
- Deletes-heavy workload test (5K inserts + 4K deletes).
- Mixed insert/delete/undo stress test with serde round-trip
  verification.
- Fuzz harness **actually run** for ~4 minutes in-session: 657K
  list_apply runs, 306K list_convergence runs, 195K text_delta runs,
  2.12M yjs_state_vector runs — **0 panics across 3.3M+ runs**.

#### Persistence

- New `storage` Cargo feature (default-on) gating
  `abyo_crdt::storage::{Storage, FileStorage, MemoryStorage,
  StorageError}`.
- Append-only log + atomic-rename snapshot, with `load_snapshot` /
  `load_ops_after_snapshot` for fast restart.
- 3 unit tests: round-trip, file-storage, snapshot-replaces-file.

#### Auto-replica-id + collision policy reversal

- `abyo_crdt::new_replica_id()` — OS-entropy-driven u64 generation
  via `getrandom`.
- `List::new_random()`, `Map::new_random()`, etc. — convenience
  constructors.
- **Removed** the strict `ReplicaIdConflict` rejection from `apply`
  paths: it broke the standard "load snapshot, replay log" pattern
  when our own replica's persisted ops re-arrived. Same-replica ops
  whose ids are NOT in our version vector are now accepted (treated
  as legitimate self-replay). The `Error::ReplicaIdConflict` variant
  is retained for documentation but no longer produced — `getrandom`
  makes id collisions vanishingly unlikely (10⁻¹⁹ over any practical
  population).

#### Convergence verification

- New `tests/stateright_model.rs`: explicit-state model checker
  using the [`stateright`](https://crates.io/crates/stateright) crate.
  Performs BFS over every interleaving of a small bounded operation
  set. 2-replica and 3-replica models both verified.

#### Yjs interop ↑

- New `yjs_compat::update::snapshot_text_to_yjs_update` and
  `snapshot_string_to_yjs_update`: emit a Y.Update v1 binary that
  `Y.Doc.applyUpdateV1` accepts, registering the content as a
  `Y.Text` at root key `"abyo"`. **Lossy** for marks; covers the
  "Rust server bootstraps a Yjs browser" handoff case.

#### FFI bindings

- New `bindings/wasm/` — `wasm-bindgen` based, exports `TextDoc`
  and `U32List` to JavaScript.
- New `bindings/py/` — `pyo3` (abi3-py38) based, builds via
  `maturin`. Exports `TextDoc`.
- Both binding crates are independent workspaces — they don't pull
  the heavy WebAssembly / Python tooling into the main `cargo build`.

#### Polish

- `CODE_OF_CONDUCT.md` adapted from Contributor Covenant 2.1.
- Allow-lints reduced to genuine necessities; module-level
  `#[allow(dead_code)]` lifted on `ost.rs` thanks to public
  exposure of new methods.

### Added — v0.4 (production-perfect push)

#### Performance — true `O(log N)` for all list ops

- **AVL order-statistic tree** (`src/ost.rs`, ~860 LOC) replaces the old
  imbl::Vector cache. All list operations are `O(log N)` including
  remote inserts, remote deletes, and `position_of(opid)` (the latter
  was `O(N)` linear scan; now uses an AVL parent-pointer walk-up).
- **Doc-order linked list** (`prev_doc`/`next_doc` per Item) provides
  `O(1)` shortcuts for the prepend/append common cases. Previously
  prepend was `O(N²)` total.
- **Bench vs `yrs`**: 17× faster on append/5000 chars (3.1ms vs 53ms).
  See `benches/vs_yrs.rs`.

#### Memory

- **Tombstone GC** (`List::gc(frontier)`): cascade-removes
  tombstoned items whose deletes are universally observed.
- **Log compaction** (`List::compact_log(frontier)`): drops covered
  entries from the event log.
- **Memory benchmark** (`benches/memory.rs`): peak heap measurements
  for typical workloads. 1000-char List = 742 KB, 10K-char = 5.9 MB.

#### Robustness

- **Replica ID collision detection**: every `apply()` now rejects ops
  claiming our own replica id with `Error::ReplicaIdConflict`. Was a
  silent corruption case before.
- **Lamport-clock overflow**: every clock increment uses `checked_add`
  and panics with a clear message instead of wrapping.
- **`Error::ClockOverflow`** variant for explicit overflow handling.
- **cargo-fuzz harness** (`fuzz/`) with 4 targets:
  `list_apply`, `list_convergence`, `text_delta`, `yjs_state_vector`.

#### Cursors / selections

- **`Cursor`**: anchor-based position handle that follows concurrent
  edits correctly. `Cursor::Start`, `Cursor::End`, or
  `Anchored { char_id, side }`. 16 bytes, `Copy`, serde-roundtrippable.
- **`Selection`**: range of two cursors.
- **`Cursor::resolve(&list)`** in `O(log N)`.

#### Grapheme handling

- `Text::grapheme_count` / `grapheme_to_char_pos` /
  `char_to_grapheme_pos` / `insert_grapheme_str` / `delete_grapheme`.
- New dep: `unicode-segmentation` (UAX #29).

#### Yjs interop (binary level)

- **`yjs_compat::lib0`**: byte-identical implementations of Yjs's
  variable-length encodings (`encodeVarUint` / `decodeVarUint`,
  `encodeVarInt` / `decodeVarInt`, `encodeVarString` /
  `decodeVarString`).
- **`yjs_compat::StateVector`**: `Y.encodeStateVector` /
  `Y.encodeStateVectorFromUpdate` byte format. Clients can negotiate
  what ops to exchange in the same wire format Yjs uses.
- Conversion to/from this crate's `VersionVector`.

#### Undo / redo

- **`List::apply_inverse(&op)`** and **`Text::apply_inverse(&op)`**:
  produce + apply inverse ops. Caller manages undo/redo stacks.
- Insert undo → tombstones the inserted item.
- Delete undo → re-inserts a fresh item anchored to the tombstone's
  left side, restoring its visible position.
- Mark `On` undo → emits `Off`. `Set(s)` undo → emits `Unset`.

#### Publishing

- `cargo package` clean (43 files, 82 KB compressed).
- `Cargo.toml` excludes plan, fuzz/, regressions, CI from the package.
- Two new CI jobs: `semver-checks` and `fuzz-build`.

### Added — v0.3-final (rich text completeness, perf, interop)

- **Valued annotations** in `Text`: marks like `href`, `color` now carry
  string values, not just on/off. New API: `Text::set_value_mark(range,
  name, Option<&str>)`. New types: `SpanValue::Set(String)` /
  `SpanValue::Unset`, `MarkValue::Boolean` / `MarkValue::Value(String)`.
  `MarkSet::value_of`, `MarkSet::iter_with_values`, `MarkSet::iter_values`.
- **Per-span expand rules**: new `ExpandRule` enum (`None` / `Right` /
  `Left` / `Both`) and `Text::set_mark_with_rule`. Lets callers choose
  whether typing at span boundaries inherits the mark.
- **`O(log N)` List operations**: replaced the `Vec<OpId>` cache with an
  incrementally-maintained `imbl::Vector<OpId>` index. Random-position
  inserts on a 1000-element document dropped from ~22 ms to ~150 µs
  (~140× speedup). Stress tests went from ~57 s to ~20 s.
- **Quill / Yjs Delta interop**: `Text::to_delta` / `Text::from_delta`
  convert to/from the Delta JSON format used by Yjs (`Y.Text`), Quill,
  Slate, and ProseMirror. New types: `DeltaOp`, `AttrValue`. Lossy:
  Delta is a snapshot, not the full op log.
- New example: `delta_interop`.
- 4 new Delta unit tests, 8 new valued/expand-rule unit tests
  (23 Text tests total).

### Added — v0.3-alpha (rich text)

- **`Text`**: Peritext-style rich text CRDT layered over `List<char>`.
  Supports format marks (`bold`, `italic`, `underline`, …) with
  proper concurrent-edit semantics:
  - Anchored start/end positions track correctly across concurrent
    inserts and deletes.
  - Concurrent set/set on overlapping ranges resolves via Lamport-ordered
    `OpId`.
  - Insertion in the middle of a span inherits the marks; insertion at
    span boundaries uses no-expand stickiness by default.
- **`Anchor`** / **`AnchorSide`** / **`Span`** / **`MarkSet`** /
  **`TextOp`**: anchor and format-span types for advanced usage.
- `Text::set_mark_with_anchors` — escape hatch for custom anchor
  stickiness (e.g. expand-right behavior).
- New example: `rich_text`.
- New tests: `tests/text_convergence.rs` (4-replica convergence +
  idempotency under randomized insert/delete/mark sequences),
  `tests/text_serde.rs` (JSON + bincode round-trip with marks).
- 15K randomized property cases pass under `PROPTEST_CASES=5000`.

### Added — v0.2 (sibling CRDTs)

- **`Map<K, V>`**: LWW-Map with Lamport-timestamped writes. Concurrent
  set/set and set/remove resolve via OpId total order.
- **`Counter`**: signed counter (PN-Counter style) with delta-based
  per-op replication. Exposes `value()`, `positive_total()`,
  `negative_total()`, `add()`, `increment()`, `decrement()`.
- **`Set<T>`**: Observed-Remove Set with **add-wins** semantics —
  concurrent `add(x)` and `remove(x)` resolve to "x is in the set".
- All three new types: full `serde` support, `ops()` event log,
  `ops_since(&version)` for incremental sync, version-vector idempotency.
- New examples: `map_lww`, `counter_likes`, `set_tags`.
- New property tests in `tests/v0_2_convergence.rs` (4-replica
  convergence + idempotency for each new type).
- New serde tests in `tests/v0_2_serde.rs` (JSON + bincode round trip).

### Performance

- `List` now caches its visible-id sequence between mutations. Read-after-
  write workloads (iter, get, len) drop from per-call O(N) tree walks to
  O(N) once + O(1) hits. Insert/delete still rebuild the cache, so pure
  write-heavy paths are unchanged. A true `O(log N)` B-tree index is
  tracked for v0.3 final.
- New `List::id_at`, `List::op_ids`, `List::phantom_positions`,
  `List::is_visible`, `List::contains_id`, `List::next_op_id`,
  `List::observe_external` — utilities for higher-level CRDTs that share
  the `List`'s Lamport clock (e.g. `Text`).

### Changed

- **Breaking**: `Op<T>` renamed to `ListOp<T>` to free up the `Op` namespace
  for sibling CRDTs (`MapOp<K, V>`, `CounterOp`, `SetOp<T>`, `TextOp`).
- **Breaking**: `List<char>::to_string` → `Display` impl. Same call site
  works (`format!("{list}")` or `list.to_string()`), now via the standard
  trait.
- `Text::to_string` is also via `Display`. Use `Text::as_string` if you
  specifically want a snapshot `String` without the `Display` machinery.
- `VersionVector` is now in its own module and shared across all CRDTs.
- `VersionVector::observe` and `VersionVector::get` are now public.

## [0.1.0-alpha.1] — 2026-05-07

Initial alpha release. Public API is **unstable** — we expect to break things
between alpha releases as the underlying tree representation evolves.

### Added

- `List<T>`: list CRDT with **Fugue-Maximal** positioning. Inserts and deletes
  operate by position; merges are commutative, associative, and idempotent.
- `OpId`: globally unique Lamport-style operation identifier.
- `Op<T>`: wire-format operation enum (`Insert` / `Delete`).
- `VersionVector`: summary of "ops this replica has seen", indexed by replica.
  Supports `ops_since(version)` for incremental sync.
- `Side`: parent-anchor side (`Left` or `Right`) used in the Fugue tree.
- `Error`: typed errors for out-of-bounds positions and missing causal
  prerequisites.
- Optional `serde` support (default-on): JSON, bincode, and any other serde
  format round-trip the full state including the event log.
- Examples: `two_replicas`, `concurrent_burst`, `persist_and_resume`.
- Property-based convergence tests (`tests/convergence.rs`): `proptest`
  generates random op sequences across 2/3/5 replicas and verifies all
  converge regardless of merge order. CI runs 10K cases per property.
- Stress tests (`tests/stress.rs`) validating up to 50K ops × 10 replicas.
- Benchmarks for append, prepend, and merge (`benches/`).

### Known limitations

- **`O(N)` per insert/delete**: each call recomputes the visible sequence by
  walking the full item tree. Building a 1000-char document takes ~22 ms in
  release mode. v0.2 will introduce a B-tree index to drop this to `O(log N)`.
- **No incremental garbage collection** of tombstones.
- **List CRDT only**. Map / Counter / Set are planned for v0.2; rich text
  (Peritext) for v0.3.

[Unreleased]: https://github.com/abyo-software/abyo-crdt/compare/v0.1.0-alpha.1...HEAD
[0.1.0-alpha.1]: https://github.com/abyo-software/abyo-crdt/releases/tag/v0.1.0-alpha.1
