//! Permission effect tracking and building.

use crate::permission::{
    AtomicPermission, DenialSet, GrantDenialPair, PermissionDelta, PermissionSet,
};

/// Tracks the effects of a permission change.
///
/// Records what was added, removed, and the net change in effective permissions.
///
/// # Examples
///
/// ```
/// use acls_rs::calculation::PermissionEffect;
/// use acls_rs::permission::{AtomicPermission, PermissionSet, GrantDenialPair};
///
/// let before = GrantDenialPair::new(
///     PermissionSet::from([AtomicPermission::new("file", "read")]),
///     PermissionSet::new(),
/// );
///
/// let after = GrantDenialPair::new(
///     PermissionSet::from([
///         AtomicPermission::new("file", "read"),
///         AtomicPermission::new("file", "write"),
///     ]),
///     PermissionSet::new(),
/// );
///
/// let effect = PermissionEffect::from_transition(&before, &after);
/// assert_eq!(effect.grants_added.len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionEffect {
    /// Grants that were added.
    pub grants_added: PermissionSet,
    /// Grants that were removed.
    pub grants_removed: PermissionSet,
    /// Denials that were added.
    pub denials_added: DenialSet,
    /// Denials that were removed.
    pub denials_removed: DenialSet,
    /// Net change in effective permissions.
    pub net_effective_change: PermissionSet,
}

impl PermissionEffect {
    /// Create a permission effect from a before/after transition.
    pub fn from_transition(before: &GrantDenialPair, after: &GrantDenialPair) -> Self {
        let grants_added = after.grants.difference(&before.grants);
        let grants_removed = before.grants.difference(&after.grants);
        let denials_added = after.denials.difference(&before.denials);
        let denials_removed = before.denials.difference(&after.denials);

        let before_effective = before.effective_permissions();
        let after_effective = after.effective_permissions();

        // Net change is what's in after but not before
        let net_effective_change = after_effective.difference(&before_effective);

        Self {
            grants_added,
            grants_removed,
            denials_added,
            denials_removed,
            net_effective_change,
        }
    }

    /// Check if there are any changes.
    pub fn is_empty(&self) -> bool {
        self.grants_added.is_empty()
            && self.grants_removed.is_empty()
            && self.denials_added.is_empty()
            && self.denials_removed.is_empty()
    }

    /// Get the total number of changes.
    pub fn total_changes_count(&self) -> usize {
        self.grants_added.len()
            + self.grants_removed.len()
            + self.denials_added.len()
            + self.denials_removed.len()
    }

    /// Convert to a permission delta.
    pub fn to_delta(&self) -> PermissionDelta {
        PermissionDelta::new(self.grants_added.clone(), self.grants_removed.clone())
    }

    /// Get a delta for grants.
    pub fn grants_delta(&self) -> PermissionDelta {
        PermissionDelta::new(self.grants_added.clone(), self.grants_removed.clone())
    }

    /// Get a delta for denials.
    pub fn denials_delta(&self) -> PermissionDelta {
        PermissionDelta::new(self.denials_added.clone(), self.denials_removed.clone())
    }
}

/// Builder for constructing permission effects.
///
/// Provides a fluent API for building complex permission changes with automatic
/// effect tracking.
///
/// # Examples
///
/// ```
/// use acls_rs::calculation::PermissionEffectBuilder;
/// use acls_rs::permission::{AtomicPermission, GrantDenialPair, PermissionSet};
///
/// let initial = GrantDenialPair::new(
///     PermissionSet::from([AtomicPermission::new("file", "read")]),
///     PermissionSet::new(),
/// );
///
/// let (updated, effect) = PermissionEffectBuilder::new(initial)
///     .grant(AtomicPermission::new("file", "write"))
///     .deny(AtomicPermission::new("file", "delete"))
///     .build();
///
/// assert_eq!(effect.grants_added.len(), 1);
/// assert_eq!(effect.denials_added.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct PermissionEffectBuilder {
    original: GrantDenialPair,
    current: GrantDenialPair,
}

impl PermissionEffectBuilder {
    /// Create a new builder with initial permissions.
    pub fn new(initial: GrantDenialPair) -> Self {
        Self {
            original: initial.clone(),
            current: initial,
        }
    }

    /// Grant a permission.
    pub fn grant(mut self, perm: AtomicPermission) -> Self {
        self.current.grants.extend([perm]);
        self
    }

    /// Revoke a grant.
    pub fn revoke(mut self, perm: AtomicPermission) -> Self {
        self.current.grants = self.current.grants.difference(&PermissionSet::from([perm]));
        self
    }

    /// Add a denial.
    pub fn deny(mut self, perm: AtomicPermission) -> Self {
        self.current.denials.extend([perm]);
        self
    }

    /// Remove a denial.
    pub fn undeny(mut self, perm: AtomicPermission) -> Self {
        self.current.denials = self
            .current
            .denials
            .difference(&PermissionSet::from([perm]));
        self
    }

    /// Grant a permission from namespace and action strings.
    pub fn grant_str(self, namespace: &str, action: &str) -> Self {
        self.grant(AtomicPermission::new(namespace, action))
    }

    /// Deny a permission from namespace and action strings.
    pub fn deny_str(self, namespace: &str, action: &str) -> Self {
        self.deny(AtomicPermission::new(namespace, action))
    }

    /// Apply a permission delta.
    pub fn apply_delta(mut self, delta: &PermissionDelta) -> Self {
        self.current.grants = delta.apply_to(self.current.grants);
        self
    }

    /// Build the final permissions and compute the effect.
    ///
    /// Returns a tuple of (updated_permissions, effect).
    pub fn build(self) -> (GrantDenialPair, PermissionEffect) {
        let effect = PermissionEffect::from_transition(&self.original, &self.current);
        (self.current, effect)
    }

    /// Check if there are any changes so far.
    pub fn has_changes(&self) -> bool {
        self.current != self.original
    }

    /// Preview the effect without consuming the builder.
    pub fn preview_effect(&self) -> PermissionEffect {
        PermissionEffect::from_transition(&self.original, &self.current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_from_transition() {
        let before = GrantDenialPair::new(
            PermissionSet::from([AtomicPermission::new("file", "read")]),
            PermissionSet::new(),
        );

        let after = GrantDenialPair::new(
            PermissionSet::from([
                AtomicPermission::new("file", "read"),
                AtomicPermission::new("file", "write"),
            ]),
            PermissionSet::from([AtomicPermission::new("file", "delete")]),
        );

        let effect = PermissionEffect::from_transition(&before, &after);

        assert_eq!(effect.grants_added.len(), 1);
        assert!(effect
            .grants_added
            .contains(&AtomicPermission::new("file", "write")));
        assert_eq!(effect.denials_added.len(), 1);
        assert!(effect
            .denials_added
            .contains(&AtomicPermission::new("file", "delete")));
    }

    #[test]
    fn test_effect_builder() {
        let initial = GrantDenialPair::new(
            PermissionSet::from([AtomicPermission::new("file", "read")]),
            PermissionSet::new(),
        );

        let (updated, effect) = PermissionEffectBuilder::new(initial)
            .grant(AtomicPermission::new("file", "write"))
            .deny(AtomicPermission::new("file", "delete"))
            .build();

        assert_eq!(effect.grants_added.len(), 1);
        assert_eq!(effect.denials_added.len(), 1);
        assert_eq!(updated.grants.len(), 2);
        assert_eq!(updated.denials.len(), 1);
    }

    #[test]
    fn test_builder_revoke() {
        let initial = GrantDenialPair::new(
            PermissionSet::from([
                AtomicPermission::new("file", "read"),
                AtomicPermission::new("file", "write"),
            ]),
            PermissionSet::new(),
        );

        let (updated, effect) = PermissionEffectBuilder::new(initial)
            .revoke(AtomicPermission::new("file", "write"))
            .build();

        assert_eq!(effect.grants_removed.len(), 1);
        assert_eq!(updated.grants.len(), 1);
    }

    #[test]
    fn test_builder_preview() {
        let initial = GrantDenialPair::new(
            PermissionSet::from([AtomicPermission::new("file", "read")]),
            PermissionSet::new(),
        );

        let builder =
            PermissionEffectBuilder::new(initial).grant(AtomicPermission::new("file", "write"));

        let preview = builder.preview_effect();
        assert_eq!(preview.grants_added.len(), 1);
        assert!(builder.has_changes());
    }
}
