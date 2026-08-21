# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.14](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.13...ac-qu-ai-nt-v0.0.14) - 2024-10-16

### Added

- *(multibinary)* support running the binary without arguments (using a sensible user interface as the default), and improve error reporting by using `snafu`

### Other

- *(multibinary)* mark the Clippy lint against manually implementing `Default` for `Command` as allowed
- *(multibinary)* add `cfg-if` and `snafu` as dependencies

## [0.0.13](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.12...ac-qu-ai-nt-v0.0.13) - 2024-10-15

### Other

- build out the infrastructure for commands to be passed to the clap CLI

## [0.0.12](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.11...ac-qu-ai-nt-v0.0.12) - 2024-10-07

### Other

- updated the following local packages: ac-qu-ai-nt-cli-clap, ac-qu-ai-nt-gui-eframe, ac-qu-ai-nt-tui-ratatui

## [0.0.11](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.10...ac-qu-ai-nt-v0.0.11) - 2024-10-06

### Other

- add Binstall configuration for the multibinary

## [0.0.10](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.9...ac-qu-ai-nt-v0.0.10) - 2024-10-06

### Other

- updated the following local packages: ac-qu-ai-nt-cli-clap, ac-qu-ai-nt-gui-eframe, ac-qu-ai-nt-tui-ratatui

## [0.0.9](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.8...ac-qu-ai-nt-v0.0.9) - 2024-10-06

### Other

- updated the following local packages: ac-qu-ai-nt-gui-eframe, ac-qu-ai-nt-tui-ratatui

## [0.0.8](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.7...ac-qu-ai-nt-v0.0.8) - 2024-10-06

### Added

- initialize `core`, `gui-eframe`, and `tui-ratatui` crates and use them in the `multibinary`

## [0.0.7](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.6...ac-qu-ai-nt-v0.0.7) - 2024-10-05

### Other

- updated the following local packages: ac-qu-ai-nt-cli-clap

## [0.0.6](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.5...ac-qu-ai-nt-v0.0.6) - 2024-10-05

### Fixed

- cover the new cases of `Command` for `GuiEframe` and `TuiRatatui`

### Other

- correctly add `gui-eframe` and `tui-ratatui` as features this time around
- add `gui-eframe` and `tui-ratatui` as subcommands with aliases `gui` and `tui` respectively
- add `gui-eframe` and `tui-ratatui` as features

## [0.0.5](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.4...ac-qu-ai-nt-v0.0.5) - 2024-10-05

### Other

- [**breaking**] put accessing the cli under a subcommand like the README suggests it would be

## [0.0.4](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.3...ac-qu-ai-nt-v0.0.4) - 2024-09-30

### Added

- *(multibinary)* initialize a basic tracing subscriber until this is changed to a directory one using `tracing-appender` at a later time
- make `tracing` a crate feature and make it a default feature of the current crates

## [0.0.3](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.2...ac-qu-ai-nt-v0.0.3) - 2024-09-30

### Added

- [**breaking**] this is the correct way to format a breaking change unlike my previous commit - ensure the application data directory, and the tracing directory within it, exists

### Other

- add `tracing` and `tracing-subscriber` as dependencies
- reserve a data directory for the application

## [0.0.2](https://github.com/babichjacob/ac-qu-ai-nt/compare/ac-qu-ai-nt-v0.0.1...ac-qu-ai-nt-v0.0.2) - 2024-09-25

### Other

- release

## [0.0.1](https://github.com/babichjacob/ac-qu-ai-nt/releases/tag/ac-qu-ai-nt-v0.0.1) - 2024-09-24

### Other

- add licenses and other required metadata to the crates
- initialize a Cargo workspace with a `cli-clap` and a `multibinary` package to align with how I want the project to work (wherein features can be turned off to optimize the resulting binary, and there are different interfaces (GUI vs TUI vs CLI vs just an API vs just a web app or any permutation of these) to choose from depending on the occasion)
