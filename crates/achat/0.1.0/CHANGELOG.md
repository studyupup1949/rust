# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-03-30

### Added
- Initial release
- Daemon architecture with Unix domain socket IPC
- mDNS peer discovery for cross-machine agents
- Local file registry for same-machine fast path
- Direct messaging (`@agent`), named groups, and broadcast
- JSON output by default with `--pretty` for humans
- `--hint` flag for agent-friendly next-action suggestions
- `help-json` command for machine-readable command reference
- Stdin pipe support for long messages
- Message persistence as JSONL files
- Identity resolution chain: `--as` > `ACHAT_NAME` env > `current` file > auto-detect
- `achat attach` for identity recovery after terminal restart

[Unreleased]: https://github.com/quadra-a/achat/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/quadra-a/achat/releases/tag/v0.1.0
