//! Role-Based Access Control (RBAC) implementation.

use crate::algebra::JoinSemilattice;
#[cfg(test)]
use crate::permission::AtomicPermission;
use crate::permission::{GrantDenialPair, PermissionSet};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Maximum role hierarchy depth to prevent excessive recursion
const MAX_ROLE_DEPTH: usize = 100;

/// A role with permissions and optional parent roles for inheritance.
///
/// # Examples
///
/// ```
/// use acls_rs::policy::Role;
/// use acls_rs::permission::{AtomicPermission, PermissionSet, GrantDenialPair};
///
/// let viewer = Role::new(
///     "viewer",
///     GrantDenialPair::new(
///         PermissionSet::from([AtomicPermission::new("resource", "read")]),
///         PermissionSet::new(),
///     )
/// );
///
/// let editor = Role::new(
///     "editor",
///     GrantDenialPair::new(
///         PermissionSet::from([AtomicPermission::new("resource", "write")]),
///         PermissionSet::new(),
///     )
/// ).with_parent("viewer");
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Role {
    pub name: String,
    pub permissions: GrantDenialPair,
    pub parent_roles: Vec<String>,
}

impl Role {
    /// Create a new role with the given name and permissions.
    pub fn new(name: impl Into<String>, permissions: GrantDenialPair) -> Self {
        Self {
            name: name.into(),
            permissions,
            parent_roles: Vec::new(),
        }
    }

    /// Add a parent role for inheritance.
    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent_roles.push(parent.into());
        self
    }

    /// Add multiple parent roles.
    pub fn with_parents(mut self, parents: Vec<String>) -> Self {
        self.parent_roles.extend(parents);
        self
    }
}

/// Error type for RBAC operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RbacError {
    /// Role not found in the registry.
    RoleNotFound(String),
    /// Cyclic dependency detected in role hierarchy.
    CyclicDependency { role: String, chain: Vec<String> },
    /// Role hierarchy exceeds maximum depth.
    MaxDepthExceeded { role: String, depth: usize },
}

impl std::fmt::Display for RbacError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RbacError::RoleNotFound(name) => write!(f, "Role not found: {}", name),
            RbacError::CyclicDependency { role, chain } => {
                write!(
                    f,
                    "Cyclic dependency detected for role '{}': {:?}",
                    role, chain
                )
            }
            RbacError::MaxDepthExceeded { role, depth } => {
                write!(
                    f,
                    "Role hierarchy depth {} exceeds maximum {} for role '{}'",
                    depth, MAX_ROLE_DEPTH, role
                )
            }
        }
    }
}

impl std::error::Error for RbacError {}

/// RBAC policy with role hierarchy and caching.
///
/// Supports:
/// - Role inheritance (child roles inherit parent permissions and denials)
/// - Cyclic dependency detection
/// - Transparent caching with automatic invalidation
///
/// # Examples
///
/// ```
/// use acls_rs::policy::{RbacPolicy, Role};
/// use acls_rs::permission::{AtomicPermission, PermissionSet, GrantDenialPair};
/// use acls_rs::Subject;
///
/// let mut policy = RbacPolicy::new();
///
/// // Define roles
/// policy.add_role(Role::new(
///     "viewer",
///     GrantDenialPair::new(
///         PermissionSet::from([AtomicPermission::new("resource", "read")]),
///         PermissionSet::new(),
///     )
/// ));
///
/// policy.add_role(Role::new(
///     "editor",
///     GrantDenialPair::new(
///         PermissionSet::from([AtomicPermission::new("resource", "write")]),
///         PermissionSet::new(),
///     )
/// ).with_parent("viewer"));
///
/// // Create subject with editor role
/// let user = Subject::builder()
///     .id("alice")
///     .role("editor")
///     .build()
///     .unwrap();
///
/// // Resolve permissions (includes inherited viewer permissions)
/// let perms = policy.resolve_permissions(&user.roles).unwrap();
/// let effective = perms.effective_permissions();
///
/// assert!(effective.contains(&AtomicPermission::new("resource", "read")));
/// assert!(effective.contains(&AtomicPermission::new("resource", "write")));
/// ```
#[derive(Debug)]
pub struct RbacPolicy {
    roles: HashMap<String, Role>,
    cache: RwLock<HashMap<Vec<String>, GrantDenialPair>>,
}

impl RbacPolicy {
    /// Create a new empty RBAC policy.
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Add a role to the policy.
    ///
    /// This invalidates the cache.
    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
        self.clear_cache();
    }

    /// Remove a role from the policy.
    pub fn remove_role(&mut self, name: &str) -> Option<Role> {
        let role = self.roles.remove(name);
        if role.is_some() {
            self.clear_cache();
        }
        role
    }

    /// Get a role by name.
    pub fn get_role(&self, name: &str) -> Option<&Role> {
        self.roles.get(name)
    }

    /// Clear the permission cache.
    fn clear_cache(&self) {
        match self.cache.write() {
            Ok(mut cache) => cache.clear(),
            Err(e) => {
                eprintln!("warning: RBAC cache lock was poisoned, recovering");
                e.into_inner().clear();
            }
        }
    }

    /// Resolve permissions for a set of roles.
    ///
    /// This flattens the role hierarchy, inheriting permissions from parent roles.
    /// Denials are also inherited.
    ///
    /// Results are cached transparently.
    pub fn resolve_permissions(&self, role_names: &[String]) -> Result<GrantDenialPair, RbacError> {
        // Try cache first (read lock)
        {
            let cache = match self.cache.read() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("warning: RBAC cache read lock was poisoned, recovering");
                    e.into_inner()
                }
            };
            if let Some(cached) = cache.get(role_names) {
                return Ok(cached.clone());
            }
        }

        // Cache miss - compute and cache
        let mut visited = HashSet::new();
        let mut result = GrantDenialPair::empty();

        for role_name in role_names {
            let role_perms =
                self.resolve_role_recursive(role_name, &mut visited, &mut Vec::new())?;
            result = result.join(role_perms); // Union all role permissions
        }

        // Cache the result (write lock)
        match self.cache.write() {
            Ok(mut cache) => {
                cache.insert(role_names.to_vec(), result.clone());
            }
            Err(e) => {
                eprintln!("warning: RBAC cache write lock was poisoned, recovering");
                e.into_inner().insert(role_names.to_vec(), result.clone());
            }
        }

        Ok(result)
    }

    /// Recursively resolve a single role's permissions.
    fn resolve_role_recursive(
        &self,
        role_name: &str,
        visited: &mut HashSet<String>,
        chain: &mut Vec<String>,
    ) -> Result<GrantDenialPair, RbacError> {
        // Check depth limit to prevent excessive recursion
        if chain.len() >= MAX_ROLE_DEPTH {
            return Err(RbacError::MaxDepthExceeded {
                role: role_name.to_string(),
                depth: chain.len(),
            });
        }

        // Check for cycles
        if visited.contains(role_name) {
            return Err(RbacError::CyclicDependency {
                role: role_name.to_string(),
                chain: chain.clone(),
            });
        }

        // Get the role
        let role = self
            .roles
            .get(role_name)
            .ok_or_else(|| RbacError::RoleNotFound(role_name.to_string()))?;

        visited.insert(role_name.to_string());
        chain.push(role_name.to_string());

        // Start with this role's permissions
        let mut result = role.permissions.clone();

        // Inherit from parents
        for parent_name in &role.parent_roles {
            let parent_perms = self.resolve_role_recursive(parent_name, visited, chain)?;
            // Join to combine permissions (union grants, intersect denials)
            result = result.join(parent_perms);
        }

        chain.pop();
        visited.remove(role_name);

        Ok(result)
    }

    /// Compute effective permissions for a set of roles.
    ///
    /// This is a convenience method that resolves permissions and computes
    /// grants - denials.
    pub fn effective_permissions(&self, role_names: &[String]) -> Result<PermissionSet, RbacError> {
        let gd = self.resolve_permissions(role_names)?;
        Ok(gd.effective_permissions())
    }
}

impl Default for RbacPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RbacPolicy {
    fn clone(&self) -> Self {
        Self {
            roles: self.roles.clone(),
            cache: RwLock::new(HashMap::new()), // Start with empty cache
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_hierarchy() {
        let mut policy = RbacPolicy::new();

        policy.add_role(Role::new(
            "viewer",
            GrantDenialPair::new(
                PermissionSet::from([AtomicPermission::new("resource", "read")]),
                PermissionSet::new(),
            ),
        ));

        policy.add_role(
            Role::new(
                "editor",
                GrantDenialPair::new(
                    PermissionSet::from([AtomicPermission::new("resource", "write")]),
                    PermissionSet::new(),
                ),
            )
            .with_parent("viewer"),
        );

        let perms = policy.resolve_permissions(&["editor".to_string()]).unwrap();
        let effective = perms.effective_permissions();

        assert!(effective.contains(&AtomicPermission::new("resource", "read")));
        assert!(effective.contains(&AtomicPermission::new("resource", "write")));
    }

    #[test]
    fn test_denial_inheritance() {
        let mut policy = RbacPolicy::new();

        policy.add_role(Role::new(
            "base",
            GrantDenialPair::new(
                PermissionSet::from([
                    AtomicPermission::new("file", "read"),
                    AtomicPermission::new("file", "write"),
                ]),
                PermissionSet::from([AtomicPermission::new("file", "delete")]),
            ),
        ));

        policy.add_role(
            Role::new(
                "child",
                GrantDenialPair::new(
                    PermissionSet::from([AtomicPermission::new("file", "execute")]),
                    PermissionSet::new(),
                ),
            )
            .with_parent("base"),
        );

        let perms = policy.resolve_permissions(&["child".to_string()]).unwrap();
        let effective = perms.effective_permissions();

        // Child inherits grants and denials
        assert!(effective.contains(&AtomicPermission::new("file", "read")));
        assert!(effective.contains(&AtomicPermission::new("file", "execute")));
        assert!(!effective.contains(&AtomicPermission::new("file", "delete"))); // Denied
    }

    #[test]
    fn test_cyclic_detection() {
        let mut policy = RbacPolicy::new();

        policy.add_role(Role::new("role_a", GrantDenialPair::empty()).with_parent("role_b"));

        policy.add_role(Role::new("role_b", GrantDenialPair::empty()).with_parent("role_a"));

        let result = policy.resolve_permissions(&["role_a".to_string()]);
        assert!(matches!(result, Err(RbacError::CyclicDependency { .. })));
    }

    #[test]
    fn test_depth_limit() {
        let mut policy = RbacPolicy::new();

        // Create a chain of 101 roles (exceeding MAX_ROLE_DEPTH=100)
        for i in 0..101 {
            let role_name = format!("role_{}", i);
            let parent_name = format!("role_{}", i + 1);

            policy
                .add_role(Role::new(role_name, GrantDenialPair::empty()).with_parent(parent_name));
        }

        // Add the final role (no parent)
        policy.add_role(Role::new("role_101", GrantDenialPair::empty()));

        let result = policy.resolve_permissions(&["role_0".to_string()]);
        assert!(matches!(result, Err(RbacError::MaxDepthExceeded { .. })));
    }

    #[test]
    fn test_caching() {
        let mut policy = RbacPolicy::new();

        policy.add_role(Role::new(
            "test",
            GrantDenialPair::new(
                PermissionSet::from([AtomicPermission::new("test", "read")]),
                PermissionSet::new(),
            ),
        ));

        // First call - cache miss
        let perms1 = policy.resolve_permissions(&["test".to_string()]).unwrap();

        // Second call - cache hit
        let perms2 = policy.resolve_permissions(&["test".to_string()]).unwrap();

        assert_eq!(perms1, perms2);
    }
}
