# Changelog

All notable changes to the `abac-rs` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.1.1] — 2026-07-08

### Added

- Rewrite README for crates.io
- Add dedicated Python binding crates
- Add type hints to all example files
- Make ComposedPolicy generic over CacheLock
- Add custom matcher support without with_gil
- Add JSON serialization support to Python bindings
- Add advanced Python bindings features
- Add essential Python bindings features
- Make builders fluent with method chaining
- Add Python usage examples for all crates
- Add maturin configuration for Python wheels
- Add Python bindings via PyO3
- Add PyO3 dependencies to core crates
- Add environmental_sensors_toml example
- Add sensor_config.toml configuration file
- Add environmental_sensors_yaml_file example
- Add sensor_config.yaml configuration file
- Add environmental_sensors_yaml example
- Add serde, serde_yaml, and toml dev dependencies
- Add environmental_sensors example and ABAC profiling harness
- Skip fast-path evaluators when custom matchers are registered
- Add AbacRuleBuilder fluent API
- Add bitmap-based AbacDenyIndex for fast deny matching
- Add configurable max_rules with DoS protection
- Skip cache when using compiled evaluator
- Optimize composite index candidate selection
- Optimize compiled evaluator with stack allocation
- Optimize request key group sorting
- Switch to AHash for 2-3× faster hashing
- Add pre-compiled evaluator for consistent dimensions
- Eliminate String clones by using rule indices
- Optimize composite index when universal allow exists
- Add universal allow fast-path
- Optimize composite index intersection
- Export new types and errors
- Add CacheLock trait for RefCell/Mutex choice
- Add RequestError and make add_attribute fallible
- Add ABAC/RBAC policy composition
- Add temporal rule support
- Add cache pipeline infrastructure
- Add thiserror and log dependencies
- Add generic ABAC evaluation engine

### Changed

- Remove in-tree python_bindings from core crates
- Remove python_bindings modules from main crates
- Split python_bindings into logical submodules
- Use SyncStrategy for ComposedPolicy
- Use shared abstractions from acls-rs
- Gate bloom filters and configure deps for WASM

### Fixed

- Add capsule API version validation and error logging
- Add SAFETY comments and module attributes
- Standardize workspace metadata inheritance
- Resolve clippy and formatting warnings across workspace
- Log mutex poison recovery in cache lock
- Restore error handling for backup rule loading failure
- Use AHashSet in tests and doc examples
- Update for add_attribute Result and private attributes

### Removed

- Remove redundant is_enabled checks and inline matcher

## [0.1.0] — 2026-06-30

### Added
- Generic ABAC evaluation engine with arbitrary dimensions
- Multi-type attribute system (String, Integer, Float, IpAddr, IpCidr)
- Pluggable matchers for custom predicate logic per dimension
- Multi-layer optimization pipeline:
  - Constant-result fast path (~15 ns)
  - Bitmap deny index (u64 bitmask intersection)
  - LRU memoization cache (sub-microsecond)
  - AHash (2-3x faster than SipHash)
  - Compiled evaluator (pre-extracted attributes + array indexing)
  - Composite indexing (O(log n))
  - Deny-only indexing (skip allow rules when universal allow exists)
- Bloom filter pre-screening (optional `bloom` feature)
- Policy composition with RBAC in four modes (And, Or, AbacFirst, RbacFirst)
- Optional serde support (`serde` feature)
- Built on `acls-rs` for algebraically correct permission composition
