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
    /// Unique identifier for this resource.
    pub id: String,
    /// Permissions required to access this resource.
    pub required_permissions: PermissionSet,
    /// Classification or type of this resource.
    pub resource_type: String,
}

impl Resource {
    /// Create a new resource with no required permissions.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            required_permissions: PermissionSet::identity(),
            resource_type: String::new(),
        }
    }

    /// Add a required permission for accessing this resource.
    pub fn requires(mut self, perm: AtomicPermission) -> Self {
        self.required_permissions.extend([perm]);
        self
    }

    /// Set the resource type classification.
    pub fn resource_type(mut self, rt: impl Into<String>) -> Self {
        self.resource_type = rt.into();
        self
    }

    /// Returns `true` if the subject has all required permissions.
    pub fn can_access(&self, subject: &Subject) -> bool {
        let effective = subject.effective_permissions();
        self.required_permissions.is_subset_of(&effective)
    }

    /// Returns the set of required permissions the subject lacks.
    pub fn missing_permissions(&self, subject: &Subject) -> PermissionSet {
        let effective = subject.effective_permissions();
        self.required_permissions.difference(&effective)
    }
}
