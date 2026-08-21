//! Resource type with required permissions.

use crate::algebra::Monoid;
use crate::permission::{AtomicPermission, PermissionSet};
use crate::Subject;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A resource that requires certain permissions to access.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Resource {
    pub id: String,
    pub required_permissions: PermissionSet,
    pub resource_type: String,
}

impl Resource {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            required_permissions: PermissionSet::identity(),
            resource_type: String::new(),
        }
    }

    pub fn requires(mut self, perm: AtomicPermission) -> Self {
        self.required_permissions.extend([perm]);
        self
    }

    pub fn resource_type(mut self, rt: impl Into<String>) -> Self {
        self.resource_type = rt.into();
        self
    }

    pub fn can_access(&self, subject: &Subject) -> bool {
        let effective = subject.effective_permissions();
        self.required_permissions.is_subset_of(&effective)
    }

    pub fn missing_permissions(&self, subject: &Subject) -> PermissionSet {
        let effective = subject.effective_permissions();
        self.required_permissions.difference(&effective)
    }
}
