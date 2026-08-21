//! Produce a lifecycle-shaped audit chain (JSONL) with the Rust SDK, for the
//! cross-SDK round-trip: the Python and TypeScript SDKs must verify it.
//!
//! Usage: cargo run --example produce_chain -- OUT.jsonl

use aae::hashchain::append_event_value;
use serde_json::json;
use std::io::Write;

fn ts(micros: u32) -> String {
    // C-3: UTC, Z-suffixed. Fixed 6-digit fractional seconds, matching the
    // Python SDK's serialization so all verifiers see identical byte shape.
    format!("2026-07-10T02:00:00.{micros:06}Z")
}

fn main() {
    let out_path = std::env::args()
        .nth(1)
        .expect("usage: produce_chain OUT.jsonl");
    let proposal_id = ulid::Ulid::new().to_string();

    let phases: Vec<(&str, &str, serde_json::Value)> = vec![
        (
            "proposal_submitted",
            "agent",
            json!({"intent": "produce_roundtrip_chain", "step_count": 1,
                   "tools": ["noop"], "rationale": "cross-SDK round-trip: rust-produced chain"}),
        ),
        (
            "preview_generated",
            "host",
            json!({"aggregate_blast_radius": "read_only", "preview_unsupported": false, "step_count": 1}),
        ),
        (
            "policy_decision",
            "policy_engine",
            json!({"decision": "allow", "policy_version": "rust-producer@1",
                   "rules_evaluated": ["allow_all"], "reason": "", "strictness": "strict_literal"}),
        ),
        ("token_minted", "host", json!({"max_uses": 1})),
        (
            "session_started",
            "gateway",
            json!({"step_count": 1, "token_use": 1}),
        ),
        (
            "step_executed",
            "gateway",
            json!({"step_index": 0, "success": true,
                   "outputs": {"exit_code": 0, "stdout": "rust producer ok"}}),
        ),
        ("session_completed", "gateway", json!({"step_count": 1})),
    ];

    let mut file = std::fs::File::create(&out_path).expect("create output");
    let mut prev: Option<String> = None;
    for (i, (event_type, actor, payload)) in phases.into_iter().enumerate() {
        let event = json!({
            "aae_version": "0.3",
            "event_id": ulid::Ulid::new().to_string(),
            "event_type": event_type,
            "proposal_id": proposal_id,
            "tenant_id": "cross-sdk",
            "agent_id": "rust-producer",
            "actor": actor,
            "ts": ts(100_000 + (i as u32) * 1_000),
            "payload": payload,
            "signature": null,
        });
        let sealed = append_event_value(&event, prev.as_deref()).expect("seal");
        writeln!(
            file,
            "{}",
            serde_json::to_string(&sealed).expect("serialize")
        )
        .expect("write");
        prev = sealed["this_event_hash"].as_str().map(str::to_string);
    }
    println!("rust chain tip: {}", prev.expect("nonempty chain"));
}
