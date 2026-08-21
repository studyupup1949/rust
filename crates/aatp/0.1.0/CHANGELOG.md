# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-07-11

### Added
- Frame codec — `[Magic][Version][Type][Length][Payload][CRC32C]` with a table-driven
  CRC32C integrity check; strict bounds, fails closed with typed errors.
- Zero-copy decode — `str`/`bytes` fields borrow from the input buffer; no allocation.
- `aatp_message!` declarative macro — generates an allocation-free codec per struct
  (lifetime and all-scalar arms).
- `codec` — safe typed `Writer`/`Reader` cursors.
- `message` — a ready `AgentMessage` payload (id, role, name, content, tokens).
- `state` — session state machine (`Handshake → Active → Closed`) rejecting
  out-of-order frames.
- `dict` — compile-time dictionary compression.
- `shadow` — reversible XOR keystream (obfuscation, **not** encryption).
- `#![no_std]`, `#![forbid(unsafe_code)]`, zero runtime and dev dependencies.
- 100% behavioural mutation coverage (0 survivors) and a deterministic fuzz/property
  harness (`tests/fuzz.rs`).
- CI: fmt · clippy · test matrix (ubuntu/windows/macos × stable + MSRV 1.85) · bare-metal
  `no_std` build (`thumbv7em-none-eabihf`).
- `AUDIT.md` and `.audit/` — reproducible audit evidence.

[Unreleased]: https://github.com/MerlijnW70/aatp/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/MerlijnW70/aatp/releases/tag/v0.1.0
