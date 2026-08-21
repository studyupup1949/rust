//! Fixed, forest-independent GUIDs that appear in AD object ACEs.
//!
//! Only GUIDs that are identical in every forest live here. Anything whose `schemaIDGUID`
//! is generated at schema-extension time — LAPS (`ms-Mcs-AdmPwd`, `msLAPS-*`), gMSA/dMSA
//! attributes and classes — is deliberately absent: resolve those at runtime with
//! [`crate::SchemaMap`], which reads them from the forest schema.

use windows_sddl::sid::Guid;

/// What an `object_type` GUID in an object-ACE denotes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuidClass {
    /// Control-access right — meaningful with `CONTROL_ACCESS`.
    ExtendedRight,
    /// Single attribute — meaningful with `READ_PROP` / `WRITE_PROP`.
    Attribute,
    /// Validated write — meaningful with `SELF`.
    ValidatedWrite,
}

/// One catalog entry: a GUID with a name and the mask bit it pairs with.
pub struct KnownGuid {
    pub name: &'static str,
    pub class: GuidClass,
    guid: &'static str,
}

impl KnownGuid {
    const fn new(name: &'static str, class: GuidClass, guid: &'static str) -> Self {
        KnownGuid { name, class, guid }
    }
    pub fn guid(&self) -> Guid {
        Guid::parse(self.guid).expect("static catalog GUID is valid")
    }
    pub fn matches(&self, g: &Guid) -> bool {
        self.guid() == *g
    }
}

use GuidClass::{Attribute, ExtendedRight, ValidatedWrite};

// ── extended rights ───────────────────────────────────────────────────────────

/// `DS-Replication-Get-Changes` — half of DCSync.
pub const REPL_GET_CHANGES: KnownGuid = KnownGuid::new(
    "DS-Replication-Get-Changes",
    ExtendedRight,
    "1131f6aa-9c07-11d1-f79f-00c04fc2dcd2",
);
/// `DS-Replication-Get-Changes-All` — the half that carries secrets.
pub const REPL_GET_CHANGES_ALL: KnownGuid = KnownGuid::new(
    "DS-Replication-Get-Changes-All",
    ExtendedRight,
    "1131f6ad-9c07-11d1-f79f-00c04fc2dcd2",
);
/// `DS-Replication-Get-Changes-In-Filtered-Set` — RODC-filtered replication.
pub const REPL_GET_CHANGES_FILTERED: KnownGuid = KnownGuid::new(
    "DS-Replication-Get-Changes-In-Filtered-Set",
    ExtendedRight,
    "89e95b76-444d-4c62-991a-0facbeda640c",
);
/// `User-Force-Change-Password` — reset a password without knowing the old one.
pub const FORCE_CHANGE_PASSWORD: KnownGuid = KnownGuid::new(
    "User-Force-Change-Password",
    ExtendedRight,
    "00299570-246d-11d0-a768-00aa006e0529",
);
/// `Reanimate-Tombstones` — resurrect deleted objects (revives stale privilege).
pub const REANIMATE_TOMBSTONES: KnownGuid = KnownGuid::new(
    "Reanimate-Tombstones",
    ExtendedRight,
    "45ec5156-db7e-47bb-b53f-dbeb2d03c40f",
);
/// `Certificate-Enrollment` on an AD CS template.
pub const ENROLLMENT: KnownGuid = KnownGuid::new(
    "Certificate-Enrollment",
    ExtendedRight,
    "0e10c968-78fb-11d2-90d4-00c04f79dc55",
);
/// `Certificate-AutoEnrollment` on an AD CS template.
pub const AUTO_ENROLLMENT: KnownGuid = KnownGuid::new(
    "Certificate-AutoEnrollment",
    ExtendedRight,
    "a05b8cc2-17bc-4802-a710-e7c15ab866a2",
);

// ── attributes ────────────────────────────────────────────────────────────────

/// `member` — write it to add anyone to a group.
pub const MEMBER: KnownGuid =
    KnownGuid::new("member", Attribute, "bf9679c0-0de6-11d0-a285-00aa003049e2");
/// `msDS-KeyCredentialLink` — write it for Shadow Credentials.
pub const KEY_CREDENTIAL_LINK: KnownGuid = KnownGuid::new(
    "msDS-KeyCredentialLink",
    Attribute,
    "5b47d60f-6090-40b2-9f37-2a4de88f3063",
);
/// `msDS-AllowedToActOnBehalfOfOtherIdentity` — write it for RBCD.
pub const RBCD: KnownGuid = KnownGuid::new(
    "msDS-AllowedToActOnBehalfOfOtherIdentity",
    Attribute,
    "3f78c3e5-f79a-46bd-a0b8-9d18116ddc79",
);
/// `servicePrincipalName` — write it to make an account Kerberoastable.
pub const SPN: KnownGuid = KnownGuid::new(
    "servicePrincipalName",
    Attribute,
    "f3a64788-5306-11d1-a9c5-0000f80367c1",
);
/// `altSecurityIdentities` — write it to bind an attacker certificate to the account.
pub const ALT_SECURITY_IDENTITIES: KnownGuid = KnownGuid::new(
    "altSecurityIdentities",
    Attribute,
    "00fbf30c-91fe-11d1-aebc-0000f80367c1",
);
/// `msDS-AllowedToDelegateTo` — write it for constrained delegation abuse.
pub const ALLOWED_TO_DELEGATE_TO: KnownGuid = KnownGuid::new(
    "msDS-AllowedToDelegateTo",
    Attribute,
    "800d94d7-b7a1-42a1-b14d-7cae1423d07f",
);
/// `gPLink` — write it on an OU to attach a hostile GPO.
pub const GP_LINK: KnownGuid =
    KnownGuid::new("gPLink", Attribute, "f30e3bbe-9ff0-11d1-b603-0000f80367c1");

// ── validated writes ──────────────────────────────────────────────────────────

/// `Self-Membership` — add *yourself* to a group (shares the `member` GUID).
pub const SELF_MEMBERSHIP: KnownGuid = KnownGuid::new(
    "Self-Membership",
    ValidatedWrite,
    "bf9679c0-0de6-11d0-a285-00aa003049e2",
);
/// `Validated-SPN` — set an SPN on yourself (shares the `servicePrincipalName` GUID).
pub const VALIDATED_SPN: KnownGuid = KnownGuid::new(
    "Validated-SPN",
    ValidatedWrite,
    "f3a64788-5306-11d1-a9c5-0000f80367c1",
);

/// Every fixed GUID this crate knows, for name resolution and reporting.
pub const ALL: &[&KnownGuid] = &[
    &REPL_GET_CHANGES,
    &REPL_GET_CHANGES_ALL,
    &REPL_GET_CHANGES_FILTERED,
    &FORCE_CHANGE_PASSWORD,
    &REANIMATE_TOMBSTONES,
    &ENROLLMENT,
    &AUTO_ENROLLMENT,
    &MEMBER,
    &KEY_CREDENTIAL_LINK,
    &RBCD,
    &SPN,
    &ALT_SECURITY_IDENTITIES,
    &ALLOWED_TO_DELEGATE_TO,
    &GP_LINK,
];

/// Human-readable name for a GUID, or `None` if it is forest-specific.
///
/// `class` disambiguates the two GUIDs that mean different things depending on the
/// mask bit (`member` / `Self-Membership`, `servicePrincipalName` / `Validated-SPN`).
pub fn name_of(g: &Guid, class: GuidClass) -> Option<&'static str> {
    if class == ValidatedWrite {
        for k in [&SELF_MEMBERSHIP, &VALIDATED_SPN] {
            if k.matches(g) {
                return Some(k.name);
            }
        }
    }
    ALL.iter()
        .find(|k| k.class == class && k.matches(g))
        .map(|k| k.name)
}

/// True if the GUID is either DCSync half (or the filtered-set variant).
pub fn is_replication_right(g: &Guid) -> bool {
    REPL_GET_CHANGES.matches(g)
        || REPL_GET_CHANGES_ALL.matches(g)
        || REPL_GET_CHANGES_FILTERED.matches(g)
}

/// True if the GUID grants certificate enrollment (manual or auto).
pub fn is_enrollment_right(g: &Guid) -> bool {
    ENROLLMENT.matches(g) || AUTO_ENROLLMENT.matches(g)
}
