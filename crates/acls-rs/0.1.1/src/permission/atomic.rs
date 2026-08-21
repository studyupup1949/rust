//! Atomic permissions: indivisible permission units.

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An atomic permission: the fundamental unit of access control.
///
/// An atomic permission consists of a namespace (resource type) and an action.
/// For example: `file:read`, `user:delete`, `admin:*`.
///
/// # Examples
///
/// ```
/// use acls_rs::permission::AtomicPermission;
///
/// let perm = AtomicPermission::new("file", "read");
/// assert_eq!(perm.namespace(), "file");
/// assert_eq!(perm.action(), "read");
/// assert_eq!(perm.to_string(), "file:read");
/// ```
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AtomicPermission {
    namespace: String,
    action: String,
}

impl AtomicPermission {
    /// Create a new atomic permission.
    ///
    /// # Arguments
    ///
    /// - `namespace`: The resource type or domain (e.g., "file", "user", "admin")
    /// - `action`: The operation (e.g., "read", "write", "delete", "*")
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::AtomicPermission;
    ///
    /// let read = AtomicPermission::new("file", "read");
    /// let write = AtomicPermission::new("file", "write");
    /// let wildcard = AtomicPermission::new("admin", "*");
    /// ```
    pub fn new(namespace: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            action: action.into(),
        }
    }

    /// Get the namespace (resource type).
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::AtomicPermission;
    ///
    /// let perm = AtomicPermission::new("file", "read");
    /// assert_eq!(perm.namespace(), "file");
    /// ```
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Get the action.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::AtomicPermission;
    ///
    /// let perm = AtomicPermission::new("file", "read");
    /// assert_eq!(perm.action(), "read");
    /// ```
    pub fn action(&self) -> &str {
        &self.action
    }

    /// Create a builder for constructing atomic permissions.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::AtomicPermission;
    ///
    /// let perm = AtomicPermission::builder()
    ///     .namespace("file")
    ///     .action("read")
    ///     .build()
    ///     .expect("namespace and action are set");
    /// ```
    pub fn builder() -> AtomicPermissionBuilder {
        AtomicPermissionBuilder::default()
    }
}

/// Builder for constructing atomic permissions.
#[derive(Default)]
pub struct AtomicPermissionBuilder {
    namespace: Option<String>,
    action: Option<String>,
}

impl AtomicPermissionBuilder {
    /// Set the namespace.
    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Set the action.
    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Build the atomic permission.
    ///
    /// # Errors
    ///
    /// Returns an error if namespace or action is not set.
    ///
    /// # Note
    ///
    /// Consider using `AtomicPermission::new(namespace, action)` directly
    /// instead of the builder for simple cases.
    pub fn build(self) -> Result<AtomicPermission, String> {
        let namespace = self.namespace.ok_or("namespace must be set")?;
        let action = self.action.ok_or("action must be set")?;

        Ok(AtomicPermission { namespace, action })
    }
}

// Implement Ord for use in BTreeSet
impl Ord for AtomicPermission {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.namespace, &self.action).cmp(&(&other.namespace, &other.action))
    }
}

impl PartialOrd for AtomicPermission {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Implement Display for human-readable output
impl fmt::Display for AtomicPermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.action)
    }
}

// Implement FromStr for parsing from strings
impl FromStr for AtomicPermission {
    type Err = ParsePermissionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(ParsePermissionError::InvalidFormat(s.to_string()));
        }

        Ok(AtomicPermission::new(parts[0], parts[1]))
    }
}

/// Error type for parsing atomic permissions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsePermissionError {
    /// The input string was not in the expected format (namespace:action).
    InvalidFormat(String),
}

impl fmt::Display for ParsePermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsePermissionError::InvalidFormat(s) => {
                write!(
                    f,
                    "Invalid permission format: '{}' (expected 'namespace:action')",
                    s
                )
            }
        }
    }
}

impl std::error::Error for ParsePermissionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let perm = AtomicPermission::new("file", "read");
        assert_eq!(perm.namespace(), "file");
        assert_eq!(perm.action(), "read");
    }

    #[test]
    fn test_builder() {
        let perm = AtomicPermission::builder()
            .namespace("user")
            .action("delete")
            .build()
            .expect("both fields set");

        assert_eq!(perm.namespace(), "user");
        assert_eq!(perm.action(), "delete");
    }

    #[test]
    fn test_builder_missing_fields() {
        // Missing namespace
        let result = AtomicPermission::builder().action("delete").build();
        assert!(result.is_err());

        // Missing action
        let result = AtomicPermission::builder().namespace("user").build();
        assert!(result.is_err());

        // Missing both
        let result = AtomicPermission::builder().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_display() {
        let perm = AtomicPermission::new("file", "read");
        assert_eq!(perm.to_string(), "file:read");
    }

    #[test]
    fn test_from_str() {
        let perm: AtomicPermission = "file:read".parse().unwrap();
        assert_eq!(perm.namespace(), "file");
        assert_eq!(perm.action(), "read");
    }

    #[test]
    fn test_from_str_invalid() {
        let result = "invalid".parse::<AtomicPermission>();
        assert!(result.is_err());
    }

    #[test]
    fn test_ord() {
        let perm1 = AtomicPermission::new("a", "read");
        let perm2 = AtomicPermission::new("b", "read");
        let perm3 = AtomicPermission::new("a", "write");

        assert!(perm1 < perm2);
        assert!(perm1 < perm3);
    }

    #[test]
    fn test_eq() {
        let perm1 = AtomicPermission::new("file", "read");
        let perm2 = AtomicPermission::new("file", "read");
        let perm3 = AtomicPermission::new("file", "write");

        assert_eq!(perm1, perm2);
        assert_ne!(perm1, perm3);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        let perm = AtomicPermission::new("file", "read");
        let json = serde_json::to_string(&perm).unwrap();
        let deserialized: AtomicPermission = serde_json::from_str(&json).unwrap();

        assert_eq!(perm, deserialized);
    }
}
