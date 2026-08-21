//! ABAC and RBAC policy composition.
//!
//! This module provides [`ComposedPolicy`] which combines ABAC and RBAC policies
//! with configurable composition modes.
//!
//! # Composition Modes
//!
//! - **And**: Both ABAC and RBAC must allow (strictest)
//! - **Or**: Either ABAC or RBAC allows (most permissive)
//! - **AbacFirst**: Use ABAC if rules match, fallback to RBAC
//! - **RbacFirst**: Use RBAC if permissions exist, fallback to ABAC
//!
//! # Examples
//!
//! ```rust
//! use abac_rs::{AbacPolicy, AbacRequest, AbacRule, AttributeType};
//! use abac_rs::{ComposedPolicy, CompositionMode};
//! use acls_rs::policy::RbacPolicy;
//! use acls_rs::permission::AtomicPermission;
//! use acls_rs::Subject;
//!
//! let mut abac = AbacPolicy::new();
//! let rule = AbacRule::builder("allow_all")
//!     .dimension_all("user")
//!     .dimension_all("resource")
//!     .dimension_all("action")
//!     .enabled(true)
//!     .build();
//! abac.add_rule(rule).unwrap();
//!
//! let rbac = RbacPolicy::new();
//!
//! // Both must allow
//! let mut policy = ComposedPolicy::new(abac, rbac, CompositionMode::And);
//!
//! let mut user = Subject::new("alice");
//! user.grant(AtomicPermission::new("server", "read"));
//!
//! let mut request = AbacRequest::new();
//! request.add_attribute("user", AttributeType::String("alice".into()), vec![]);
//! request.add_attribute("resource", AttributeType::String("server".into()), vec![]);
//! request.add_attribute("action", AttributeType::String("read".into()), vec![]);
//!
//! // Requires both ABAC and RBAC approval
//! let result = policy.evaluate(&mut request, &user);
//! ```

use crate::cache::RequestKey;
use crate::policy::CacheLock;
use crate::{AbacPolicyCore, AbacRequest, Decision};
use acls_rs::permission::{AtomicPermission, PermissionSet};
use acls_rs::policy::RbacPolicy;
use acls_rs::Subject;
use lru::LruCache;
use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::sync::Mutex;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// PermissionCacheLock is now provided by acls_rs::sync::SyncStrategy
// Re-export for backward compatibility
pub use acls_rs::sync::SyncStrategy as PermissionCacheLock;

/// Composition mode for combining ABAC and RBAC policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CompositionMode {
    /// ABAC and RBAC must both allow (intersection)
    And,
    /// ABAC or RBAC must allow (union)
    Or,
    /// ABAC takes precedence, RBAC is fallback
    AbacFirst,
    /// RBAC takes precedence, ABAC is fallback
    RbacFirst,
}

/// A composed policy combining ABAC and RBAC.
///
/// This allows integrating attribute-based access control with
/// role-based access control from acls-rs. The composition mode determines
/// how the two policy types interact.
///
/// # Example
///
/// ```
/// use abac_rs::{AbacPolicy, AbacRequest, AbacRule, AttributeType};
/// use abac_rs::{ComposedPolicy, CompositionMode};
/// use acls_rs::policy::RbacPolicy;
/// use acls_rs::Subject;
///
/// let mut abac = AbacPolicy::new();
/// let rule = AbacRule::builder("allow_read")
///     .dimension_all("action")
///     .enabled(true)
///     .build();
/// abac.add_rule(rule).unwrap();
///
/// let rbac = RbacPolicy::new();
///
/// let mut policy = ComposedPolicy::new(abac, rbac, CompositionMode::And);
///
/// let user = Subject::new("alice");
/// let mut request = AbacRequest::new();
/// request.add_attribute("action", AttributeType::String("read".into()), vec![]);
///
/// let result = policy.evaluate(&mut request, &user);
/// ```
pub struct ComposedPolicyCore<
    A: CacheLock<LruCache<RequestKey, Decision>>,
    P: PermissionCacheLock<LruCache<Vec<String>, PermissionSet>>,
> {
    abac_policy: AbacPolicyCore<A>,
    rbac_policy: RbacPolicy,
    /// Composition mode determining how ABAC and RBAC results are combined
    pub mode: CompositionMode,
    /// LRU cache for resolved RBAC permissions keyed by sorted role set
    permission_cache: P,
}

impl<
        A: CacheLock<LruCache<RequestKey, Decision>>,
        P: PermissionCacheLock<LruCache<Vec<String>, PermissionSet>>,
    > std::fmt::Debug for ComposedPolicyCore<A, P>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComposedPolicy")
            .field("abac_policy", &self.abac_policy)
            .field("rbac_policy", &self.rbac_policy)
            .field("mode", &self.mode)
            .field("permission_cache", &"<LRU cache>")
            .finish()
    }
}

/// Type alias for thread-safe composed policy (uses Mutex for both caches).
pub type ComposedPolicy = ComposedPolicyCore<
    Mutex<LruCache<RequestKey, Decision>>,
    Mutex<LruCache<Vec<String>, PermissionSet>>,
>;

/// Type alias for single-threaded composed policy (uses RefCell for both caches).
pub type ComposedPolicyLocal = ComposedPolicyCore<
    RefCell<LruCache<RequestKey, Decision>>,
    RefCell<LruCache<Vec<String>, PermissionSet>>,
>;

impl<
        A: CacheLock<LruCache<RequestKey, Decision>>,
        P: PermissionCacheLock<LruCache<Vec<String>, PermissionSet>>,
    > ComposedPolicyCore<A, P>
{
    /// Default size for permission cache (100 unique role sets).
    const DEFAULT_PERMISSION_CACHE_SIZE: usize = 100;

    /// Creates a new composed policy.
    pub fn new(
        abac_policy: AbacPolicyCore<A>,
        rbac_policy: RbacPolicy,
        mode: CompositionMode,
    ) -> Self {
        Self::with_cache_size(
            abac_policy,
            rbac_policy,
            mode,
            Self::DEFAULT_PERMISSION_CACHE_SIZE,
        )
    }

    /// Creates a new composed policy with a custom permission cache size.
    pub fn with_cache_size(
        abac_policy: AbacPolicyCore<A>,
        rbac_policy: RbacPolicy,
        mode: CompositionMode,
        cache_size: usize,
    ) -> Self {
        let cache = LruCache::new(NonZeroUsize::new(cache_size).unwrap());
        Self {
            abac_policy,
            rbac_policy,
            mode,
            permission_cache: P::new(cache),
        }
    }

    /// Returns a reference to the ABAC policy.
    pub fn abac_policy(&self) -> &AbacPolicyCore<A> {
        &self.abac_policy
    }

    /// Returns a mutable reference to the ABAC policy.
    pub fn abac_policy_mut(&mut self) -> &mut AbacPolicyCore<A> {
        &mut self.abac_policy
    }

    /// Returns a reference to the RBAC policy.
    pub fn rbac_policy(&self) -> &RbacPolicy {
        &self.rbac_policy
    }

    /// Returns a mutable reference to the RBAC policy.
    ///
    /// Note: Modifying the RBAC policy invalidates the permission cache.
    pub fn rbac_policy_mut(&mut self) -> &mut RbacPolicy {
        self.invalidate_permission_cache();
        &mut self.rbac_policy
    }

    /// Invalidates the RBAC permission cache.
    ///
    /// Call this when the RBAC policy or role hierarchy changes.
    pub fn invalidate_permission_cache(&self) {
        self.permission_cache.with(|cache| cache.clear());
    }

    /// Evaluates access based on the composition mode.
    ///
    /// The request is checked against both ABAC and RBAC policies,
    /// and the results are combined according to the composition mode.
    ///
    /// # RBAC Permission Mapping
    ///
    /// For RBAC evaluation, the request must have "resource" and "action"
    /// attributes. These are extracted to create an `AtomicPermission` for
    /// RBAC checking:
    /// - resource → permission object
    /// - action → permission action
    ///
    /// If these attributes are missing, RBAC evaluation is skipped.
    pub fn evaluate(&mut self, request: &mut AbacRequest, user: &Subject) -> Decision {
        // Evaluate ABAC
        let abac_result = self.abac_policy.evaluate(request);
        let abac_allowed = abac_result.is_allowed();

        // Extract RBAC permission from ABAC request
        // We need "resource" and "action" dimensions for RBAC
        let rbac_allowed = if let (Some((resource, _)), Some((action, _))) = (
            request.get_attribute("resource"),
            request.get_attribute("action"),
        ) {
            // Convert to strings for RBAC permission
            let resource_str = match resource {
                crate::AttributeType::String(s) => s.as_str(),
                _ => return abac_result, // Can't map to RBAC, return ABAC result only
            };

            let action_str = match action {
                crate::AttributeType::String(s) => s.as_str(),
                _ => return abac_result, // Can't map to RBAC, return ABAC result only
            };

            let required_perm = AtomicPermission::new(resource_str, action_str);

            // Check RBAC permission using role hierarchy resolution with caching
            if user.roles.is_empty() {
                user.has_permission(&required_perm)
            } else {
                // Create sorted cache key from roles
                let mut role_key: Vec<String> = user.roles.to_vec();
                role_key.sort();

                // Check cache first
                let effective_perms = self
                    .permission_cache
                    .with(|cache| cache.get(&role_key).cloned());

                let effective_perms = if let Some(perms) = effective_perms {
                    perms
                } else {
                    // Cache miss - resolve permissions
                    match self.rbac_policy.resolve_permissions(&user.roles) {
                        Ok(resolved) => {
                            let effective = resolved.effective_permissions();
                            self.permission_cache.with(|cache| {
                                cache.put(role_key.clone(), effective.clone());
                            });
                            effective
                        }
                        Err(e) => {
                            log::warn!(
                                "RBAC role resolution failed ({:?}), falling back to direct permissions",
                                e
                            );
                            let has_perm = user.has_permission(&required_perm);
                            return if has_perm {
                                Decision::Allow
                            } else {
                                Decision::Deny
                            };
                        }
                    }
                };

                effective_perms.contains(&required_perm)
            }
        } else {
            // No resource/action attributes, can't evaluate RBAC
            return abac_result;
        };

        // Combine results based on mode
        let final_allowed = match self.mode {
            CompositionMode::And => abac_allowed && rbac_allowed,
            CompositionMode::Or => abac_allowed || rbac_allowed,
            CompositionMode::AbacFirst => {
                // If ABAC has rules for this request (not just default deny), use ABAC
                // Otherwise use RBAC
                // For simplicity, we check if ABAC allowed - if yes, definitely use ABAC
                // If ABAC denied, we still use ABAC result (could be refined)
                if abac_allowed || self.abac_policy.rule_count() > 0 {
                    abac_allowed
                } else {
                    rbac_allowed
                }
            }
            CompositionMode::RbacFirst => {
                // If user has any permissions, use RBAC result
                // Otherwise use ABAC
                if !user.effective_permissions().is_empty() {
                    rbac_allowed
                } else {
                    abac_allowed
                }
            }
        };

        if final_allowed {
            Decision::Allow
        } else {
            Decision::Deny
        }
    }

    /// Checks access with basic allow/deny result.
    pub fn check_access(&mut self, request: &mut AbacRequest, user: &Subject) -> bool {
        self.evaluate(request, user).is_allowed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AbacPolicy, AbacRequest, AbacRule, AttributeType, AttributeValue};
    use ahash::AHashSet as HashSet;

    #[test]
    fn test_composition_and_both_allow() {
        let mut abac = AbacPolicy::new();
        let mut rule = AbacRule::new("allow_all");
        rule.add_dimension("user", AttributeValue::All);
        rule.add_dimension("resource", AttributeValue::All);
        rule.add_dimension("action", AttributeValue::All);
        rule.enable();
        abac.add_rule(rule).unwrap();

        let rbac = RbacPolicy::new();

        let mut user = Subject::new("alice");
        user.grant(AtomicPermission::new("server", "read"));

        let mut policy = ComposedPolicy::new(abac, rbac, CompositionMode::And);

        let mut request = AbacRequest::new();
        request
            .add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();
        request
            .add_attribute("resource", AttributeType::String("server".into()), vec![])
            .unwrap();
        request
            .add_attribute("action", AttributeType::String("read".into()), vec![])
            .unwrap();

        assert!(policy.check_access(&mut request, &user));
    }

    #[test]
    fn test_composition_and_abac_deny() {
        let abac = AbacPolicy::new(); // No rules, denies all

        let rbac = RbacPolicy::new();

        let mut user = Subject::new("alice");
        user.grant(AtomicPermission::new("server", "read"));

        let mut policy = ComposedPolicy::new(abac, rbac, CompositionMode::And);

        let mut request = AbacRequest::new();
        request
            .add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();
        request
            .add_attribute("resource", AttributeType::String("server".into()), vec![])
            .unwrap();
        request
            .add_attribute("action", AttributeType::String("read".into()), vec![])
            .unwrap();

        assert!(!policy.check_access(&mut request, &user));
    }

    #[test]
    fn test_composition_or_one_allows() {
        let mut abac = AbacPolicy::new();
        let mut rule = AbacRule::new("allow_all");
        rule.add_dimension("user", AttributeValue::All);
        rule.add_dimension("resource", AttributeValue::All);
        rule.add_dimension("action", AttributeValue::All);
        rule.enable();
        abac.add_rule(rule).unwrap();

        let rbac = RbacPolicy::new(); // RBAC denies (no permissions)

        let user = Subject::new("alice"); // No permissions

        let mut policy = ComposedPolicy::new(abac, rbac, CompositionMode::Or);

        let mut request = AbacRequest::new();
        request
            .add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();
        request
            .add_attribute("resource", AttributeType::String("server".into()), vec![])
            .unwrap();
        request
            .add_attribute("action", AttributeType::String("read".into()), vec![])
            .unwrap();

        assert!(policy.check_access(&mut request, &user));
    }

    #[test]
    fn test_composition_abac_first() {
        let mut abac = AbacPolicy::new();
        let mut rule = AbacRule::new("allow_read");
        let mut action_set = HashSet::new();
        action_set.insert(AttributeType::String("read".into()));
        rule.add_dimension("action", AttributeValue::Specific(action_set));
        rule.enable();
        abac.add_rule(rule).unwrap();

        let rbac = RbacPolicy::new();

        let user = Subject::new("alice");

        let mut policy = ComposedPolicy::new(abac, rbac, CompositionMode::AbacFirst);

        let mut request = AbacRequest::new();
        request
            .add_attribute("action", AttributeType::String("read".into()), vec![])
            .unwrap();
        request
            .add_attribute("resource", AttributeType::String("server".into()), vec![])
            .unwrap();

        // ABAC allows, RBAC denies, but ABAC takes precedence
        assert!(policy.check_access(&mut request, &user));
    }
}
