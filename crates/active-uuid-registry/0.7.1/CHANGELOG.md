# Changelog

All notable changes to this project will be documented in this file.

## [0.7.1] - 2026-03-07

### Added
- **`get_random_uuid_with_base(base)`** — Returns a new UUID generated with the given base (v8-style, same as reservation). Does not register the UUID in the registry; useful for generating standalone UUIDs with a consistent base.

### Changed
- **Dependencies** — `uuid` updated from 1.21.0 to 1.22.0
- **Crate documentation** — Added module-level docs in `lib.rs` describing the library, two-level namespace→context registry, feature flags (default vs `concurrent-map`), and usage responsibility

---

## [0.7.0] - 2026-02-28

### Added
- **`list_ids`** — `list_ids(ns, ctx)` returns `Vec<Uuid>` of all raw UUIDs within a context
- **`NamespaceString`, `ContextString` types** - String wrappers for easier data identification

### Changed
- **Getter renames** — `get_pairs` → `get_context_entries`, `get_namespace_pairs` → `get_namespace_entries`, `get_all_pairs` → `get_all_namespace_entries`
- **Getter return types** — all three getter functions now return `Vec<(NamespaceString, ContextString, Uuid)>` (was `Vec<(String, Uuid)>`)
- **`list_namespaces` return type** — now returns `Vec<NamespaceString>` (was `Vec<String>`)
- **`list_contexts` return type** — now returns `Vec<ContextString>` (was `Vec<String>`)

### Breaking
- `get_pairs`, `get_namespace_pairs`, `get_all_pairs` removed; use the renamed equivalents above

---

## [0.6.1] - 2026-02-28

### Changed
- **`UuidPoolError` Derives** — `UuidPoolError` now derives `Clone`, `PartialEq`, `Eq`, and `Hash`, making it usable in collections and comparable contexts

---

## [0.6.0] - 2026-02-27

### Added
- **Namespace Clear** — `clear_namespace(ns)` removes a namespace and all its contexts entirely from the registry
- **Clear All Namespaces** — `clear_all_namespaces()` drops the entire registry (non-returning)
- **Clear All Contexts** — `clear_all_contexts(ns)` drops all contexts within a namespace while retaining the namespace entry (non-returning)

### Changed
- **`registry_uuid` Re-export** — simplified from `pub mod registry_uuid { pub use uuid::*; }` to `pub use uuid as registry_uuid` for a more idiomatic crate-level re-export

### Breaking
- `clear_all()` removed from public interface; use `clear_all_namespaces()` for equivalent global behavior, or `clear_all_contexts(ns)` for namespace-scoped clearing

---

## [0.5.0] - 2026-02-19

### Added
- **Namespace Pool Segmentation** — Registry pool refactored into a two-level `namespace → context → Set<Uuid>` structure (`HashMap<NamespaceKey, HashMap<ContextKey, HashSet<Uuid>>>` / `DashMap<NamespaceKey, DashMap<ContextKey, DashSet<Uuid>>>`)
- **Namespace Management** — `add_namespace()`, `remove_namespace()`, `replace_namespace()` for pre-creating, removing, and renaming namespaces
- **Namespace Queries** — `get_namespace_pairs()` to retrieve all context-UUID pairs within a namespace; `list_namespaces()` to list all registered namespaces
- **Namespace Drain Operations** — `drain_namespace()` and `drain_all_namespaces()` for atomic read-and-clear of a single namespace or all namespaces, returning `Vec<(namespace, context, uuid)>` triples

### Changed
- **Global UUID Pool Structure** — Pool is now two-level nested (`namespace → context → UUIDs`) in both single-threaded and concurrent-map feature variants
- **Public API Signatures** — All context-scoped functions now require `namespace: &str` as their first argument (`reserve_id`, `reserve_id_with_base`, `reserve_id_with`, `add_id`, `remove_id`, `try_remove_id`, `replace_id`, `get_pairs`, `list_contexts`, `clear_context`, `drain_context`, `drain_all_contexts`)
- **`drain_all_contexts(namespace)`** — Now scoped to a single namespace and returns `Vec<(String, String, Uuid)>` (namespace, context, uuid) triples instead of `Vec<(String, Uuid)>`
- **`clear_all_contexts()`** — Renamed to `clear_all()` in the public interface to reflect that it clears all namespaces, not just contexts
- **Empty Namespace Cleanup** — `remove()` now cleans up empty inner context maps and empty namespace entries after UUID removal

### Breaking
- All public interface functions that previously accepted only `context: &str` now require `namespace: &str` as first argument
- `clear_all_contexts()` renamed to `clear_all()` in public interface
- `drain_all_contexts()` return type changed from `Result<Vec<(String, Uuid)>, UuidPoolError>` to `Result<Vec<(String, String, Uuid)>, UuidPoolError>`
- `list_contexts()` now requires a `namespace: &str` argument

---

## [0.4.0] - 2026-02-12

### Added
- **Drain Operations** - `drain_context()` and `drain_all_contexts()` functions for atomic read-and-clear operations
  - Single-threaded version holds mutex lock across both read and clear
  - Concurrent version uses `DashMap::remove()` for atomic removal with data return
- **DashMap Empty Set Cleanup** - Concurrent mode now removes empty `DashSet` entries from `DashMap` for memory efficiency
- **Granular Error Types** - More specific error variants for better error handling
  - `FailedToAddUuidToPoolError` for insertion failures
  - `FailedToRemoveUuidFromPoolError` for removal failures
  - `FailedToReplaceUuidInPoolError` for replacement failures

### Changed
- **Dependencies Updated** - `uuid` 1.20.0, `rand` 0.10.0, `thiserror` 2.0.18
- **Atomic Operations** - Improved drain functions to eliminate race conditions between read and clear operations

### Fixed
- **Race Condition** - Fixed race condition in `drain_context` and `drain_all_contexts` with atomic implementations
- **Error Message Accuracy** - `replace_uuid_in_pool` now references correct UUID in error messages

---

## [0.3.0] - 2026-01-05

### Added
- **Get Operations** - `get()` function to retrieve all UUIDs for a specific context
- **Get All Operations** - `get_all()` function to retrieve all UUIDs across all contexts
- **Context Clearing** - `clear_context()` function for context-specific clearing

### Changed
- **Function Renaming** - Renamed `clear()` to `clear_all_contexts()` for clarity

### Breaking
- Feature renamed from `concurrent` to `concurrent-map`
- Function `clear()` renamed to `clear_all_contexts()`

---

## [0.2.0] - 2025-12-31

### Added
- **Try Remove** - `try_remove()` function returning `bool` instead of `Result` for simpler error handling
- **Replace Operation** - `replace()` function for atomic UUID replacement within a context

---

## [0.1.0] - 2025-12-29

### Added
- **Core UUID Registry** - Context-based UUID organization with global registry
- **Reserve Operations** - `reserve()` function to generate and register new UUIDs
  - `reserve_with_base()` for custom base parameter
  - `reserve_with()` for custom base and retry count
- **Basic CRUD Operations** - `add()`, `remove()`, and `clear()` functions
- **Thread-Safe Registry** - Global registry using `parking_lot::Mutex` with `HashMap<Arc<str>, HashSet<Uuid>>`
- **Concurrent Feature** - Optional `concurrent` feature with `DashMap`/`DashSet` for high-concurrency scenarios
- **Context Management** - Context-aware UUID tracking with `Arc<str>` keys for efficient string sharing
- **Collision Handling** - Automatic retry mechanism for UUID generation conflicts with configurable retry limits
