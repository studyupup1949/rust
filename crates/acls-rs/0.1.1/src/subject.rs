//! Subject type representing users/roles with permissions.

use crate::algebra::{MonoidAction, Semigroup};
use crate::calculation::HasPermissions;
use crate::permission::{
    AtomicPermission, GrantDenialPair, PermissionDelta, PermissionSet, TemporalPermissionSet,
    Timestamp,
};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Errors from building a [`Subject`] via [`SubjectBuilder`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuilderError {
    /// A required field was not set.
    MissingField(&'static str),
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuilderError::MissingField(field) => write!(f, "missing required field: {}", field),
        }
    }
}

impl std::error::Error for BuilderError {}

/// A subject (user, service account, etc.) with permissions.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Subject {
    pub id: String,
    pub permissions: GrantDenialPair,
    pub roles: Vec<String>,
    pub temporal_permissions: TemporalPermissionSet,
}

impl Subject {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            permissions: GrantDenialPair::empty(),
            roles: Vec::new(),
            temporal_permissions: TemporalPermissionSet::new(),
        }
    }

    pub fn effective_permissions(&self) -> PermissionSet {
        let base = self.permissions.effective_permissions();
        let temporal = self.temporal_permissions.currently_effective();
        base.combine(temporal)
    }

    pub fn effective_permissions_at(&self, time: Timestamp) -> PermissionSet {
        let base = self.permissions.effective_permissions();
        let temporal = self.temporal_permissions.effective_at(time);
        base.combine(temporal)
    }

    pub fn has_permission(&self, perm: &AtomicPermission) -> bool {
        self.effective_permissions().contains(perm)
    }

    pub fn grant(&mut self, perm: AtomicPermission) {
        self.permissions.grants.extend([perm]);
    }

    pub fn revoke(&mut self, perm: AtomicPermission) {
        self.permissions.grants = self
            .permissions
            .grants
            .difference(&PermissionSet::from([perm]));
    }

    pub fn deny(&mut self, perm: AtomicPermission) {
        self.permissions.denials.extend([perm]);
    }

    pub fn builder() -> SubjectBuilder {
        SubjectBuilder::default()
    }
}

impl Default for SubjectBuilder {
    fn default() -> Self {
        Self {
            id: None,
            permissions: GrantDenialPair::empty(),
            roles: Vec::new(),
        }
    }
}
pub struct SubjectBuilder {
    id: Option<String>,
    permissions: GrantDenialPair,
    roles: Vec<String>,
}

impl SubjectBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    pub fn grant(mut self, perm: AtomicPermission) -> Self {
        self.permissions.grants.extend([perm]);
        self
    }

    pub fn deny(mut self, perm: AtomicPermission) -> Self {
        self.permissions.denials.extend([perm]);
        self
    }

    pub fn build(self) -> Result<Subject, BuilderError> {
        Ok(Subject {
            id: self.id.ok_or(BuilderError::MissingField("id"))?,
            permissions: self.permissions,
            roles: self.roles,
            temporal_permissions: TemporalPermissionSet::new(),
        })
    }
}

// MonoidAction: PermissionDelta acts on Subject
impl MonoidAction<PermissionDelta, Subject> for PermissionDelta {
    fn act(delta: PermissionDelta, mut subject: Subject) -> Subject {
        subject.permissions.grants = delta.apply_to(subject.permissions.grants);
        subject
    }
}

// Implement HasPermissions for Subject
impl HasPermissions for Subject {
    fn permissions(&self) -> &GrantDenialPair {
        &self.permissions
    }

    fn permissions_mut(&mut self) -> &mut GrantDenialPair {
        &mut self.permissions
    }

    fn effective_permissions_at(&self, time: Timestamp) -> PermissionSet {
        self.effective_permissions_at(time)
    }
}
