# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.2] - 2025-01-06

### Fixed
- Fixed all Clippy warnings for better code quality
- Fixed uninlined format arguments throughout the codebase
- Fixed collapsible if statements for cleaner code
- Fixed redundant closures
- Replaced `map_or` with `is_some_and` for better readability
- Fixed module inception in tests.rs
- Changed `len() > 0` to `!is_empty()` for idiomatic Rust

### Changed
- Improved code formatting and style consistency
- Enhanced error messages with inline format strings

## [1.0.1] - 2025-01-06

### Added
- Prepared for crates.io publishing
- Added comprehensive README documentation

## [1.0.0] - 2025-01-06

### Added
- Initial release of abi2human
- Zero-dependency Ethereum ABI to human-readable converter
- Support for functions, events, constructors, receive, and fallback
- Multiple output formats: JSON array, raw text, compact JSON
- Batch processing for directories
- Stream processing via stdin/stdout
- Comprehensive test suite

### Features
- Convert single ABI files
- Convert entire directories with pattern matching
- Pipeline support for integration with other tools
- Multiple output format options
- Human-readable function and event signatures

[Unreleased]: https://github.com/yourusername/abi2human-rs/compare/v1.0.2...HEAD
[1.0.2]: https://github.com/yourusername/abi2human-rs/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/yourusername/abi2human-rs/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/yourusername/abi2human-rs/releases/tag/v1.0.0