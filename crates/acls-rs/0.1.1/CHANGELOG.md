# Changelog

All notable changes to the `acls-rs` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.1.1] — 2026-07-08

### Added
- `PermissionSet::insert(perm: AtomicPermission) -> bool` — O(log n) in-place
  insertion; returns `true` if the permission was not already present
- `PermissionSet::remove(perm: &AtomicPermission) -> bool` — O(log n) in-place
  removal; returns `true` if the permission was present

## [0.1.0] — 2026-06-30

### Added
- Algebraic permission operations (union, intersection, difference)
- RBAC (Role-Based Access Control) with inheritance and cycle detection
- ABAC (Attribute-Based Access Control) with context evaluation
- Temporal permissions with validity windows
- Zero runtime dependencies
- Full serde support (optional `serde` feature)
- 47 unit tests
