//! Shared policy error types.
//!
//! This module provides common error types used across policy implementations
//! (RBAC, HBAC, ABAC) to avoid duplication.

use thiserror::Error;

/// Errors that can occur during policy operations.
///
/// This error type is shared across RBAC, HBAC, and ABAC policy implementations
/// to provide consistent error handling.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PolicyError {
    /// Too many rules in policy (DoS protection limit exceeded).
    ///
    /// Policies enforce a maximum rule count to prevent denial-of-service attacks
    /// through excessive memory consumption. When this limit is exceeded, this error
    /// is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use acls_rs::policy::PolicyError;
    /// let error = PolicyError::TooManyRules {
    ///     requested: 2_000_000,
    ///     maximum: 1_000_000,
    /// };
    /// assert_eq!(
    ///     error.to_string(),
    ///     "too many rules: requested 2000000, maximum 1000000"
    /// );
    /// ```
    #[error("too many rules: requested {requested}, maximum {maximum}")]
    TooManyRules {
        /// Number of rules that were requested to be loaded
        requested: usize,
        /// Maximum allowed rules
        maximum: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_error_display() {
        let error = PolicyError::TooManyRules {
            requested: 1500,
            maximum: 1000,
        };
        assert_eq!(
            error.to_string(),
            "too many rules: requested 1500, maximum 1000"
        );
    }

    #[test]
    fn test_policy_error_is_error() {
        let error = PolicyError::TooManyRules {
            requested: 1500,
            maximum: 1000,
        };
        // Should implement std::error::Error
        let _: &dyn std::error::Error = &error;
    }
}
