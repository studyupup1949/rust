//! Attribute-Based Access Control (ABAC) implementation.

use crate::permission::{AtomicPermission, GrantDenialPair, PermissionSet};
use crate::policy::conflict::{ConflictResolver, MeetResolver};
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Attribute context for ABAC evaluation.
pub type AttributeContext = HashMap<String, String>;

/// A permission with attribute predicates.
///
/// The permission is granted only if all attributes match the context.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AttributePermission {
    pub permission: AtomicPermission,
    pub attributes: AttributeContext,
}

impl AttributePermission {
    /// Create a new attribute permission.
    pub fn new(permission: AtomicPermission, attributes: AttributeContext) -> Self {
        Self {
            permission,
            attributes,
        }
    }

    /// Check if this permission matches the given context.
    ///
    /// Returns true if all required attributes are present in the context
    /// with matching values.
    pub fn matches_context(&self, context: &AttributeContext) -> bool {
        self.attributes
            .iter()
            .all(|(key, value)| context.get(key) == Some(value))
    }
}

/// A rule defining grants and denials based on attributes.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AttributeRule {
    pub grants: Vec<AttributePermission>,
    pub denials: Vec<AttributePermission>,
}

impl AttributeRule {
    /// Create a new empty attribute rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a grant with attributes.
    pub fn grant(mut self, perm: AtomicPermission, attrs: AttributeContext) -> Self {
        self.grants.push(AttributePermission::new(perm, attrs));
        self
    }

    /// Add a denial with attributes.
    pub fn deny(mut self, perm: AtomicPermission, attrs: AttributeContext) -> Self {
        self.denials.push(AttributePermission::new(perm, attrs));
        self
    }

    /// Evaluate this rule against a context.
    ///
    /// Returns the grants and denials that match the context.
    pub fn evaluate(&self, context: &AttributeContext) -> GrantDenialPair {
        let grants: PermissionSet = self
            .grants
            .iter()
            .filter(|ap| ap.matches_context(context))
            .map(|ap| ap.permission.clone())
            .collect();

        let denials: PermissionSet = self
            .denials
            .iter()
            .filter(|ap| ap.matches_context(context))
            .map(|ap| ap.permission.clone())
            .collect();

        GrantDenialPair::new(grants, denials)
    }
}

/// ABAC policy with pluggable conflict resolution.
///
/// # Examples
///
/// ```
/// use acls_rs::policy::{AbacPolicy, AttributeRule};
/// use acls_rs::permission::AtomicPermission;
/// use std::collections::HashMap;
///
/// let mut policy = AbacPolicy::new();
///
/// // Add rule: engineering can read code
/// let rule = AttributeRule::new()
///     .grant(
///         AtomicPermission::new("code", "read"),
///         [("department".to_string(), "engineering".to_string())].into()
///     );
///
/// policy.add_rule(rule);
///
/// // Evaluate with context
/// let mut context = HashMap::new();
/// context.insert("department".to_string(), "engineering".to_string());
///
/// let result = policy.evaluate(&context);
/// let effective = result.effective_permissions();
///
/// assert!(effective.contains(&AtomicPermission::new("code", "read")));
/// ```
#[derive(Debug)]
pub struct AbacPolicy<R: ConflictResolver = MeetResolver> {
    rules: Vec<AttributeRule>,
    resolver: R,
}

impl AbacPolicy<MeetResolver> {
    /// Create a new ABAC policy with the default meet resolver.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            resolver: MeetResolver,
        }
    }
}

impl<R: ConflictResolver> AbacPolicy<R> {
    /// Create a new ABAC policy with a custom resolver.
    pub fn with_resolver(resolver: R) -> Self {
        Self {
            rules: Vec::new(),
            resolver,
        }
    }

    /// Add a rule to the policy.
    pub fn add_rule(&mut self, rule: AttributeRule) {
        self.rules.push(rule);
    }

    /// Evaluate the policy against an attribute context.
    ///
    /// All matching rules are combined using the conflict resolver.
    pub fn evaluate(&self, context: &AttributeContext) -> GrantDenialPair {
        let results: Vec<_> = self
            .rules
            .iter()
            .map(|rule| rule.evaluate(context))
            .filter(|gd| !gd.grants.is_empty() || !gd.denials.is_empty())
            .collect();

        if results.is_empty() {
            GrantDenialPair::empty()
        } else {
            self.resolver.resolve(results)
        }
    }

    /// Get the number of rules in the policy.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for AbacPolicy<MeetResolver> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_permission_matches() {
        let ap = AttributePermission::new(
            AtomicPermission::new("resource", "read"),
            [
                ("department".to_string(), "engineering".to_string()),
                ("level".to_string(), "senior".to_string()),
            ]
            .into(),
        );

        let mut context1 = HashMap::new();
        context1.insert("department".to_string(), "engineering".to_string());
        context1.insert("level".to_string(), "senior".to_string());
        assert!(ap.matches_context(&context1));

        let mut context2 = HashMap::new();
        context2.insert("department".to_string(), "engineering".to_string());
        assert!(!ap.matches_context(&context2)); // Missing level

        let mut context3 = HashMap::new();
        context3.insert("department".to_string(), "sales".to_string());
        context3.insert("level".to_string(), "senior".to_string());
        assert!(!ap.matches_context(&context3)); // Wrong department
    }

    #[test]
    fn test_attribute_rule_evaluation() {
        let rule = AttributeRule::new()
            .grant(
                AtomicPermission::new("file", "read"),
                [("department".to_string(), "engineering".to_string())].into(),
            )
            .deny(
                AtomicPermission::new("file", "delete"),
                [("level".to_string(), "junior".to_string())].into(),
            );

        let mut context = HashMap::new();
        context.insert("department".to_string(), "engineering".to_string());
        context.insert("level".to_string(), "junior".to_string());

        let result = rule.evaluate(&context);
        let effective = result.effective_permissions();

        assert!(effective.contains(&AtomicPermission::new("file", "read")));
        assert!(!effective.contains(&AtomicPermission::new("file", "delete"))); // Denied
    }

    #[test]
    fn test_abac_policy() {
        let mut policy = AbacPolicy::new();

        policy.add_rule(AttributeRule::new().grant(
            AtomicPermission::new("resource", "read"),
            [("role".to_string(), "viewer".to_string())].into(),
        ));

        policy.add_rule(AttributeRule::new().grant(
            AtomicPermission::new("resource", "write"),
            [("role".to_string(), "editor".to_string())].into(),
        ));

        let mut context = HashMap::new();
        context.insert("role".to_string(), "editor".to_string());

        let result = policy.evaluate(&context);
        let effective = result.effective_permissions();

        // Only write is granted (role=editor doesn't match role=viewer)
        assert_eq!(effective.len(), 1);
        assert!(effective.contains(&AtomicPermission::new("resource", "write")));
    }

    #[test]
    fn test_conflict_resolution() {
        use crate::policy::conflict::JoinResolver;

        let mut policy = AbacPolicy::with_resolver(JoinResolver);

        policy.add_rule(AttributeRule::new().grant(
            AtomicPermission::new("file", "read"),
            [("tag".to_string(), "public".to_string())].into(),
        ));

        policy.add_rule(AttributeRule::new().grant(
            AtomicPermission::new("file", "write"),
            [("tag".to_string(), "public".to_string())].into(),
        ));

        let mut context = HashMap::new();
        context.insert("tag".to_string(), "public".to_string());

        let result = policy.evaluate(&context);
        let effective = result.effective_permissions();

        // JoinResolver unions grants
        assert_eq!(effective.len(), 2);
    }
}
