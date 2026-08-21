# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - Unreleased

### Added
- **Documentation:** Added a comprehensive Mermaid architecture diagram to `README.md` explaining the `!Send` to `Send` concurrency bridge.
- **Documentation:** Added an "Advanced Integration & Troubleshooting" section to `README.md` with best practices for middleware ordering.
- **Documentation:** Completely overhauled `src/lib.rs` module-level documentation with the "Ultimate Microservice" example.
- **Examples:** Created a robust `examples/advanced_microservice.rs` demonstrating how to seamlessly combine native `tower_http::trace::TraceLayer`, `tower_http::timeout::TimeoutLayer`, native rate limiting, and a custom mocked JWT authentication extractor.

## [0.1.1] - 2026-07-04

### Added
- Initial successful release to crates.io.
- **Tower Compatibility:** `TowerLayerCompat` wrapper for safely injecting `tower` and `tower-http` middleware into the Actix Web execution model.
- **Ergonomics:** `AutoJson`, `AutoQuery`, and `AutoPath` extractors that eliminate boilerplate `.into_inner()` calls.
- **Middleware:** Feature-gated `RateLimit`, `Cache`, `RequestId`, `Tracing`, and `Timeout` modules natively optimized for Actix Web.
- **Utilities:** Unified `ApiResponse` and `ApiError` serialization envelopes.
