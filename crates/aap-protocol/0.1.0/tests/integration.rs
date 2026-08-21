use aap_protocol::*;

fn supervisor() -> KeyPair { KeyPair::generate() }
fn agent() -> KeyPair { KeyPair::generate() }

fn identity(sup: &KeyPair, ag: &KeyPair) -> Identity {
    Identity::new(
        "aap://acme/worker/bot@1.0.0",
        vec!["read:files".into(), "write:*".into()],
        ag, sup, "did:key:z6MkTest",
    ).unwrap()
}

// ── Identity ─────────────────────────────────────────────────────────────────

#[test] fn test_identity_create() {
    let (sup, ag) = (supervisor(), agent());
    let ident = identity(&sup, &ag);
    assert_eq!(ident.id, "aap://acme/worker/bot@1.0.0");
    assert!(!ident.signature.is_empty());
}

#[test] fn test_identity_invalid_id() {
    let (sup, ag) = (supervisor(), agent());
    let r = Identity::new("not-valid", vec!["read:files".into()], &ag, &sup, "did:key:z");
    assert!(matches!(r, Err(AAPError::Validation { .. })));
}

#[test] fn test_identity_empty_scope() {
    let (sup, ag) = (supervisor(), agent());
    let r = Identity::new("aap://x/y/z@1.0.0", vec![], &ag, &sup, "did:key:z");
    assert!(matches!(r, Err(AAPError::Validation { .. })));
}

#[test] fn test_identity_allows_action() {
    let (sup, ag) = (supervisor(), agent());
    let ident = identity(&sup, &ag);
    assert!(ident.allows_action("read:files"));
    assert!(ident.allows_action("write:anything"));  // wildcard
    assert!(!ident.allows_action("delete:files"));
}

#[test] fn test_identity_verify_correct_key() {
    let (sup, ag) = (supervisor(), agent());
    let ident = identity(&sup, &ag);
    assert!(ident.verify(&sup.public_key_b64()).is_ok());
}

#[test] fn test_identity_verify_wrong_key() {
    let (sup, ag) = (supervisor(), agent());
    let ident = identity(&sup, &ag);
    let wrong = supervisor();
    assert!(ident.verify(&wrong.public_key_b64()).is_err());
}

#[test] fn test_identity_revoked_fails() {
    let (sup, ag) = (supervisor(), agent());
    let mut ident = identity(&sup, &ag);
    ident.revoked = true;
    assert!(matches!(ident.verify(&sup.public_key_b64()), Err(AAPError::Revocation { .. })));
}

// ── Authorization ─────────────────────────────────────────────────────────────

#[test] fn test_auth_valid() {
    let sup = supervisor();
    let auth = Authorization::new(
        "aap://x/y/z@1.0.0", Level::Supervised, vec!["write:files".into()],
        false, &sup, "did:key:z",
    ).unwrap();
    assert!(auth.is_valid());
}

#[test] fn test_physical_world_rule() {
    let sup = supervisor();
    let r = Authorization::new(
        "aap://factory/robot/arm@1.0.0", Level::Autonomous, vec!["move:arm".into()],
        true, &sup, "did:key:z",
    );
    assert!(matches!(r, Err(AAPError::PhysicalWorldViolation { .. })));
}

#[test] fn test_physical_supervised_allowed() {
    let sup = supervisor();
    let r = Authorization::new(
        "aap://factory/robot/arm@1.0.0", Level::Supervised, vec!["move:arm".into()],
        true, &sup, "did:key:z",
    );
    assert!(r.is_ok());
}

#[test] fn test_digital_autonomous_allowed() {
    let sup = supervisor();
    let r = Authorization::new(
        "aap://acme/worker/bot@1.0.0", Level::Autonomous, vec!["read:files".into()],
        false, &sup, "did:key:z",
    );
    assert!(r.is_ok());
}

#[test] fn test_auth_revoke() {
    let sup = supervisor();
    let mut auth = Authorization::new(
        "aap://x/y/z@1.0.0", Level::Observe, vec!["read:files".into()],
        false, &sup, "did:key:z",
    ).unwrap();
    auth.revoke();
    assert!(!auth.is_valid());
    assert!(auth.check().is_err());
}

// ── Provenance ────────────────────────────────────────────────────────────────

#[test] fn test_provenance_create_and_verify() {
    let ag = agent();
    let prov = Provenance::new(
        "aap://x/y/z@1.0.0", "write:file", b"input", b"output", "session-1", &ag,
    ).unwrap();
    assert!(!prov.artifact_id.is_empty());
    assert!(prov.verify(&ag.public_key_b64()).is_ok());
}

#[test] fn test_provenance_same_input_same_hash() {
    let ag = agent();
    let p1 = Provenance::new("aap://x/y/z@1.0.0", "read:file", b"same", b"same", "s1", &ag).unwrap();
    let p2 = Provenance::new("aap://x/y/z@1.0.0", "read:file", b"same", b"same", "s2", &ag).unwrap();
    assert_eq!(p1.input_hash, p2.input_hash);
}

// ── AuditChain ────────────────────────────────────────────────────────────────

#[test] fn test_empty_chain_valid() {
    let chain = AuditChain::new();
    let (valid, count, broken) = chain.verify();
    assert!(valid);
    assert_eq!(count, 0);
    assert!(broken.is_none());
}

#[test] fn test_append_and_verify() {
    let ag = agent();
    let mut chain = AuditChain::new();
    chain.append("aap://x/y/z@1.0.0", "write:file", AuditResult::Success, "prov-1", &ag, 3, false).unwrap();
    let (valid, count, _) = chain.verify();
    assert!(valid);
    assert_eq!(count, 1);
}

#[test] fn test_genesis_prev_hash() {
    let ag = agent();
    let mut chain = AuditChain::new();
    let entry = chain.append("aap://x/y/z@1.0.0", "read:file", AuditResult::Success, "prov-1", &ag, 0, false).unwrap();
    assert_eq!(entry.prev_hash, "genesis");
}

#[test] fn test_multiple_entries_valid() {
    let ag = agent();
    let mut chain = AuditChain::new();
    for i in 0..5 {
        chain.append("aap://x/y/z@1.0.0", "write:file", AuditResult::Success, &format!("prov-{i}"), &ag, 3, false).unwrap();
    }
    let (valid, count, _) = chain.verify();
    assert!(valid);
    assert_eq!(count, 5);
}

#[test] fn test_blocked_result_recorded() {
    let ag = agent();
    let mut chain = AuditChain::new();
    let entry = chain.append("aap://factory/robot/arm@1.0.0", "move:arm", AuditResult::Blocked, "prov-1", &ag, 3, true).unwrap();
    assert_eq!(entry.result, AuditResult::Blocked);
    assert!(entry.physical);
}

#[test] fn test_persistence_and_reload() {
    let ag = agent();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");
    {
        let mut chain = AuditChain::with_storage(&path).unwrap();
        chain.append("aap://x/y/z@1.0.0", "write:file", AuditResult::Success, "prov-1", &ag, 3, false).unwrap();
    }
    let chain2 = AuditChain::with_storage(&path).unwrap();
    assert_eq!(chain2.len(), 1);
    let (valid, _, _) = chain2.verify();
    assert!(valid);
}
