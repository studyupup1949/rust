//! Rule count limits for DoS protection.
//!
//! This module provides a trait for enforcing maximum rule count limits
//! across policy implementations (RBAC, HBAC, ABAC).

use super::PolicyError;

/// A policy that enforces maximum rule count limits for DoS protection.
///
/// Policies implementing this trait maintain a configurable maximum number
/// of rules to prevent denial-of-service attacks through excessive memory
/// consumption during policy loading.
///
/// # Limit Enforcement
///
/// - `max_limit() == 0` means no limit is enforced
/// - `max_limit() > 0` enforces the limit strictly
/// - Operations that would exceed the limit return `PolicyError::TooManyRules`
///
/// # Examples
///
/// ```
/// use acls_rs::policy::{RuleLimitedPolicy, PolicyError};
///
/// struct MyPolicy {
///     max_rules: usize,
///     current_count: usize,
/// }
///
/// impl RuleLimitedPolicy for MyPolicy {
///     fn max_limit(&self) -> usize {
///         self.max_rules
///     }
///
///     fn current_count(&self) -> usize {
///         self.current_count
///     }
/// }
///
/// let policy = MyPolicy { max_rules: 100, current_count: 50 };
///
/// // Check if we can add 30 more rules
/// assert!(policy.check_limit(30).is_ok());
///
/// // Check if we can add 60 more rules (would exceed 100)
/// assert!(policy.check_limit(60).is_err());
/// ```
pub trait RuleLimitedPolicy {
    /// Returns the maximum number of rules allowed.
    ///
    /// A value of 0 means no limit is enforced.
    fn max_limit(&self) -> usize;

    /// Returns the current number of rules in the policy.
    fn current_count(&self) -> usize;

    /// Checks if adding `additional` rules would exceed the maximum limit.
    ///
    /// # Arguments
    ///
    /// * `additional` - Number of rules to potentially add
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the operation is allowed
    /// - `Err(PolicyError::TooManyRules)` if it would exceed the limit
    ///
    /// # Examples
    ///
    /// ```
    /// # use acls_rs::policy::{RuleLimitedPolicy, PolicyError};
    /// # struct MyPolicy { max_rules: usize, current_count: usize }
    /// # impl RuleLimitedPolicy for MyPolicy {
    /// #     fn max_limit(&self) -> usize { self.max_rules }
    /// #     fn current_count(&self) -> usize { self.current_count }
    /// # }
    /// let policy = MyPolicy { max_rules: 100, current_count: 95 };
    ///
    /// // Can add 5 more rules (total = 100)
    /// assert!(policy.check_limit(5).is_ok());
    ///
    /// // Cannot add 6 more rules (total = 101)
    /// match policy.check_limit(6) {
    ///     Err(PolicyError::TooManyRules { requested, maximum }) => {
    ///         assert_eq!(requested, 101);
    ///         assert_eq!(maximum, 100);
    ///     }
    ///     _ => panic!("expected TooManyRules error"),
    /// }
    /// ```
    fn check_limit(&self, additional: usize) -> Result<(), PolicyError> {
        let max = self.max_limit();
        if max == 0 {
            return Ok(());
        }

        let total = self.current_count().saturating_add(additional);
        if total > max {
            return Err(PolicyError::TooManyRules {
                requested: total,
                maximum: max,
            });
        }

        Ok(())
    }

    /// Checks if a specific total count would exceed the maximum limit.
    ///
    /// Unlike `check_limit`, this validates against an absolute count
    /// rather than an additional count.
    ///
    /// # Arguments
    ///
    /// * `total` - Total number of rules to validate
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the count is within limits
    /// - `Err(PolicyError::TooManyRules)` if it exceeds the limit
    ///
    /// # Examples
    ///
    /// ```
    /// # use acls_rs::policy::{RuleLimitedPolicy, PolicyError};
    /// # struct MyPolicy { max_rules: usize, current_count: usize }
    /// # impl RuleLimitedPolicy for MyPolicy {
    /// #     fn max_limit(&self) -> usize { self.max_rules }
    /// #     fn current_count(&self) -> usize { self.current_count }
    /// # }
    /// let policy = MyPolicy { max_rules: 1000, current_count: 0 };
    ///
    /// // Validate capacity allocation
    /// assert!(policy.check_total(500).is_ok());
    /// assert!(policy.check_total(1001).is_err());
    /// ```
    fn check_total(&self, total: usize) -> Result<(), PolicyError> {
        let max = self.max_limit();
        if max == 0 {
            return Ok(());
        }

        if total > max {
            return Err(PolicyError::TooManyRules {
                requested: total,
                maximum: max,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPolicy {
        max_rules: usize,
        current_count: usize,
    }

    impl RuleLimitedPolicy for TestPolicy {
        fn max_limit(&self) -> usize {
            self.max_rules
        }

        fn current_count(&self) -> usize {
            self.current_count
        }
    }

    #[test]
    fn test_no_limit() {
        let policy = TestPolicy {
            max_rules: 0,
            current_count: 1_000_000,
        };
        assert!(policy.check_limit(1_000_000).is_ok());
        assert!(policy.check_total(999_999_999).is_ok());
    }

    #[test]
    fn test_within_limit() {
        let policy = TestPolicy {
            max_rules: 100,
            current_count: 50,
        };
        assert!(policy.check_limit(30).is_ok());
        assert!(policy.check_limit(50).is_ok());
    }

    #[test]
    fn test_exact_limit() {
        let policy = TestPolicy {
            max_rules: 100,
            current_count: 90,
        };
        assert!(policy.check_limit(10).is_ok());
    }

    #[test]
    fn test_exceed_limit() {
        let policy = TestPolicy {
            max_rules: 100,
            current_count: 90,
        };
        match policy.check_limit(11) {
            Err(PolicyError::TooManyRules { requested, maximum }) => {
                assert_eq!(requested, 101);
                assert_eq!(maximum, 100);
            }
            _ => panic!("expected TooManyRules error"),
        }
    }

    #[test]
    fn test_check_total_within() {
        let policy = TestPolicy {
            max_rules: 1000,
            current_count: 0,
        };
        assert!(policy.check_total(500).is_ok());
        assert!(policy.check_total(1000).is_ok());
    }

    #[test]
    fn test_check_total_exceed() {
        let policy = TestPolicy {
            max_rules: 1000,
            current_count: 0,
        };
        match policy.check_total(1001) {
            Err(PolicyError::TooManyRules { requested, maximum }) => {
                assert_eq!(requested, 1001);
                assert_eq!(maximum, 1000);
            }
            _ => panic!("expected TooManyRules error"),
        }
    }
}
