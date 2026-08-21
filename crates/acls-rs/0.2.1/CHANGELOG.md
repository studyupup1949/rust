# Changelog

All notable changes to the `acls-rs` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.2.1] — 2026-08-07

_No changes to `acls-rs` in this release. Version bumped to stay in sync
with the workspace._

## [0.2.0] — 2026-07-27

### Added

- **PermissionMapping** for configurable bitmask-to-permission translation —
  bridge domain-specific access masks (Windows file, Active Directory,
  registry, POSIX) to typed `PermissionSet` values and back.
- `PermissionSet::insert()` and `PermissionSet::remove()` for O(log n)
  in-place mutation of permission sets.
- `SyncStrategy<T>` trait for generic interior mutability — eliminates
  duplicate lock implementations across HBAC and ABAC crates.
- `PolicyError` shared error type for DoS-protection violations
  (`TooManyRules`), with `#[non_exhaustive]` for future expansion.
- `RuleLimitedPolicy` trait for enforcing maximum rule counts across all
  policy types.
- Python dunder methods (`__contains__`, `__len__`, `__bool__`, `__iter__`,
  `__or__`, `__and__`, `__sub__`, `__eq__`, `__hash__`) on permission types
  for idiomatic Python usage.

### Changed

- Poisoned mutexes are now recovered with `log::warn!` instead of panicking.

### Fixed

- Corrected code examples in README.

## [0.1.0] — 2026-06-30

### Added

- Algebraic permission operations (union, intersection, difference).
- RBAC (Role-Based Access Control) with inheritance and cycle detection.
- ABAC (Attribute-Based Access Control) with context evaluation.
- Temporal permissions with validity windows.
- Zero runtime dependencies.
- Full serde support (optional `serde` feature).
