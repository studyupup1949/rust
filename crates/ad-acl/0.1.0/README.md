# ad-acl

[![crates.io](https://img.shields.io/crates/v/ad-acl.svg)](https://crates.io/crates/ad-acl)
[![docs.rs](https://docs.rs/ad-acl/badge.svg)](https://docs.rs/ad-acl)

Active Directory ACL semantics, in pure Rust, with no FFI and no Windows dependency.

[`windows-sddl`](https://crates.io/crates/windows-sddl) turns an `nTSecurityDescriptor`
blob into ACEs. This crate answers the next question: **what can the trustee actually do
with it?**

An ACE carrying `WRITE_PROP` plus object GUID `5b47d60f-6090-40b2-9f37-2a4de88f3063` is not
"write property" — it is `AddKeyCredential`: Shadow Credentials, i.e. account takeover
without touching the password.

```rust
use ad_acl::{grants, ControlPrimitive};

let sd = windows_sddl::parse(&raw)?;
for g in grants(&sd) {
    if g.primitive == ControlPrimitive::DcsyncGetChangesAll {
        println!("{} can DCSync", g.trustee);
        println!("  impact: {}", g.primitive.impact());
        println!("  fix:    {}", g.primitive.mitigation());
    }
}
```

## What it gives you

* `classify(&Ace) -> Vec<ControlPrimitive>` — one ACE to the primitives it grants.
* `grants(&SecurityDescriptor) -> Vec<Grant>` — every trustee/primitive pair, owner included.
* `ControlPrimitive::cost()` — attacker cost, ready to use as a graph edge weight.
* `ControlPrimitive::impact()` / `mitigation()` — the report lines for each primitive.
* `catalog` — the fixed, forest-independent extended-right and attribute GUIDs.
* `SchemaMap` — runtime resolution for the GUIDs that are *not* fixed.

Recognised primitives include `GenericAll`, `WriteDacl`, `WriteOwner`,
`ForceChangePassword`, `AddMember`, `AddSelfToGroup`, `AddKeyCredential`, `WriteRbcd`,
`WriteSpn`, `WriteAltSecurityIdentities`, `WriteAllowedToDelegateTo`, `WriteGpLink`,
`ReadGmsaPassword`, `ReadLapsPassword`, the three replication rights, `ReanimateTombstones`,
`Enroll` and `CreateDmsa`.

## Forest-specific GUIDs

LAPS (`ms-Mcs-AdmPwd`, `msLAPS-Password`, `msLAPS-EncryptedPassword`), the gMSA blob
(`msDS-ManagedPassword`) and the dMSA class (`msDS-DelegatedManagedServiceAccount`) get
their `schemaIDGUID` generated when the schema is extended, so they differ per forest and
cannot be hard-coded. Read them from `CN=Schema,CN=Configuration,<root>` and pass them in:

```rust
let schema = ad_acl::SchemaMap::from_entries(pairs); // (lDAPDisplayName, schemaIDGUID)
let grants = ad_acl::grants_with(&sd, &schema);
```

Without the map those ACEs are simply not classified — the crate never guesses.

## Scope

Only *allow* ACEs are interpreted. Deny ACEs are skipped rather than subtracted, so the
output is an over-approximation of effective access. That is what attack-path tooling
wants; it is **not** an effective-permissions engine.

## License

MIT
