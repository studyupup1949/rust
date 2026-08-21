# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-04-06

### Fixed

- Compilation with `hash` feature enabled

## [0.4.0] - 2026-04-06

### Changed

- `OsPatch::new()` and `OsVersion::new()` are now fallible

## [0.3.1] - 2026-01-26

### Changed

- Remove explicit `--cfg docsrs`

## [0.3.0] - 2026-01-26

### Changed

- Patch `generic-array` dependency to fix docs
- Update `binrw` dependency

## [0.2.3] - 2025-11-17

### Added

- add `HeaderV0::full_write`
- add `HeaderV0::compute_hash_digest`
- add `HeaderV0::boot_image_size`

## [0.2.2] - 2025-07-30

### Added

- Section layout diagram to `VendorHeader`'s documentation
- Functions to calculate section positions in `VendorHeader`
- `no_std` with `alloc` support
- `EitherHeader`

## [0.2.1] - 2025-07-28

### Fixed

- Add stricter Clippy lints
  * Added `const` and `#[must_use]` when recommended
- Truncate first part of `OsVersion` to 7 bits
  * This bug was caught by Clippy!

### Changed

- Add MSRV (minimum supported Rust version)

## [0.2.0] - 2024-12-18

### Added

- `Header::cmdline()`
- Warning about unreliable OS versioning

### Changed

- Combine `HeaderV0`'s cmdline fields

## [0.1.2] - 2024-12-18

### Added

- Re-export `binrw`'s `BufReader`
- Example
- Description to `unpack_bootimg`'s Cargo.toml
- Expose `VendorHeaderV4`

## [0.1.1] - 2024-09-06

### Fixed

- Handle unknown header version properly instead of panicking

## 0.1.0 - 2024-09-01

### Added

- Initial release

[unreleased]: https://github.com/axelkar/abootimg-oxide/compare/0.5.0...HEAD
[0.5.0]: https://github.com/axelkar/abootimg-oxide/compare/0.4.0...0.5.0
[0.4.0]: https://github.com/axelkar/abootimg-oxide/compare/0.3.1...0.4.0
[0.3.1]: https://github.com/axelkar/abootimg-oxide/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/axelkar/abootimg-oxide/compare/0.2.3...0.3.0
[0.2.3]: https://github.com/axelkar/abootimg-oxide/compare/0.2.2...0.2.3
[0.2.2]: https://github.com/axelkar/abootimg-oxide/compare/0.2.1...0.2.2
[0.2.1]: https://github.com/axelkar/abootimg-oxide/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/axelkar/abootimg-oxide/compare/0.1.2...0.2.0
[0.1.2]: https://github.com/axelkar/abootimg-oxide/compare/0.1.1...0.1.2
[0.1.1]: https://github.com/axelkar/abootimg-oxide/releases/tag/0.1.1
