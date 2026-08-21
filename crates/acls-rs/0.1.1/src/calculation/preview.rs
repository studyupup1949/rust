//! Permission preview for safe exploration of changes.

use super::{HasPermissions, PermissionEffect};
use crate::permission::{GrantDenialPair, PermissionDelta, PermissionSet, Timestamp};

/// A preview of permission changes before committing them.
///
/// Allows exploring permission modifications safely, with the ability to
/// commit or discard changes.
///
/// # Examples
///
/// ```
/// use acls_rs::calculation::PermissionPreview;
/// use acls_rs::permission::{AtomicPermission, PermissionDelta};
/// use acls_rs::Subject;
///
/// let subject = Subject::builder()
///     .id("alice")
///     .grant(AtomicPermission::new("file", "read"))
///     .build()
///     .unwrap();
///
/// // Preview adding write permission
/// let delta = PermissionDelta::builder()
///     .grant_str("file", "write")
///     .build();
///
/// let preview = PermissionPreview::new(subject)
///     .apply_delta(&delta);
///
/// if preview.has_changes() {
///     println!("Would add write permission");
///     let updated = preview.commit();
/// }
/// ```
#[derive(Debug)]
pub struct PermissionPreview<T: HasPermissions> {
    original: T,
    predicted: T,
}

impl<T: HasPermissions + Clone> PermissionPreview<T> {
    /// Create a new preview with the original state.
    pub fn new(original: T) -> Self {
        Self {
            predicted: original.clone(),
            original,
        }
    }

    /// Apply a permission delta to the preview.
    pub fn apply_delta(mut self, delta: &PermissionDelta) -> Self {
        let current_perms = self.predicted.permissions().clone();
        let new_grants = delta.apply_to(current_perms.grants);
        *self.predicted.permissions_mut() = GrantDenialPair::new(new_grants, current_perms.denials);
        self
    }

    /// Apply a grant/denial pair to the preview.
    pub fn apply_grant_denial(mut self, gd: &GrantDenialPair) -> Self {
        use crate::algebra::Semigroup;
        let current = self.predicted.permissions().clone();
        *self.predicted.permissions_mut() = current.combine(gd.clone());
        self
    }

    /// Grant a permission in the preview.
    pub fn grant(mut self, perm: crate::permission::AtomicPermission) -> Self {
        self.predicted.permissions_mut().grants.extend([perm]);
        self
    }

    /// Deny a permission in the preview.
    pub fn deny(mut self, perm: crate::permission::AtomicPermission) -> Self {
        self.predicted.permissions_mut().denials.extend([perm]);
        self
    }

    /// Check if there are any changes between original and predicted.
    pub fn has_changes(&self) -> bool {
        self.original.permissions() != self.predicted.permissions()
    }

    /// Compute the changes as a permission delta.
    pub fn changes(&self) -> PermissionDelta {
        let original_grants = &self.original.permissions().grants;
        let predicted_grants = &self.predicted.permissions().grants;

        let added = predicted_grants.difference(original_grants);
        let removed = original_grants.difference(predicted_grants);

        PermissionDelta::new(added, removed)
    }

    /// Compute the effect of the changes.
    pub fn effect(&self) -> PermissionEffect {
        PermissionEffect::from_transition(self.original.permissions(), self.predicted.permissions())
    }

    /// Get the predicted effective permissions.
    pub fn effective_permissions(&self) -> PermissionSet {
        self.predicted.effective_permissions()
    }

    /// Get the predicted effective permissions at a specific time.
    pub fn effective_permissions_at(&self, time: Timestamp) -> PermissionSet {
        self.predicted.effective_permissions_at(time)
    }

    /// Get a reference to the original state.
    pub fn original(&self) -> &T {
        &self.original
    }

    /// Get a reference to the predicted state.
    pub fn predicted(&self) -> &T {
        &self.predicted
    }

    /// Commit the changes and return the updated state.
    pub fn commit(self) -> T {
        self.predicted
    }

    /// Discard the changes and return the original state.
    pub fn discard(self) -> T {
        self.original
    }

    /// Preview at a specific timestamp (for temporal permissions).
    pub fn preview_at(&self, time: Timestamp) -> PermissionSet {
        self.predicted.effective_permissions_at(time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission::AtomicPermission;
    use crate::Subject;

    #[test]
    fn test_preview_delta() {
        let subject = Subject::builder()
            .id("alice")
            .grant(AtomicPermission::new("file", "read"))
            .build()
            .unwrap();

        let delta = PermissionDelta::builder()
            .grant_str("file", "write")
            .build();

        let preview = PermissionPreview::new(subject).apply_delta(&delta);

        assert!(preview.has_changes());
        let changes = preview.changes();
        assert_eq!(changes.add.len(), 1);
    }

    #[test]
    fn test_preview_commit() {
        let subject = Subject::builder()
            .id("bob")
            .grant(AtomicPermission::new("file", "read"))
            .build()
            .unwrap();

        let preview = PermissionPreview::new(subject).grant(AtomicPermission::new("file", "write"));

        assert!(preview.has_changes());
        let updated = preview.commit();

        assert!(updated.has_permission(&AtomicPermission::new("file", "write")));
    }

    #[test]
    fn test_preview_discard() {
        let subject = Subject::builder()
            .id("charlie")
            .grant(AtomicPermission::new("file", "read"))
            .build()
            .unwrap();

        let preview = PermissionPreview::new(subject).grant(AtomicPermission::new("file", "write"));

        let original = preview.discard();
        assert!(!original.has_permission(&AtomicPermission::new("file", "write")));
    }

    #[test]
    fn test_preview_effect() {
        let subject = Subject::builder()
            .id("diana")
            .grant(AtomicPermission::new("file", "read"))
            .build()
            .unwrap();

        let preview = PermissionPreview::new(subject)
            .grant(AtomicPermission::new("file", "write"))
            .deny(AtomicPermission::new("file", "delete"));

        let effect = preview.effect();
        assert_eq!(effect.grants_added.len(), 1);
        assert_eq!(effect.denials_added.len(), 1);
    }
}
