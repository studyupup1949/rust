# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.6](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-core-v0.0.5...ac-qu-ai-nt-core-v0.0.6) - 2024-10-18

### Fixed

- *(ac-qu-ai-nt-core)* raise the minimum version of the `reqwest` dependency to the first one that includes the `zstd` feature

### Other

- *(ac-qu-ai-nt-core)* add `reqwest` as a dependency because I expect to use it and I'd like to make sure everything works out

## [0.0.5](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-core-v0.0.4...ac-qu-ai-nt-core-v0.0.5) - 2024-10-17

### Fixed

- *(ac-qu-ai-nt-core)* raise the minimum acceptable version of `rten` to `0.13.1`

### Other

- *(ac-qu-ai-nt-core)* remove `surrealdb` as a dependency because the BUSL doesn't fit what I want this project to be. sorry. we'll see if I feel inclined to give it another chance
- *(ac-qu-ai-nt-core)* upgrade `surrealdb` to `2` and I'm not sure why it defaulted to an old version
- *(ac-qu-ai-nt-core)* add `ocrs`, `rten`, and `surrealdb` as dependencies because I expect to use them in the future and this will allow GitHub Actions to build cache for them or to report back to me that they're incompatible with the currently `supporteds`
- *(ac-qu-ai-nt-core)* [**breaking**] make `tracing` no longer a default feature

## [0.0.4](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-core-v0.0.3...ac-qu-ai-nt-core-v0.0.4) - 2024-10-15

### Other

- update `llama-cpp-2` and other packages
- *(ac-qu-ai-nt-core)* update `llama-cpp-2`
- *(core)* add `snafu` as a dependency

## [0.0.3](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-core-v0.0.2...ac-qu-ai-nt-core-v0.0.3) - 2024-10-07

### Other

- [**breaking**] add `llama-cpp-2` as a dependency (because it will get used in the future), possibly breaking portability (but we'll see what the GitHub Actions workflows say back)

## [0.0.2](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-core-v0.0.1...ac-qu-ai-nt-core-v0.0.2) - 2024-10-06

### Other

- release

## [0.0.1](https://github.com/babichjacob/ac-qu-ai-nt/releases/tag/ac-qu-ai-nt-core-v0.0.1) - 2024-10-06

### Added

- initialize `core`, `gui-eframe`, and `tui-ratatui` crates and use them in the `multibinary`

### Other

- add package `description`s so that they can actually be published
- release
- run cargo fmt

## [0.0.1](https://github.com/babichjacob/ac-qu-ai-nt/releases/tag/ac-qu-ai-nt-core-v0.0.1) - 2024-10-06

### Added

- initialize `core`, `gui-eframe`, and `tui-ratatui` crates and use them in the `multibinary`

### Other

- release
- run cargo fmt

## [0.0.1](https://github.com/babichjacob/ac-qu-ai-nt/releases/tag/ac-qu-ai-nt-core-v0.0.1) - 2024-10-06

### Added

- initialize `core`, `gui-eframe`, and `tui-ratatui` crates and use them in the `multibinary`

### Other

- run cargo fmt
