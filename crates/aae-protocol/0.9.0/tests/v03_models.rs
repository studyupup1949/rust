//! Model parity tests: serde round-trips of the v0.3+ additions that remain
//! core in v0.8, backward compatibility with v0.1-shaped JSON, and the v0.8
//! surface reduction (closed 3-value decision enum, open strictness string,
//! `artifact_attested`).

use aae::{AuditEvent, Decision, PolicyDecision, Proposal};

#[test]
fn v01_proposal_json_still_parses() {
    let json = r#"{
        "aae_version": "0.1",
        "proposal_id": "01J0AAE0000000000000000001",
        "agent_id": "ops-agent-v3",
        "tenant_id": "acme-prod",
        "intent": "restart_nginx",
        "context": {"rationale": "test"},
        "steps": [{
            "tool": "ssh_exec",
            "args": {"host": "web-01", "command": "true"},
            "blast_radius": "single_service"
        }],
        "submitted_at": "2026-05-10T14:32:11Z"
    }"#;
    let p: Proposal = serde_json::from_str(json).expect("v0.1 shape parses");
    assert!(p.agent_chain.is_none());
    // v0.1 shape round-trips without optional fields appearing
    let out = serde_json::to_value(&p).unwrap();
    assert!(out.get("agent_chain").is_none());
}

#[test]
fn proposal_round_trips_with_agent_chain_and_derived_from() {
    let json = r#"{
        "aae_version": "0.8",
        "proposal_id": "01J0AAE0000000000000000001",
        "agent_id": "worker",
        "agent_chain": ["orchestrator", "worker"],
        "tenant_id": "acme-prod",
        "intent": "restart_nginx",
        "context": {
            "rationale": "test",
            "derived_from": "01J0AAE0000000000000000000"
        },
        "steps": [{
            "tool": "ssh_exec",
            "args": {"host": "web-01", "command": "true"},
            "blast_radius": "single_service"
        }],
        "submitted_at": "2026-07-09T00:00:00Z"
    }"#;
    let p: Proposal = serde_json::from_str(json).expect("v0.8 shape parses");
    assert_eq!(
        p.agent_chain.as_deref(),
        Some(&["orchestrator".to_string(), "worker".to_string()][..])
    );
    assert_eq!(
        p.context.derived_from.as_deref(),
        Some("01J0AAE0000000000000000000")
    );
    let out = serde_json::to_value(&p).unwrap();
    assert_eq!(out["agent_chain"][1], "worker");
}

// ───────────────── v0.8: decision enum closed at three values ─────────────

#[test]
fn core_decisions_round_trip() {
    for (variant, wire) in [
        (Decision::Allow, "\"allow\""),
        (Decision::Deny, "\"deny\""),
        (Decision::RequireApproval, "\"require_approval\""),
    ] {
        assert_eq!(serde_json::to_string(&variant).unwrap(), wire);
        let back: Decision = serde_json::from_str(wire).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn retired_and_unknown_decision_values_fail_deserialization() {
    // Retired v0.3 refinements are now unknown values: MUST-reject.
    for wire in [
        "\"require_amendment\"",
        "\"require_legibility_adapter\"",
        "\"allow_with_conditions\"",
        "\"ALLOW\"",
    ] {
        assert!(
            serde_json::from_str::<Decision>(wire).is_err(),
            "decision value {wire} must fail deserialization"
        );
    }
}

#[test]
fn policy_decision_with_retired_decision_value_fails() {
    for retired in ["require_amendment", "require_legibility_adapter"] {
        let json = format!(
            r#"{{
                "aae_version": "0.8",
                "decision_id": "01J0AAE000000000000000000D",
                "proposal_id": "01J0AAE0000000000000000001",
                "decision": "{retired}",
                "policy_version": "p@1",
                "rules_evaluated": [],
                "reason": "no",
                "decided_at": "2026-07-12T12:00:00Z"
            }}"#
        );
        assert!(
            serde_json::from_str::<PolicyDecision>(&json).is_err(),
            "PolicyDecision with decision={retired} must fail deserialization"
        );
    }
}

// ───────────────── v0.8: strictness is an open string ─────────────────────

#[test]
fn strictness_defaults_and_accepts_core_and_extension_values() {
    let base = |strictness: &str| {
        format!(
            r#"{{
                "aae_version": "0.8",
                "decision_id": "01J0AAE000000000000000000D",
                "proposal_id": "01J0AAE0000000000000000001",
                "decision": "allow",
                "policy_version": "p@1",
                "rules_evaluated": ["r1"],
                "reason": "ok",
                "expires_at": "2026-07-12T12:05:00Z",
                "strictness": "{strictness}",
                "decided_at": "2026-07-12T12:00:00Z"
            }}"#
        )
    };
    // Core values.
    for mode in [
        aae::STRICTNESS_STRICT_LITERAL,
        aae::STRICTNESS_STRICT_TEMPLATE,
    ] {
        let d: PolicyDecision = serde_json::from_str(&base(mode)).unwrap();
        assert_eq!(d.strictness, mode);
    }
    // Extension/companion modes are schema-valid (open string). A host that
    // does not implement one MUST fail closed (C-16) — a lifecycle concern,
    // not a parse error.
    let d: PolicyDecision = serde_json::from_str(&base("acme.custom_mode")).unwrap();
    assert_eq!(d.strictness, "acme.custom_mode");

    // Absent strictness defaults to strict_literal.
    let legacy = r#"{
        "aae_version": "0.1",
        "decision_id": "01J0AAE000000000000000000D",
        "proposal_id": "01J0AAE0000000000000000001",
        "decision": "deny",
        "policy_version": "p@1",
        "rules_evaluated": [],
        "reason": "no",
        "decided_at": "2026-07-12T12:00:00Z"
    }"#;
    let d: PolicyDecision = serde_json::from_str(legacy).unwrap();
    assert_eq!(d.strictness, aae::STRICTNESS_STRICT_LITERAL);
}

// ───────────────── v0.8: artifact_attested event ──────────────────────────

#[test]
fn artifact_attested_event_round_trips() {
    let json = r#"{
        "aae_version": "0.8",
        "event_id": "01J0AAE00000000000000000E2",
        "event_type": "artifact_attested",
        "proposal_id": "01J0AAE0000000000000000001",
        "tenant_id": "acme-prod",
        "agent_id": "ops-agent-v3",
        "actor": "host",
        "ts": "2026-07-12T12:00:00Z",
        "payload": {
            "artifact_hash": "sha256:d2f45f194a4d1a1a44a4a45b6bfcbd54d84dcf9a2b1f8d3f5f194a4d1a1a44a4",
            "bound_event_id": "01J0AAE00000000000000000E1",
            "artifact_kind": "legible_record",
            "uri": "s3://audit-artifacts/records/r1.json",
            "media_type": "application/json"
        },
        "prev_event_hash": null,
        "this_event_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    }"#;
    let e: AuditEvent = serde_json::from_str(json).expect("artifact_attested parses");
    assert_eq!(e.event_type, aae::EVENT_TYPE_ARTIFACT_ATTESTED);
    // Payload minimum per the v0.8 payload contract.
    for required in ["artifact_hash", "bound_event_id", "artifact_kind"] {
        assert!(
            e.payload.get(required).is_some(),
            "payload must carry {required}"
        );
    }
    let out = serde_json::to_value(&e).unwrap();
    assert_eq!(out["event_type"], "artifact_attested");
    assert_eq!(out["payload"]["artifact_kind"], "legible_record");
    assert_eq!(
        out["payload"]["bound_event_id"],
        "01J0AAE00000000000000000E1"
    );
}

// ─────────────────── v0.7: signatures + confidence ───────────────────────

#[test]
fn confidence_and_ext_round_trip_on_decision() {
    let json = r#"{
        "aae_version": "0.7",
        "decision_id": "01J0AAE000000000000000000D",
        "proposal_id": "01J0AAE0000000000000000001",
        "decision": "allow",
        "policy_version": "p@1",
        "rules_evaluated": ["r1"],
        "reason": "ok",
        "expires_at": "2026-07-10T12:05:00Z",
        "decided_at": "2026-07-10T12:00:00Z",
        "ext": {"confidence": {"score": 0.72, "basis": "similar plans allowed"}}
    }"#;
    let d: aae::PolicyDecision = serde_json::from_str(json).expect("v0.7 decision parses");
    let conf: aae::Confidence =
        serde_json::from_value(d.ext.clone().unwrap()["confidence"].clone()).unwrap();
    assert!((conf.score - 0.72).abs() < f64::EPSILON);
    // a v0.1-shaped decision (no ext) still parses and omits it on output
    let legacy = r#"{
        "aae_version": "0.1",
        "decision_id": "01J0AAE000000000000000000D",
        "proposal_id": "01J0AAE0000000000000000001",
        "decision": "deny",
        "policy_version": "p@1",
        "rules_evaluated": [],
        "reason": "no",
        "decided_at": "2026-07-10T12:00:00Z"
    }"#;
    let d2: aae::PolicyDecision = serde_json::from_str(legacy).unwrap();
    assert!(d2.ext.is_none());
    let out = serde_json::to_value(&d2).unwrap();
    assert!(out.get("ext").is_none());
}

#[test]
fn signed_event_round_trips_and_position_is_bound() {
    use aae::hashchain::{append_event_value_signed, verify_chain_values};
    use aae::signing::{verify_event_signature, Ed25519EventSigner};

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let priv_pem =
        std::fs::read_to_string(root.join("reference-tests/fixtures/keys/test-signing-key.pem"))
            .unwrap();
    let pub_pem = std::fs::read_to_string(
        root.join("reference-tests/fixtures/keys/test-signing-key.pub.pem"),
    )
    .unwrap();
    let signer = Ed25519EventSigner::from_pem("aae-test-1", &priv_pem).unwrap();

    let event = serde_json::json!({
        "aae_version": "0.7",
        "event_id": "01J0AAE00000000000000000E1",
        "event_type": "proposal_submitted",
        "proposal_id": "01J0AAE0000000000000000001",
        "tenant_id": "t",
        "agent_id": "a",
        "actor": "host",
        "ts": "2026-07-10T12:00:00.000001Z",
        "payload": {},
        "signature": null,
    });
    let sealed = append_event_value_signed(&event, None, &signer).unwrap();
    verify_chain_values(std::slice::from_ref(&sealed)).expect("hash chain valid with signature");

    let mut keys = std::collections::HashMap::new();
    keys.insert("aae-test-1".to_string(), pub_pem);
    assert!(verify_event_signature(&sealed, &keys).unwrap());

    // re-parenting the signed event must fail: prev_event_hash is signed
    let mut moved = sealed.clone();
    moved["prev_event_hash"] = serde_json::json!("sha256:".to_string() + &"ab".repeat(32));
    assert!(!verify_event_signature(&moved, &keys).unwrap());
}
