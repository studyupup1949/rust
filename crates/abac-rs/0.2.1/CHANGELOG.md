# Changelog

All notable changes to the `abac-rs` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.2.1] — 2026-08-07

### Added

- **`ExplainedDecision::new()` constructor** — downstream code that
  determines a decision via the cheaper `evaluate()` or `evaluate_at()`
  fast paths can now build an `ExplainedDecision` directly, without
  paying the cost of a full explained evaluation it doesn't need.

### Fixed

- **Global and dimension-sparse rules no longer silently ignored** —
  the composite index now treats a rule that doesn't mention a given
  dimension as unconstrained on that dimension (matching all values),
  consistent with how the evaluator itself works.  Previously, a global
  deny-all rule (zero dimensions) would stop being evaluated as soon as
  any other dimension-scoped rule existed in the same policy, causing
  `evaluate()` to incorrectly return Allow instead of Deny.

## [0.2.0] — 2026-07-27

### Added

- **Explained evaluation** — `evaluate_explained()` and
  `evaluate_explained_at()` return which rules matched, their types, and
  whether they are temporal, so callers can understand *why* a decision
  was made.
- **Rule IDs** — assign a stable, optional `id` field to each rule
  (distinct from the rule name) for correlation with external systems.
- **PolicyBuilder** for constructing policies with custom configuration
  (max rules, cache size) in a single expression.
- **Temporal rules** — `TemporalAbacRule` wraps a rule with
  `valid_from` / `valid_until` timestamps; `evaluate_at()` filters by
  wall-clock time.
- **Custom matchers** — plug in per-dimension predicate functions (e.g.
  threshold comparisons, CIDR matching) without requiring GIL access.
- **Fluent builder API** — `AbacRule::builder("name").dimension_values(…).enabled(true).build()`.
- **ABAC / RBAC composition** — combine ABAC decisions with RBAC
  policies in four modes (And, Or, AbacFirst, RbacFirst).
- **Configurable DoS protection** — `max_rules` limit prevents
  unbounded rule loading; returns `PolicyError::TooManyRules` on breach.
- Optional **serde** support for rules, requests, and decisions.
- Multi-layer optimization pipeline: compiled evaluator, bitmap deny
  index, composite indexing, LRU cache, AHash, Bloom filter.

### Changed

- `add_attribute()` now returns `Result<(), RequestError>` instead of
  silently accepting invalid input.
- Poisoned mutexes are recovered with `log::warn!` instead of panicking.

### Fixed

- Temporal override logic now correctly prioritises deny rules in
  time-bounded windows.
- Serde deserialization rejects excessively large payloads to prevent
  DoS.

## [0.1.0] — 2026-06-30

### Added

- Generic ABAC evaluation engine with arbitrary dimensions.
- Multi-type attribute system (String, Integer, Float, IpAddr, IpCidr).
- Pluggable matchers for custom predicate logic per dimension.
- Bloom filter pre-screening (optional `bloom` feature).
- Policy composition with RBAC in four modes.
- Optional serde support (`serde` feature).
- Built on `acls-rs` for algebraically correct permission composition.
