//! AAP Quick Start — run with: cargo run --example quickstart
//! https://aap-protocol.dev

use aap_protocol::{AuditChain, AuditResult, Authorization, Identity, KeyPair, Level, Provenance};

fn main() -> aap_protocol::Result<()> {
    println!("AAP — Agent Accountability Protocol");
    println!("=====================================\n");

    // 1. Keypairs
    let supervisor = KeyPair::generate();
    let agent = KeyPair::generate();
    println!("1. Keypairs generated (Ed25519, OsRng)");

    // 2. Identity — signed by human supervisor
    let identity = Identity::new(
        "aap://acme/worker/deploy-bot@1.0.0",
        vec!["write:files".into(), "exec:deploy".into()],
        &agent,
        &supervisor,
        "did:key:z6MkSupervisor",
    )?;
    println!("2. Identity:  {}", identity.id);
    println!("   Scope:     {:?}", identity.scope);

    // 3. Authorization — human approves at Level 3
    let auth = Authorization::new(
        &identity.id,
        Level::Supervised,
        vec!["write:files".into()],
        false,
        &supervisor,
        "did:key:z6MkSupervisor",
    )?;
    println!("3. Auth:      level={} valid={}", auth.level_name, auth.is_valid());

    // 4. Physical World Rule — Level 4 on physical node is rejected
    let result = Authorization::new(
        "aap://factory/robot/arm@1.0.0",
        Level::Autonomous,
        vec!["move:arm".into()],
        true,
        &supervisor,
        "did:key:z6MkSupervisor",
    );
    println!("4. Phys rule: blocked={}", result.is_err());

    // 5. Provenance — what the agent produced
    let prov = Provenance::new(
        &identity.id,
        "write:file",
        b"deploy instruction",
        b"deployed successfully",
        &auth.session_id,
        &agent,
    )?;
    println!("5. Provenance: {}...", &prov.artifact_id[..8]);

    // 6. Audit chain — tamper-evident
    let mut chain = AuditChain::new();
    chain.append(
        &identity.id, "write:file",
        AuditResult::Success,
        &prov.artifact_id,
        &agent, auth.level, false,
    )?;
    let (valid, count, _) = chain.verify();
    println!("6. Audit:     {} entries, valid={}", count, valid);

    println!("\n✅ Every action identified, authorized, traced, audited.");
    Ok(())
}
