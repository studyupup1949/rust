//! Pluggable matching strategies for attribute evaluation.
//!
//! Matchers determine whether a request attribute value satisfies a rule's
//! attribute requirement. The `Matcher` trait enables custom matching logic
//! (predicates, ranges, CIDR checks) without modifying the core engine.

use crate::attribute::{AttributeType, AttributeValue};

/// Pluggable attribute matching strategy.
///
/// Implementations define how request attributes are matched against rule
/// requirements. The default `ExactMatcher` uses HashSet membership (O(1)),
/// while custom matchers can implement predicates, ranges, or other logic.
///
/// # Thread Safety
///
/// Matchers must be `Send + Sync` as they're shared across evaluations.
pub trait Matcher: Send + Sync {
    /// Check if a request attribute matches a rule requirement.
    ///
    /// # Arguments
    ///
    /// * `rule_value` - The attribute value specified in the rule (All or Specific set)
    /// * `request_value` - The primary attribute value from the request
    /// * `request_groups` - Group memberships for this attribute (e.g., user groups)
    ///
    /// # Returns
    ///
    /// `true` if the request matches the rule requirement, `false` otherwise.
    fn matches(
        &self,
        rule_value: &AttributeValue,
        request_value: &AttributeType,
        request_groups: &[AttributeType],
    ) -> bool;

    /// Check if this matcher supports Bloom filter optimization.
    ///
    /// Returns `false` for custom matchers with predicate logic that can't be
    /// pre-screened. The default is `true` (exact matching).
    fn supports_bloom_filter(&self) -> bool {
        true
    }
}

/// Exact match using HashSet membership (like hbac-rs).
///
/// This is the default matcher and provides the best performance:
/// - O(1) membership checks via HashSet
/// - Compatible with Bloom filter pre-screening
/// - Zero-cost abstraction over raw HashSet lookups
///
/// # Matching Rules
///
/// - `AttributeValue::All` matches any request value
/// - `AttributeValue::Specific(set)` matches if `request_value` OR any
///   `request_groups` member is in `set`
///
/// # Examples
///
/// ```
/// use abac_rs::{ExactMatcher, Matcher, AttributeValue, AttributeType};
/// use ahash::AHashSet as HashSet;
///
/// let matcher = ExactMatcher;
///
/// // All matches everything
/// assert!(matcher.matches(&AttributeValue::All, &AttributeType::String("alice".into()), &[]));
///
/// // Specific set checks membership
/// let mut allowed = HashSet::new();
/// allowed.insert(AttributeType::String("alice".into()));
/// allowed.insert(AttributeType::String("group:admins".into()));
/// let specific = AttributeValue::Specific(allowed);
///
/// // Direct match
/// assert!(matcher.matches(&specific, &AttributeType::String("alice".into()), &[]));
///
/// // Group match
/// let groups = vec![AttributeType::String("group:admins".into())];
/// assert!(matcher.matches(&specific, &AttributeType::String("bob".into()), &groups));
///
/// // No match
/// assert!(!matcher.matches(&specific, &AttributeType::String("eve".into()), &[]));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ExactMatcher;

impl Matcher for ExactMatcher {
    #[inline]
    fn matches(
        &self,
        rule_value: &AttributeValue,
        request_value: &AttributeType,
        request_groups: &[AttributeType],
    ) -> bool {
        match rule_value {
            AttributeValue::All => true,
            AttributeValue::Specific(set) => {
                set.contains(request_value) || request_groups.iter().any(|g| set.contains(g))
            }
        }
    }

    #[inline]
    fn supports_bloom_filter(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHashSet as HashSet;

    #[test]
    fn test_exact_matcher_all() {
        let matcher = ExactMatcher;
        let all = AttributeValue::All;

        assert!(matcher.matches(&all, &AttributeType::String("alice".into()), &[]));
        assert!(matcher.matches(&all, &AttributeType::String("bob".into()), &[]));
        assert!(matcher.matches(&all, &AttributeType::Integer(42), &[]));
    }

    #[test]
    fn test_exact_matcher_specific_direct() {
        let matcher = ExactMatcher;

        let mut set = HashSet::new();
        set.insert(AttributeType::String("alice".into()));
        set.insert(AttributeType::String("bob".into()));
        let specific = AttributeValue::Specific(set);

        assert!(matcher.matches(&specific, &AttributeType::String("alice".into()), &[]));
        assert!(matcher.matches(&specific, &AttributeType::String("bob".into()), &[]));
        assert!(!matcher.matches(&specific, &AttributeType::String("eve".into()), &[]));
    }

    #[test]
    fn test_exact_matcher_specific_groups() {
        let matcher = ExactMatcher;

        let mut set = HashSet::new();
        set.insert(AttributeType::String("group:admins".into()));
        set.insert(AttributeType::String("group:developers".into()));
        let specific = AttributeValue::Specific(set);

        let admin_groups = vec![
            AttributeType::String("group:admins".into()),
            AttributeType::String("group:users".into()),
        ];

        let dev_groups = vec![AttributeType::String("group:developers".into())];

        let no_groups: Vec<AttributeType> = vec![];

        // User not in set, but group matches
        assert!(matcher.matches(
            &specific,
            &AttributeType::String("alice".into()),
            &admin_groups
        ));
        assert!(matcher.matches(&specific, &AttributeType::String("bob".into()), &dev_groups));

        // User not in set, no matching groups
        assert!(!matcher.matches(&specific, &AttributeType::String("eve".into()), &no_groups));
    }

    #[test]
    fn test_exact_matcher_supports_bloom() {
        let matcher = ExactMatcher;
        assert!(matcher.supports_bloom_filter());
    }
}
