# aae-protocol (Rust)

Rust SDK for the [Accountable Agentic Execution (AAE)](https://github.com/r3moteBee/aae) protocol.

## Install

```toml
[dependencies]
aae-protocol = "0.1"
```

## What this provides

- Serde-derived types for every AAE schema (Proposal, Preview, PolicyDecision, CapabilityToken, AuditEvent, ToolRegistration)
- Hash chain helpers (canonicalization, chain construction, verification)
- Capability token mint/verify (JWT via `jsonwebtoken`)
- Async-friendly traits (with `async` feature) for policy adapters, audit sinks, approval delivery
- Compiles to WASM for use in component-model hosts (e.g., Iarsma)

## Features

- `default = ["jwt"]`
- `jwt` — JWT capability token support via `jsonwebtoken`
- `async` — `async-trait`-based async interfaces

## Quick example

```rust
use aae::{Proposal, Step, Context, BlastRadius};
use aae::hashchain::{append_event, verify_chain};

// Construct a proposal
let proposal = Proposal {
    aae_version: aae::PROTOCOL_VERSION.to_string(),
    proposal_id: ulid::Ulid::new().to_string(),
    agent_id: "ops-agent-v3".to_string(),
    tenant_id: "default".to_string(),
    intent: "restart_test_service".to_string(),
    context: Context {
        rationale: "QA asked".to_string(),
        triggered_by: None,
        derived_from: None,
        extra: Default::default(),
    },
    steps: vec![Step {
        tool: "ssh_exec".to_string(),
        args: serde_json::json!({"host": "test-01", "command": "systemctl restart test"}),
        expected: None,
        blast_radius: BlastRadius::SingleService,
    }],
    submitted_at: chrono::Utc::now(),
};

// Serialize, send to a host application implementing AAE...
```

## Status

v0.1 — early development. APIs may shift before v1.0.

## License

MIT OR Apache-2.0
