# Changelog: abyssiniandb

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.1.2] (2023-01-31)
### Added
* `.github/workflows/test-ubuntu.yml`
* `.github/workflows/test-macos.yml`
* `.github/workflows/test-windows.yml`

### Changed
* test status badges into `README.tpl`

### Removed
* `.github/workflows/test.yml`

## [0.1.1] (2023-01-24)
### Added
* workspace: `xtool`
* `_xorshift64s()` for `myhasher`
* build status badges into `README.tpl`
* badges into `README.tpl`
* `.github/workflows/test.yml`
* `.github/workflows/test-miri.yml`
* `.github/workflows/test-outdated.yml`

### Changed
* move `src/check_main.rs` to `xtool/src/check_main.rs`
* rename feature `myhasher` to `std_default_hasher`
* minimum support rustc 1.58.1
* reformat `CHANGELOG.md`

### Fixed
* bypass test `test_size_of()` on windows
* clippy: this let-binding has unit value
* clippy: unnecessary\_cast, needless\_borrow
* clippy: bool\_assert\_comparison, explicit\_counter\_loop, useless\_conversion
* clippy: uninlined\_format\_args, seek\_from\_current

## [0.1.0] (2022-02-13)
* first commit

[Unreleased]: https://github.com/aki-akaguma/abyssiniandb/compare/v0.1.2..HEAD
[0.1.2]: https://github.com/aki-akaguma/abyssiniandb/releases/tag/v0.1.1..v0.1.2
[0.1.1]: https://github.com/aki-akaguma/abyssiniandb/releases/tag/v0.1.0..v0.1.1
[0.1.0]: https://github.com/aki-akaguma/abyssiniandb/releases/tag/v0.1.0
