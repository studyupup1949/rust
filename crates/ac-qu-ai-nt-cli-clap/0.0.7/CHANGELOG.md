# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.7](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-cli-clap-v0.0.6...ac-qu-ai-nt-cli-clap-v0.0.7) - 2024-10-06

### Fixed

- move the Binstall configuration to the right spot

## [0.0.6](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-cli-clap-v0.0.5...ac-qu-ai-nt-cli-clap-v0.0.6) - 2024-10-06

### Other

- specify the correct download URLs for Binstall to work correctly

## [0.0.5](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-cli-clap-v0.0.4...ac-qu-ai-nt-cli-clap-v0.0.5) - 2024-10-05

### Other

- change `Hello, world!` to `What's up world?` to allow me to cut a release

## [0.0.4](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-cli-clap-v0.0.3...ac-qu-ai-nt-cli-clap-v0.0.4) - 2024-09-30

### Added

- make `tracing` a crate feature and make it a default feature of the current crates

### Other

- *(cli-clap)* or at least I hope chore is the right conventional commit type to use - switch `println!` to a `tracing` invocation (to verify that the tracing infrastructure works)

## [0.0.3](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-cli-clap-v0.0.2...ac-qu-ai-nt-cli-clap-v0.0.3) - 2024-09-30

### Other

- add `tracing` and `tracing-subscriber` as dependencies

## [0.0.2](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-cli-clap-v0.0.1...ac-qu-ai-nt-cli-clap-v0.0.2) - 2024-09-25

### Other

- release

## [0.0.1](https://github.com/babichjacob/ac-qu-ai-nt/releases/tag/ac-qu-ai-nt-cli-clap-v0.0.1) - 2024-09-24

### Other

- add licenses and other required metadata to the crates
- initialize a Cargo workspace with a `cli-clap` and a `multibinary` package to align with how I want the project to work (wherein features can be turned off to optimize the resulting binary, and there are different interfaces (GUI vs TUI vs CLI vs just an API vs just a web app or any permutation of these) to choose from depending on the occasion)
