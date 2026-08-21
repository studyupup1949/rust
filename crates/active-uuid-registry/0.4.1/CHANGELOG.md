# Changelog

All notable changes to this project will be documented in this file.

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
