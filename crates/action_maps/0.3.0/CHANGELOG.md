# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Fixed

### Changed

### Removed

## 0.3.0

### Added
- `make_controls!`
- `make_multi_input!`

### Fixed

### Changed

### Removed

- `ControlScheme::with_controls`

## 0.2.1

### Changed

- Dev: `examples/` and `assets/` now excluded from publishing
- Bump Bevy to 0.12.1

## 0.2.0

### Added

- Dev: CI
- Added functionality to more easily handle multiplayer control schemes
    - MultiInput
    - MultiAction
    - MultiActionMapPlugin

### Fixed

- Typos in README

### Changed

- Breaking: Rename UniversalInputSet to ActionMapSet to better reflect it's purpose
- Made `name` field on Action public to allow greater flexibility
- Update README with planned changes section

### Removed

## 0.1.0

### Added

- Initial Version
