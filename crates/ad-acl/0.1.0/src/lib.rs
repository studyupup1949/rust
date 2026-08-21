//! Active Directory ACL semantics.
//!
//! [`windows-sddl`](https://docs.rs/windows-sddl) turns a `nTSecurityDescriptor` blob into
//! ACEs. This crate answers the next question: *what can the trustee actually do with it?*
//! An ACE carrying `WRITE_PROP` plus object GUID `5b47d60f-…` is not "write property" — it
//! is [`ControlPrimitive::AddKeyCredential`], i.e. Shadow Credentials, i.e. account takeover
//! without touching the password.
//!
//! ```no_run
//! use ad_acl::{grants, ControlPrimitive};
//! # let raw_nt_security_descriptor: Vec<u8> = vec![];
//!
//! let sd = windows_sddl::parse(&raw_nt_security_descriptor).unwrap();
//! for g in grants(&sd) {
//!     if g.primitive == ControlPrimitive::DcsyncGetChangesAll {
//!         println!("{} can DCSync — {}", g.trustee, g.primitive.mitigation());
//!     }
//! }
//! ```
//!
//! Forest-specific attributes (LAPS, gMSA, dMSA) have per-forest `schemaIDGUID`s and are
//! resolved at runtime — see [`SchemaMap`] and [`grants_with`].
//!
//! Only *allow* ACEs are interpreted; deny ACEs are skipped rather than subtracted, so the
//! output is an over-approximation of effective access. That matches how attack-path tools
//! reason (a deny ACE that is ordered after an allow does not remove the primitive), but it
//! is not an effective-permissions engine.

pub mod catalog;
mod schema;

pub use schema::{names, SchemaMap};

use windows_sddl::sid::{Guid, Sid};
use windows_sddl::{AccessMask, Ace, SecurityDescriptor};

/// A concrete thing a trustee can do to an object, derived from one ACE.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ControlPrimitive {
    /// Owner of the object — can rewrite its DACL at will.
    Owns,
    /// `WRITE_DAC` — can grant itself anything else.
    WriteDacl,
    /// `WRITE_OWNER` — can take ownership, then rewrite the DACL.
    WriteOwner,
    /// `GENERIC_ALL` / full control.
    GenericAll,
    /// `GENERIC_WRITE`, or unscoped `WRITE_PROP` — can write every attribute.
    GenericWrite,
    /// Unscoped `CONTROL_ACCESS` — holds every extended right, DCSync included.
    AllExtendedRights,
    /// `User-Force-Change-Password`.
    ForceChangePassword,
    /// Write `member` — add any principal to the group.
    AddMember,
    /// Validated write on `member` — add *itself* to the group.
    AddSelfToGroup,
    /// Write `msDS-KeyCredentialLink` — Shadow Credentials.
    AddKeyCredential,
    /// Write `msDS-AllowedToActOnBehalfOfOtherIdentity` — resource-based constrained delegation.
    WriteRbcd,
    /// Write `servicePrincipalName` — make the account Kerberoastable (targeted roasting).
    WriteSpn,
    /// Write `altSecurityIdentities` — bind an attacker certificate to the account.
    WriteAltSecurityIdentities,
    /// Write `msDS-AllowedToDelegateTo` — constrained delegation with protocol transition.
    WriteAllowedToDelegateTo,
    /// Write `gPLink` — attach a hostile GPO to the container.
    WriteGpLink,
    /// Read the gMSA managed-password blob — derive the account's keys.
    ReadGmsaPassword,
    /// Read a LAPS password attribute — local administrator on that machine.
    ReadLapsPassword,
    /// `DS-Replication-Get-Changes`.
    DcsyncGetChanges,
    /// `DS-Replication-Get-Changes-All` — the half that carries secrets.
    DcsyncGetChangesAll,
    /// `DS-Replication-Get-Changes-In-Filtered-Set`.
    DcsyncGetChangesFiltered,
    /// `Reanimate-Tombstones` — resurrect deleted objects.
    ReanimateTombstones,
    /// `Certificate-Enrollment` / `Certificate-AutoEnrollment` on a template.
    Enroll,
    /// Create a delegated MSA under this container (BadSuccessor).
    CreateDmsa,
    /// `CREATE_CHILD`, optionally scoped to one object class GUID.
    CreateChild(Option<Guid>),
}

impl ControlPrimitive {
    /// Stable identifier, usable as a graph edge label.
    pub fn name(self) -> &'static str {
        use ControlPrimitive::*;
        match self {
            Owns => "Owns",
            WriteDacl => "WriteDacl",
            WriteOwner => "WriteOwner",
            GenericAll => "GenericAll",
            GenericWrite => "GenericWrite",
            AllExtendedRights => "AllExtendedRights",
            ForceChangePassword => "ForceChangePassword",
            AddMember => "AddMember",
            AddSelfToGroup => "AddSelfToGroup",
            AddKeyCredential => "AddKeyCredential",
            WriteRbcd => "WriteRbcd",
            WriteSpn => "WriteSpn",
            WriteAltSecurityIdentities => "WriteAltSecurityIdentities",
            WriteAllowedToDelegateTo => "WriteAllowedToDelegateTo",
            WriteGpLink => "WriteGpLink",
            ReadGmsaPassword => "ReadGmsaPassword",
            ReadLapsPassword => "ReadLapsPassword",
            DcsyncGetChanges => "DcsyncGetChanges",
            DcsyncGetChangesAll => "DcsyncGetChangesAll",
            DcsyncGetChangesFiltered => "DcsyncGetChangesFiltered",
            ReanimateTombstones => "ReanimateTombstones",
            Enroll => "Enroll",
            CreateDmsa => "CreateDmsa",
            CreateChild(_) => "CreateChild",
        }
    }

    /// Attacker cost of traversing this primitive. Lower = cheaper = more dangerous.
    ///
    /// `0` — already equivalent to control (no action needed).
    /// `1` — one write/read and the target is owned.
    /// `2` — needs a second step (a coerced auth, a TGT request, a roast).
    /// `3` — noisy or slow (offline cracking, waiting for a GPO refresh).
    pub fn cost(self) -> u32 {
        use ControlPrimitive::*;
        match self {
            AllExtendedRights | DcsyncGetChangesAll => 0,
            DcsyncGetChanges | DcsyncGetChangesFiltered => 1,
            Owns | WriteDacl | WriteOwner | GenericAll => 1,
            ForceChangePassword | AddMember | AddSelfToGroup | AddKeyCredential => 1,
            ReadGmsaPassword | ReadLapsPassword => 1,
            GenericWrite | WriteAltSecurityIdentities => 2,
            WriteRbcd | WriteAllowedToDelegateTo => 2,
            CreateDmsa => 2,
            Enroll | CreateChild(_) | ReanimateTombstones => 3,
            WriteSpn | WriteGpLink => 3,
        }
    }

    /// What the attacker gets out of it — the `impact` line of a report.
    pub fn impact(self) -> &'static str {
        use ControlPrimitive::*;
        match self {
            Owns => "owner can rewrite the DACL and grant itself full control",
            WriteDacl => "can grant itself full control over the object",
            WriteOwner => "can take ownership, then rewrite the DACL",
            GenericAll => "full control over the object",
            GenericWrite => {
                "can write every attribute, including the delegation and credential ones"
            }
            AllExtendedRights => "holds every extended right on the object, DCSync included",
            ForceChangePassword => "can reset the password without knowing the current one",
            AddMember => "can add any principal to the group, inheriting its privilege",
            AddSelfToGroup => "can add itself to the group, inheriting its privilege",
            AddKeyCredential => "Shadow Credentials: PKINIT as the target, then its NT hash",
            WriteRbcd => "RBCD: S4U2Self+S4U2Proxy to impersonate any user to the target",
            WriteSpn => "targeted Kerberoast: set an SPN, request a TGS, crack it offline",
            WriteAltSecurityIdentities => "binds an attacker certificate to the account for PKINIT",
            WriteAllowedToDelegateTo => {
                "constrained delegation with protocol transition to any service"
            }
            WriteGpLink => "attaches a hostile GPO to every computer under the container",
            ReadGmsaPassword => "reads the managed-password blob and derives the account's keys",
            ReadLapsPassword => "local administrator on that machine, no cracking needed",
            DcsyncGetChanges => {
                "half of DCSync; combined with Get-Changes-All it replicates secrets"
            }
            DcsyncGetChangesAll => "replicates every secret in the domain, krbtgt included",
            DcsyncGetChangesFiltered => "replicates the RODC-filtered attribute set",
            ReanimateTombstones => "resurrects deleted objects, reviving stale privilege",
            Enroll => "requests a certificate from the template; abusable if the template is weak",
            CreateDmsa => "BadSuccessor: a delegated MSA that inherits a privileged account's keys",
            CreateChild(_) => "creates child objects under the container",
        }
    }

    /// The defensive counterpart — the `defence` line of a report.
    pub fn mitigation(self) -> &'static str {
        use ControlPrimitive::*;
        match self {
            Owns | WriteOwner => "reset the owner to Domain Admins / the object's OU owner and audit ownership changes",
            WriteDacl => "remove the WRITE_DAC ACE; DACL writes on Tier-0 objects belong to Domain Admins only",
            GenericAll | GenericWrite => "replace full control with the narrowest right the delegation actually needs",
            AllExtendedRights => "remove the unscoped CONTROL_ACCESS ACE; grant individual extended rights instead",
            ForceChangePassword => "restrict password resets to the helpdesk OU; never on Tier-0 accounts",
            AddMember | AddSelfToGroup => "manage membership through a PAM/AGDLP group, not a write ACE on the group",
            AddKeyCredential => "remove write access to msDS-KeyCredentialLink and audit 5136 on that attribute",
            WriteRbcd => "clear msDS-AllowedToActOnBehalfOfOtherIdentity and deny writes to it",
            WriteSpn => "deny servicePrincipalName writes; put service accounts in Protected Users or use gMSA",
            WriteAltSecurityIdentities => "deny writes to altSecurityIdentities and enforce strong certificate mapping (KB5014754)",
            WriteAllowedToDelegateTo => "remove the delegation; mark Tier-0 accounts sensitive and non-delegatable",
            WriteGpLink => "restrict gPLink writes on the OU; review linked GPOs",
            ReadGmsaPassword => "narrow msDS-GroupMSAMembership to the hosts that actually run the service",
            ReadLapsPassword => "scope the LAPS read ACL to the machine's admins; enable Windows LAPS encryption",
            DcsyncGetChanges | DcsyncGetChangesAll | DcsyncGetChangesFiltered =>
                "remove replication rights from the domain head for anyone but DCs and AAD Connect",
            ReanimateTombstones => "remove the right; audit object restores",
            Enroll => "restrict template enrollment and fix the template flags (manager approval, no SAN)",
            CreateDmsa => "deny CreateChild for msDS-DelegatedManagedServiceAccount on OUs low-privilege users control",
            CreateChild(_) => "scope CreateChild to the classes the delegation needs",
        }
    }
}

/// Where a grant came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// The security descriptor's owner field.
    Owner,
    /// An allow ACE in the DACL.
    Dacl,
}

/// One trustee holding one primitive over the object the descriptor belongs to.
#[derive(Clone, Debug)]
pub struct Grant {
    pub trustee: Sid,
    pub primitive: ControlPrimitive,
    /// The ACE carried `INHERITED_ACE` (0x10) — it came from a parent container.
    pub inherited: bool,
    pub source: Source,
}

impl Grant {
    /// Well-known trustees (Everyone, SYSTEM, BUILTIN\\Administrators, …) are usually noise
    /// in an attack graph; callers normally drop them.
    pub fn trustee_is_well_known(&self) -> bool {
        self.trustee.is_well_known()
    }
}

const INHERITED_ACE: u8 = 0x10;

/// Every primitive one allow-ACE grants. Deny ACEs yield nothing.
pub fn classify(ace: &Ace) -> Vec<ControlPrimitive> {
    classify_with(ace, &SchemaMap::new())
}

/// [`classify`], additionally resolving forest-specific attributes through `schema`.
pub fn classify_with(ace: &Ace, schema: &SchemaMap) -> Vec<ControlPrimitive> {
    use ControlPrimitive as P;

    if !ace.is_allow() {
        return Vec::new();
    }
    let m = ace.mask;
    let g = ace.object_type;
    let mut v = Vec::new();

    if m.contains(AccessMask::GENERIC_ALL) {
        v.push(P::GenericAll);
    }
    if m.contains(AccessMask::WRITE_DAC) {
        v.push(P::WriteDacl);
    }
    if m.contains(AccessMask::WRITE_OWNER) {
        v.push(P::WriteOwner);
    }
    if m.contains(AccessMask::GENERIC_WRITE) {
        v.push(P::GenericWrite);
    }

    // Extended rights. No GUID = every extended right on the object.
    if m.contains(AccessMask::CONTROL_ACCESS) {
        match &g {
            None => v.push(P::AllExtendedRights),
            Some(g) if catalog::FORCE_CHANGE_PASSWORD.matches(g) => v.push(P::ForceChangePassword),
            Some(g) if catalog::REPL_GET_CHANGES_ALL.matches(g) => v.push(P::DcsyncGetChangesAll),
            Some(g) if catalog::REPL_GET_CHANGES.matches(g) => v.push(P::DcsyncGetChanges),
            Some(g) if catalog::REPL_GET_CHANGES_FILTERED.matches(g) => {
                v.push(P::DcsyncGetChangesFiltered)
            }
            Some(g) if catalog::REANIMATE_TOMBSTONES.matches(g) => v.push(P::ReanimateTombstones),
            Some(g) if catalog::is_enrollment_right(g) => v.push(P::Enroll),
            Some(_) => {}
        }
    }

    // Attribute writes. No GUID = every attribute.
    if m.contains(AccessMask::WRITE_PROP) {
        match &g {
            None => v.push(P::GenericWrite),
            Some(g) if catalog::MEMBER.matches(g) => v.push(P::AddMember),
            Some(g) if catalog::KEY_CREDENTIAL_LINK.matches(g) => v.push(P::AddKeyCredential),
            Some(g) if catalog::RBCD.matches(g) => v.push(P::WriteRbcd),
            Some(g) if catalog::SPN.matches(g) => v.push(P::WriteSpn),
            Some(g) if catalog::ALT_SECURITY_IDENTITIES.matches(g) => {
                v.push(P::WriteAltSecurityIdentities)
            }
            Some(g) if catalog::ALLOWED_TO_DELEGATE_TO.matches(g) => {
                v.push(P::WriteAllowedToDelegateTo)
            }
            Some(g) if catalog::GP_LINK.matches(g) => v.push(P::WriteGpLink),
            Some(_) => {}
        }
    }

    // Attribute reads only matter for the two secret-bearing attributes, both forest-specific.
    if m.contains(AccessMask::READ_PROP) {
        if let Some(g) = &g {
            if schema.is_managed_password_attr(g) {
                v.push(P::ReadGmsaPassword);
            } else if schema.is_laps_attr(g) {
                v.push(P::ReadLapsPassword);
            }
        }
    }

    // Validated writes.
    if m.contains(AccessMask::SELF) {
        match &g {
            Some(g) if catalog::SELF_MEMBERSHIP.matches(g) => v.push(P::AddSelfToGroup),
            Some(g) if catalog::VALIDATED_SPN.matches(g) => v.push(P::WriteSpn),
            _ => {}
        }
    }

    // Child creation. Scoped to the dMSA class this is BadSuccessor.
    if m.contains(AccessMask::CREATE_CHILD) {
        match &g {
            Some(g) if schema.is_dmsa_class(g) => v.push(P::CreateDmsa),
            other => v.push(P::CreateChild(*other)),
        }
    }

    v.dedup();
    v
}

/// Every grant a descriptor hands out: the owner, plus one entry per allow-ACE primitive.
pub fn grants(sd: &SecurityDescriptor) -> Vec<Grant> {
    grants_with(sd, &SchemaMap::new())
}

/// [`grants`], resolving forest-specific attributes through `schema`.
pub fn grants_with(sd: &SecurityDescriptor, schema: &SchemaMap) -> Vec<Grant> {
    let mut out = Vec::new();

    if let Some(owner) = &sd.owner {
        out.push(Grant {
            trustee: owner.clone(),
            primitive: ControlPrimitive::Owns,
            inherited: false,
            source: Source::Owner,
        });
    }

    for ace in sd.dacl.iter().flat_map(|d| &d.aces) {
        let inherited = ace.flags & INHERITED_ACE != 0;
        for primitive in classify_with(ace, schema) {
            out.push(Grant {
                trustee: ace.trustee.clone(),
                primitive,
                inherited,
                source: Source::Dacl,
            });
        }
    }
    out
}

/// True if the set of primitives held over the domain head amounts to DCSync.
///
/// `Get-Changes` alone is not enough; it needs `Get-Changes-All` (or a blanket right).
pub fn is_dcsync(primitives: &[ControlPrimitive]) -> bool {
    use ControlPrimitive::*;
    let has = |p: ControlPrimitive| primitives.contains(&p);
    if has(GenericAll) || has(AllExtendedRights) {
        return true;
    }
    has(DcsyncGetChangesAll) && (has(DcsyncGetChanges) || has(DcsyncGetChangesFiltered))
}
