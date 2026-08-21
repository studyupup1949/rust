//! Runtime `schemaIDGUID` lookup.
//!
//! Attributes and classes added by a schema extension get a GUID generated at install
//! time, so they cannot be hard-coded: LAPS (`ms-Mcs-AdmPwd`, `msLAPS-Password`,
//! `msLAPS-EncryptedPassword`), the gMSA blob (`msDS-ManagedPassword`) and the dMSA class
//! (`msDS-DelegatedManagedServiceAccount`) all differ per forest. Fill this map from
//! `CN=Schema,CN=Configuration,<root>` — `(lDAPDisplayName, schemaIDGUID)` — and pass it to
//! [`crate::grants_with`] so those ACEs resolve into real primitives.

use std::collections::HashMap;
use windows_sddl::sid::Guid;

/// Attribute / class names this crate reacts to when they are present in the map.
pub mod names {
    /// Legacy LAPS password attribute (readable = local admin on the machine).
    pub const LAPS_LEGACY: &str = "ms-Mcs-AdmPwd";
    /// Windows LAPS cleartext password attribute.
    pub const LAPS_PASSWORD: &str = "msLAPS-Password";
    /// Windows LAPS DPAPI-NG-encrypted password attribute.
    pub const LAPS_ENCRYPTED: &str = "msLAPS-EncryptedPassword";
    /// gMSA managed-password blob.
    pub const MANAGED_PASSWORD: &str = "msDS-ManagedPassword";
    /// Delegated MSA class — `CreateChild` for it on an OU is the BadSuccessor primitive.
    pub const DMSA_CLASS: &str = "msDS-DelegatedManagedServiceAccount";
}

/// `lDAPDisplayName` ⇄ `schemaIDGUID`, case-insensitive by name.
#[derive(Clone, Debug, Default)]
pub struct SchemaMap {
    by_guid: HashMap<Guid, String>,
    by_name: HashMap<String, Guid>,
}

impl SchemaMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from `(lDAPDisplayName, schemaIDGUID)` pairs as read from the schema NC.
    pub fn from_entries<I, S>(entries: I) -> Self
    where
        I: IntoIterator<Item = (S, Guid)>,
        S: AsRef<str>,
    {
        let mut m = SchemaMap::new();
        for (name, guid) in entries {
            m.insert(name.as_ref(), guid);
        }
        m
    }

    pub fn insert(&mut self, name: &str, guid: Guid) {
        self.by_guid.insert(guid, name.to_string());
        self.by_name.insert(name.to_ascii_lowercase(), guid);
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// `lDAPDisplayName` for a GUID seen in an ACE.
    pub fn name(&self, g: &Guid) -> Option<&str> {
        self.by_guid.get(g).map(String::as_str)
    }

    /// `schemaIDGUID` for an attribute or class name.
    pub fn guid(&self, name: &str) -> Option<Guid> {
        self.by_name.get(&name.to_ascii_lowercase()).copied()
    }

    /// True if `g` is any of the LAPS password attributes present in this forest.
    pub fn is_laps_attr(&self, g: &Guid) -> bool {
        matches!(
            self.name(g),
            Some(n) if n.eq_ignore_ascii_case(names::LAPS_LEGACY)
                || n.eq_ignore_ascii_case(names::LAPS_PASSWORD)
                || n.eq_ignore_ascii_case(names::LAPS_ENCRYPTED)
        )
    }

    /// True if `g` is the gMSA managed-password blob attribute.
    pub fn is_managed_password_attr(&self, g: &Guid) -> bool {
        matches!(self.name(g), Some(n) if n.eq_ignore_ascii_case(names::MANAGED_PASSWORD))
    }

    /// True if `g` is the delegated-MSA class (BadSuccessor).
    pub fn is_dmsa_class(&self, g: &Guid) -> bool {
        matches!(self.name(g), Some(n) if n.eq_ignore_ascii_case(names::DMSA_CLASS))
    }
}
