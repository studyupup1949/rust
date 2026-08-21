# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-12-09

### Added

- `i2c` module exposes the `Acs37800I2c` driver.
- Crate `prelude` simplifies bringing driver traits into scope.
- I²C addressing documentation ships with the new driver API.
- `Acs37800` and `Acs37800EepromExt` traits unify sync and async EEPROM access.
- Reusable mock device keeps sync and async EEPROM tests on one code path.
- `rp2350-i2c-async` example targets Embassy-based platforms.
- Project-specific spelling dictionary supports `cspell` runs.
- README badges plus min supported rust version guidance describe local toolchain expectations.
- GitHub Actions workflow runs checks/tests on every supported toolchain.
- `just` recipes replicate the full CI matrix locally.

### Changed

- The crate now requires consumers to enable at least one interface feature (`i2c` or `spi`).
- The I²C driver must now be accessed through the new module and trait APIs (breaking change).
- `Acs37800ReadError` is no longer generic over the bus error type; it now reports a simplified `Io` variant that works in both `std` and `no_std` builds.
- Examples and documentation now refer to `Acs37800I2c` directly.
- Development dependencies were reorganized to support async Embassy-based targets.

### Removed

- Legacy `.env`/`just` remote example execution flow.
- `defaults` module that was replaced by per-module constants.

## [0.1.1] - 2025-12-08

### Fixed

- Made visibility of EEPROM data structures public.

## [0.1.0] - 2025-12-08

### Added

- Read and basic interpretation of EEPROM values from I2C variants.

[unreleased]: https://github.com/sbruton/acs37800/compare/0.2.0...HEAD
[0.2.0]: https://github.com/sbruton/acs37800/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/sbruton/acs37800/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/sbruton/acs37800/releases/tag/0.1.0
