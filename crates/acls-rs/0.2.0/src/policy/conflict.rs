//! Conflict resolution strategies for ABAC.

use crate::algebra::{JoinSemilattice, MeetSemilattice};
use crate::permission::GrantDenialPair;
#[cfg(test)]
use crate::permission::PermissionSet;

/// Trait for resolving conflicts when multiple permission rules match.
///
/// When multiple ABAC rules apply to the same context, a conflict resolver
/// determines how to combine their results into a single decision.
pub trait ConflictResolver {
    /// Resolve a set of conflicting permission results into a single result.
    ///
    /// # Arguments
    ///
    /// * `results` - Permission results from multiple matching rules
    ///
    /// # Returns
    ///
    /// A single `GrantDenialPair` representing the resolved decision.
    fn resolve(&self, results: Vec<GrantDenialPair>) -> GrantDenialPair;
}

/// Most restrictive resolver: intersects grants, unions denials.
///
/// This is the safest default for security-critical systems.
///
/// # Examples
///
/// ```
/// use acls_rs::policy::{ConflictResolver, MeetResolver};
/// use acls_rs::permission::{AtomicPermission, PermissionSet, GrantDenialPair};
///
/// let resolver = MeetResolver;
///
/// let result1 = GrantDenialPair::new(
///     PermissionSet::from([
///         AtomicPermission::new("file", "read"),
///         AtomicPermission::new("file", "write"),
///     ]),
///     PermissionSet::new(),
/// );
///
/// let result2 = GrantDenialPair::new(
///     PermissionSet::from([
///         AtomicPermission::new("file", "read"),
///     ]),
///     PermissionSet::from([
///         AtomicPermission::new("file", "delete"),
///     ]),
/// );
///
/// let resolved = resolver.resolve(vec![result1, result2]);
/// // Most restrictive: only file:read granted, file:delete denied
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MeetResolver;

impl ConflictResolver for MeetResolver {
    fn resolve(&self, results: Vec<GrantDenialPair>) -> GrantDenialPair {
        if results.is_empty() {
            return GrantDenialPair::empty();
        }

        results
            .into_iter()
            .reduce(|a, b| a.meet(b))
            .unwrap_or_else(GrantDenialPair::empty)
    }
}

/// Least restrictive resolver: unions grants, intersects denials.
///
/// This makes it easier for users to gain access but may be less secure.
///
/// # Examples
///
/// ```
/// use acls_rs::policy::{ConflictResolver, JoinResolver};
/// use acls_rs::permission::{AtomicPermission, PermissionSet, GrantDenialPair};
///
/// let resolver = JoinResolver;
///
/// let result1 = GrantDenialPair::new(
///     PermissionSet::from([
///         AtomicPermission::new("file", "read"),
///     ]),
///     PermissionSet::new(),
/// );
///
/// let result2 = GrantDenialPair::new(
///     PermissionSet::from([
///         AtomicPermission::new("file", "write"),
///     ]),
///     PermissionSet::new(),
/// );
///
/// let resolved = resolver.resolve(vec![result1, result2]);
/// // Least restrictive: both file:read and file:write granted
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct JoinResolver;

impl ConflictResolver for JoinResolver {
    fn resolve(&self, results: Vec<GrantDenialPair>) -> GrantDenialPair {
        if results.is_empty() {
            return GrantDenialPair::empty();
        }

        results
            .into_iter()
            .reduce(|a, b| a.join(b))
            .unwrap_or_else(GrantDenialPair::empty)
    }
}

/// Priority-based resolver: uses rule priorities to determine precedence.
///
/// Higher priority rules win. On ties, uses meet (most restrictive).
///
/// # Examples
///
/// ```
/// use acls_rs::policy::{ConflictResolver, PriorityResolver};
/// use acls_rs::permission::{AtomicPermission, PermissionSet, GrantDenialPair};
///
/// let mut resolver = PriorityResolver::new();
/// resolver.set_priority(0, 10); // Rule 0 has priority 10
/// resolver.set_priority(1, 5);  // Rule 1 has priority 5
///
/// let result1 = GrantDenialPair::new(
///     PermissionSet::from([AtomicPermission::new("file", "read")]),
///     PermissionSet::new(),
/// );
///
/// let result2 = GrantDenialPair::new(
///     PermissionSet::from([AtomicPermission::new("file", "write")]),
///     PermissionSet::new(),
/// );
///
/// // Rule 0 wins due to higher priority
/// let resolved = resolver.resolve_with_ids(vec![(0, result1), (1, result2)]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct PriorityResolver {
    priorities: std::collections::HashMap<usize, u32>,
}

impl PriorityResolver {
    /// Create a new priority resolver.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the priority for a rule ID.
    ///
    /// Higher numbers indicate higher priority.
    pub fn set_priority(&mut self, rule_id: usize, priority: u32) {
        self.priorities.insert(rule_id, priority);
    }

    /// Resolve results with associated rule IDs.
    ///
    /// Rule IDs are used to look up priorities.
    pub fn resolve_with_ids(&self, results: Vec<(usize, GrantDenialPair)>) -> GrantDenialPair {
        if results.is_empty() {
            return GrantDenialPair::empty();
        }

        // Find maximum priority
        let max_priority = results
            .iter()
            .map(|(id, _)| self.priorities.get(id).copied().unwrap_or(0))
            .max()
            .unwrap_or(0);

        // Filter to highest priority rules
        let highest_priority_results: Vec<_> = results
            .into_iter()
            .filter(|(id, _)| self.priorities.get(id).copied().unwrap_or(0) == max_priority)
            .map(|(_, result)| result)
            .collect();

        // Use meet to resolve ties (most restrictive)
        MeetResolver.resolve(highest_priority_results)
    }
}

// PriorityResolver intentionally does NOT implement ConflictResolver.
// Use resolve_with_ids() which requires rule IDs for priority lookup.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission::AtomicPermission;

    #[test]
    fn test_meet_resolver() {
        let resolver = MeetResolver;

        let r1 = GrantDenialPair::new(
            PermissionSet::from([
                AtomicPermission::new("file", "read"),
                AtomicPermission::new("file", "write"),
            ]),
            PermissionSet::new(),
        );

        let r2 = GrantDenialPair::new(
            PermissionSet::from([AtomicPermission::new("file", "read")]),
            PermissionSet::from([AtomicPermission::new("file", "delete")]),
        );

        let result = resolver.resolve(vec![r1, r2]);
        let effective = result.effective_permissions();

        // Most restrictive: only read, delete is denied
        assert_eq!(effective.len(), 1);
        assert!(effective.contains(&AtomicPermission::new("file", "read")));
    }

    #[test]
    fn test_join_resolver() {
        let resolver = JoinResolver;

        let r1 = GrantDenialPair::new(
            PermissionSet::from([AtomicPermission::new("file", "read")]),
            PermissionSet::new(),
        );

        let r2 = GrantDenialPair::new(
            PermissionSet::from([AtomicPermission::new("file", "write")]),
            PermissionSet::new(),
        );

        let result = resolver.resolve(vec![r1, r2]);
        let effective = result.effective_permissions();

        // Least restrictive: both granted
        assert_eq!(effective.len(), 2);
    }

    #[test]
    fn test_priority_resolver() {
        let mut resolver = PriorityResolver::new();
        resolver.set_priority(0, 10);
        resolver.set_priority(1, 5);

        let r1 = GrantDenialPair::new(
            PermissionSet::from([AtomicPermission::new("file", "read")]),
            PermissionSet::new(),
        );

        let r2 = GrantDenialPair::new(
            PermissionSet::from([AtomicPermission::new("file", "write")]),
            PermissionSet::new(),
        );

        let result = resolver.resolve_with_ids(vec![(0, r1), (1, r2)]);
        let effective = result.effective_permissions();

        // Rule 0 wins
        assert_eq!(effective.len(), 1);
        assert!(effective.contains(&AtomicPermission::new("file", "read")));
    }
}
