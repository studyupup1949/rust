# aap-protocol

**Agent Accountability Protocol — Rust SDK**

```toml
[dependencies]
aap-protocol = "0.1"
```

```rust
use aap_protocol::{KeyPair, Identity, Authorization, Level, Provenance, AuditChain, AuditResult};

let supervisor = KeyPair::generate();
let agent      = KeyPair::generate();

// Identity — signed by human supervisor
let identity = Identity::new(
    "aap://acme/worker/bot@1.0.0",
    vec!["write:files".into()],
    &agent, &supervisor, "did:key:z6MkSupervisor",
)?;

// Authorization — human approves at Level 3
let auth = Authorization::new(
    &identity.id, Level::Supervised,
    vec!["write:files".into()],
    false, &supervisor, "did:key:z6MkSupervisor",
)?;

// Physical World Rule — Level 4 on physical node is rejected
let result = Authorization::new(
    "aap://factory/robot/arm@1.0.0", Level::Autonomous,
    vec!["move:arm".into()], true, &supervisor, "did:key:z6Mk",
);
assert!(result.is_err()); // PhysicalWorldViolation

// Provenance + Audit
let prov = Provenance::new(&identity.id, "write:file", b"input", b"output", &auth.session_id, &agent)?;
let mut chain = AuditChain::new();
chain.append(&identity.id, "write:file", AuditResult::Success, &prov.artifact_id, &agent, auth.level, false)?;
let (valid, count, _) = chain.verify();
assert!(valid && count == 1);
```

**[AAP Spec](https://aap-protocol.dev) · [NRP](https://nrprotocol.dev) · [Halyn](https://halyn.dev)**

License: MIT
