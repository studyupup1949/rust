# adaptive-card-core

[![crates.io](https://img.shields.io/crates/v/adaptive-card-core.svg)](https://crates.io/crates/adaptive-card-core)
[![docs.rs](https://docs.rs/adaptive-card-core/badge.svg)](https://docs.rs/adaptive-card-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Pure-Rust library for validating, optimizing, and transforming Microsoft Adaptive Cards v1.6.

Deterministic, side-effect-free operations against the official v1.6 JSON Schema. No `tokio`, no network, no async — just `serde_json::Value` in, typed reports out.

## Features

- **Schema validation** — official Microsoft v1.6 JSON Schema, embedded at build time
- **Accessibility scoring** — 0–100 score with rule-by-rule fix hints
- **Host compatibility** — capability matrix for Teams, Outlook, WebChat, Webex, Viva Connections, Windows
- **Card transformation** — version downgrade (v1.6 → v1.5 → v1.4 → v1.3) and host adaptation
- **Optimization** — auto-fix accessibility, flatten redundant containers, modernize action shapes
- **Templating** — convert literal cards into `${expression}` templates with sample data
- **Data → Card** — auto-detect Table / FactSet / List / Chart from data shape
- **Knowledge base** — curated example library with keyword search

## Quick start

```toml
[dependencies]
adaptive-card-core = "0.1"
```

```rust
use adaptive_card_core::{validate_card, Host};

let card = serde_json::json!({
    "type": "AdaptiveCard",
    "version": "1.6",
    "speak": "Hello",
    "body": [
        { "type": "TextBlock", "text": "Hello, world", "wrap": true }
    ]
});

let report = validate_card(&card, Some(Host::Teams));
assert!(report.valid);
assert_eq!(report.accessibility.score, 100);
```

### Transforming for a different host

```rust
use adaptive_card_core::{transform_card, Host, TransformTarget};

let card = serde_json::json!({
    "type": "AdaptiveCard", "version": "1.6",
    "body": [],
    "actions": [{ "type": "Action.Execute", "title": "Save", "verb": "save" }]
});

let report = transform_card(card, &TransformTarget {
    host: Some(Host::Outlook),
    ..Default::default()
}).unwrap();

// Card has been downgraded: version 1.6 -> 1.4 and Action.Execute -> Action.Submit
assert_eq!(report.card["version"], "1.4");
assert_eq!(report.card["actions"][0]["type"], "Action.Submit");
```

### Optimizing for accessibility

```rust
use adaptive_card_core::{optimize_card, OptimizeOpts};

let card = serde_json::json!({
    "type": "AdaptiveCard", "version": "1.6",
    "body": [{ "type": "Image", "url": "https://example.com/x.png" }]
});

let report = optimize_card(card, &OptimizeOpts {
    accessibility: true,
    ..Default::default()
});

// Image now has placeholder altText and the card has a `speak` property.
```

## Companion MCP server

For LLM-driven workflows, use [`adaptive-card-mcp`](https://crates.io/crates/adaptive-card-mcp) — a binary that exposes every function in this crate as an MCP tool over stdio.

## License

MIT — see `LICENSE` at the workspace root.

The embedded Adaptive Cards v1.6 JSON Schema is sourced from
[`microsoft/AdaptiveCards`](https://github.com/microsoft/AdaptiveCards) under
its MIT license. See `data/LICENSE-MICROSOFT`.
