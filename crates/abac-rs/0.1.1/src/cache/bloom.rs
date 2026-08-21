//! Bloom filter pre-screening for fast negative checks.
//!
//! Provides O(1) guaranteed rejection of non-matching requests with ~1% false
//! positive rate.

use std::collections::HashSet;

use fastbloom::BloomFilter;

use crate::{AbacRule, AttributeType, AttributeValue};

/// Per-dimension Bloom filters for fast negative checks.
///
/// Enables O(1) rejection of requests that definitely won't match any rules,
/// with ~1% false positive rate (rare cases where Bloom says "maybe" but no
/// rules actually match).
#[derive(Debug, Clone)]
pub struct DimensionBloom {
    dimension: String,
    primary: BloomFilter,
    groups: BloomFilter,
    has_all_category: bool,
}

impl DimensionBloom {
    /// Returns the dimension name.
    pub fn dimension(&self) -> &str {
        &self.dimension
    }

    /// Returns whether any rule uses `All` (wildcard) for this dimension.
    pub fn has_all_category(&self) -> bool {
        self.has_all_category
    }
}

impl DimensionBloom {
    /// Create Bloom filters for a dimension from rules.
    ///
    /// # Arguments
    ///
    /// * `dimension` - Dimension name
    /// * `rules` - All enabled rules
    /// * `false_positive_rate` - Target false positive rate (e.g., 0.01 for 1%)
    pub fn from_rules(
        dimension: impl Into<String>,
        rules: &[&AbacRule],
        false_positive_rate: f64,
    ) -> Self {
        let dimension = dimension.into();

        // Collect all values for this dimension
        let mut primary_values = HashSet::new();
        let mut group_values = HashSet::new();
        let mut has_all = false;

        for rule in rules {
            if !rule.is_enabled() {
                continue;
            }

            if let Some(attr_value) = rule.get_dimension(&dimension) {
                match attr_value {
                    AttributeValue::All => {
                        has_all = true;
                    }
                    AttributeValue::Specific(set) => {
                        for value in set {
                            // Heuristic: values starting with "group:" are groups
                            if let AttributeType::String(s) = value {
                                if s.starts_with("group:") {
                                    group_values.insert(value.clone());
                                } else {
                                    primary_values.insert(value.clone());
                                }
                            } else {
                                primary_values.insert(value.clone());
                            }
                        }
                    }
                }
            }
        }

        // Create Bloom filters
        let primary_count = primary_values.len().max(1);
        let group_count = group_values.len().max(1);

        let mut primary_bloom =
            BloomFilter::with_false_pos(false_positive_rate).expected_items(primary_count);

        let mut groups_bloom =
            BloomFilter::with_false_pos(false_positive_rate).expected_items(group_count);

        // Insert values
        for value in primary_values {
            primary_bloom.insert(&value_to_bytes(&value));
        }

        for value in group_values {
            groups_bloom.insert(&value_to_bytes(&value));
        }

        Self {
            dimension,
            primary: primary_bloom,
            groups: groups_bloom,
            has_all_category: has_all,
        }
    }

    /// Check if a request value definitely does NOT match any rules.
    ///
    /// Returns `true` if we can guarantee no rules match (safe to reject).
    /// Returns `false` if rules might match (need to check further).
    pub fn definitely_no_match(&self, value: &AttributeType, groups: &[AttributeType]) -> bool {
        // If any rule has `All`, we can't rule out a match
        if self.has_all_category {
            return false;
        }

        // Check primary value
        if self.primary.contains(&value_to_bytes(value)) {
            return false; // Might match
        }

        // Check groups
        for group in groups {
            if self.groups.contains(&value_to_bytes(group)) {
                return false; // Might match
            }
        }

        // Definitely no match
        true
    }
}

/// Convert AttributeType to bytes for Bloom filter insertion/lookup.
///
/// Uses a stack-allocated buffer (max 17 bytes) to avoid heap allocation on
/// every call. Only String and IPv6 CIDR values exceed the inline capacity
/// and fall back to heap allocation.
fn value_to_bytes(value: &AttributeType) -> smallvec::SmallVec<[u8; 17]> {
    match value {
        AttributeType::String(s) => smallvec::SmallVec::from_slice(s.as_bytes()),
        AttributeType::Integer(i) => smallvec::SmallVec::from_slice(&i.to_le_bytes()),
        AttributeType::Float(f) => smallvec::SmallVec::from_slice(&f.to_bits().to_le_bytes()),
        AttributeType::IpAddr(ip) => match ip {
            std::net::IpAddr::V4(v4) => smallvec::SmallVec::from_slice(&v4.octets()),
            std::net::IpAddr::V6(v6) => smallvec::SmallVec::from_slice(&v6.octets()),
        },
        AttributeType::IpCidr(cidr) => {
            let mut bytes: smallvec::SmallVec<[u8; 17]> = match cidr.network() {
                std::net::IpAddr::V4(v4) => smallvec::SmallVec::from_slice(&v4.octets()),
                std::net::IpAddr::V6(v6) => smallvec::SmallVec::from_slice(&v6.octets()),
            };
            bytes.push(cidr.prefix());
            bytes
        }
        AttributeType::Custom(c) => {
            let mut bytes = smallvec::SmallVec::new();
            bytes.push(0xff);
            bytes.extend_from_slice(&c.hash().to_le_bytes());
            bytes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHashSet as HashSet;

    #[test]
    fn test_bloom_from_rules_specific() {
        let mut rule1 = AbacRule::new("rule1");
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("alice".into()));
        user_set.insert(AttributeType::String("bob".into()));
        rule1.add_dimension("user", AttributeValue::Specific(user_set));
        rule1.enable();

        let mut rule2 = AbacRule::new("rule2");
        let mut user_set2 = HashSet::new();
        user_set2.insert(AttributeType::String("charlie".into()));
        rule2.add_dimension("user", AttributeValue::Specific(user_set2));
        rule2.enable();

        let rules = vec![&rule1, &rule2];
        let bloom = DimensionBloom::from_rules("user", &rules, 0.01);

        assert!(!bloom.has_all_category());

        // Values in the rules should not be ruled out
        assert!(!bloom.definitely_no_match(&AttributeType::String("alice".into()), &[]));
        assert!(!bloom.definitely_no_match(&AttributeType::String("bob".into()), &[]));
        assert!(!bloom.definitely_no_match(&AttributeType::String("charlie".into()), &[]));

        // Value not in rules should be ruled out (most of the time, with 1% false positive)
        // Note: Due to false positives, we can't guarantee this will always be true
        let unknown = bloom.definitely_no_match(&AttributeType::String("eve".into()), &[]);
        // Just check that the method runs without error
        let _ = unknown;
    }

    #[test]
    fn test_bloom_from_rules_all() {
        let mut rule = AbacRule::new("rule");
        rule.add_dimension("user", AttributeValue::All);
        rule.enable();

        let rules = vec![&rule];
        let bloom = DimensionBloom::from_rules("user", &rules, 0.01);

        assert!(bloom.has_all_category());

        // With All category, can never rule out matches
        assert!(!bloom.definitely_no_match(&AttributeType::String("alice".into()), &[]));
        assert!(!bloom.definitely_no_match(&AttributeType::String("anyone".into()), &[]));
    }

    #[test]
    fn test_bloom_with_groups() {
        let mut rule = AbacRule::new("rule");
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("group:admins".into()));
        user_set.insert(AttributeType::String("group:developers".into()));
        rule.add_dimension("user", AttributeValue::Specific(user_set));
        rule.enable();

        let rules = vec![&rule];
        let bloom = DimensionBloom::from_rules("user", &rules, 0.01);

        // Groups should be detected
        let groups = vec![AttributeType::String("group:admins".into())];
        assert!(!bloom.definitely_no_match(&AttributeType::String("alice".into()), &groups));

        // Non-matching groups
        let wrong_groups = vec![AttributeType::String("group:guests".into())];
        // This might or might not be ruled out due to false positives
        let _ = bloom.definitely_no_match(&AttributeType::String("alice".into()), &wrong_groups);
    }
}
