# AdMyChat Zed Extension

Privacy-first developer monetization & sponsorship network for **Zed IDE** (`zed.dev`).

## Overview
Zed IDE runs native Rust + Wasm plugins. This package provides the Rust/Wasm extension manifest and telemetry heartbeat adapter to allow Zed developers to earn revenue while coding.

## Features
- **Privacy-first attention heartbeats**: Sends periodic active signals (`surface="zed"`) to `https://admychat.com/api/v1/telemetry/tick`.
- **Status bar sponsorship rendering**: Displays non-intrusive developer sponsorships directly in Zed's status bar.
- **Zero code tracking**: Only timing & active attention metrics, zero code or contents transmitted.

## Building
```bash
cargo build --target wasm32-wasip1 --release
```
