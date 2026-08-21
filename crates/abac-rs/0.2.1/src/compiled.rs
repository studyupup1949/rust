//! Pre-compiled evaluation for rules with consistent dimensions.
//!
//! When all rules use the same set of dimensions (e.g., user/resource/action),
//! we can pre-compile an optimized evaluation path that avoids HashMap lookups.

use crate::{AbacRequest, AbacRule, AttributeType, AttributeValue, Decision};

/// Compiled evaluator for rules with consistent dimensions.
///
/// Stores dimension order and uses array-based lookups instead of HashMap.
#[derive(Debug, Clone)]
pub struct CompiledEvaluator {
    /// Ordered list of dimension names (e.g., ["action", "resource", "user"])
    dimensions: Vec<String>,

    /// Deny rules to check (if universal allow exists, only these matter)
    deny_rules: Vec<CompiledRule>,

    /// Allow rules to check (only if no universal allow)
    allow_rules: Vec<CompiledRule>,

    /// Whether a universal allow rule exists
    has_universal_allow: bool,
}

/// A rule compiled for fast evaluation.
#[derive(Debug, Clone)]
struct CompiledRule {
    /// Pre-compiled attribute requirements (one per dimension, in dimension order)
    /// None means the dimension was missing in the rule
    attributes: Vec<Option<AttributeValue>>,
}

impl CompiledEvaluator {
    /// Try to build a compiled evaluator from rules.
    ///
    /// Returns None if rules don't have consistent dimensions.
    pub fn try_build(rules: &[AbacRule]) -> Option<Self> {
        // Find common dimensions across all enabled rules
        let dimensions = Self::detect_common_dimensions(rules)?;

        let mut deny_rules = Vec::new();
        let mut allow_rules = Vec::new();
        let mut has_universal_allow = false;

        for rule in rules.iter() {
            if !rule.is_enabled() {
                continue;
            }

            // Check if universal allow
            if rule.is_allow()
                && !rule.dimensions.is_empty()
                && rule
                    .dimensions
                    .values()
                    .all(|v| matches!(v, AttributeValue::All))
            {
                has_universal_allow = true;
            }

            // Compile rule attributes in dimension order
            let mut attributes = Vec::with_capacity(dimensions.len());
            for dim in &dimensions {
                attributes.push(rule.dimensions.get(dim).cloned());
            }

            let compiled = CompiledRule { attributes };

            if rule.is_deny() {
                deny_rules.push(compiled);
            } else {
                allow_rules.push(compiled);
            }
        }

        Some(CompiledEvaluator {
            dimensions,
            deny_rules,
            allow_rules,
            has_universal_allow,
        })
    }

    /// Detect common dimensions across all rules.
    ///
    /// Returns None if rules have inconsistent dimensions.
    fn detect_common_dimensions(rules: &[AbacRule]) -> Option<Vec<String>> {
        let enabled_rules: Vec<_> = rules.iter().filter(|r| r.is_enabled()).collect();

        if enabled_rules.is_empty() {
            return None;
        }

        // Get dimensions from first rule
        let mut common_dims: Vec<String> = enabled_rules[0]
            .dimension_names()
            .map(|s| s.to_string())
            .collect();
        common_dims.sort();

        // Verify all rules have the same dimensions
        for rule in &enabled_rules[1..] {
            let mut rule_dims: Vec<String> =
                rule.dimension_names().map(|s| s.to_string()).collect();
            rule_dims.sort();

            if rule_dims != common_dims {
                return None; // Inconsistent dimensions
            }
        }

        Some(common_dims)
    }

    /// Evaluate a request using the compiled rules.
    ///
    /// Pre-extracts all attribute values from the request into an array for fast access.
    #[inline]
    pub fn evaluate(&self, request: &AbacRequest) -> Decision {
        // Optimization: Use stack-allocated array for common case (<=8 dimensions)
        // This avoids heap allocation for the vast majority of policies
        const MAX_INLINE_DIMS: usize = 8;

        if self.dimensions.len() <= MAX_INLINE_DIMS {
            // Fast path: stack allocation
            // Explicit initialization to help compiler optimize
            #[allow(invalid_value)]
            let mut request_attrs_array: [Option<(&AttributeType, &[AttributeType])>;
                MAX_INLINE_DIMS] = [None, None, None, None, None, None, None, None];

            // Unrolled loop for common dimensions (most policies have 3-4 dimensions)
            if let Some(dim) = self.dimensions.first() {
                request_attrs_array[0] = request.get_attribute(dim).map(|(v, g)| (v, g.as_slice()));
            }
            if let Some(dim) = self.dimensions.get(1) {
                request_attrs_array[1] = request.get_attribute(dim).map(|(v, g)| (v, g.as_slice()));
            }
            if let Some(dim) = self.dimensions.get(2) {
                request_attrs_array[2] = request.get_attribute(dim).map(|(v, g)| (v, g.as_slice()));
            }
            if let Some(dim) = self.dimensions.get(3) {
                request_attrs_array[3] = request.get_attribute(dim).map(|(v, g)| (v, g.as_slice()));
            }
            // Remaining dimensions (rare case)
            for (i, dim) in self.dimensions.iter().enumerate().skip(4) {
                request_attrs_array[i] = request.get_attribute(dim).map(|(v, g)| (v, g.as_slice()));
            }

            let request_attrs = &request_attrs_array[..self.dimensions.len()];

            // Phase 1: Check deny rules (early return on first match)
            for deny_rule in &self.deny_rules {
                if self.rule_matches(&deny_rule.attributes, request_attrs) {
                    return Decision::Deny;
                }
            }

            // Phase 2: Universal allow fast-path
            if self.has_universal_allow {
                return Decision::Allow;
            }

            // Phase 3: Check allow rules (early return on first match)
            for allow_rule in &self.allow_rules {
                if self.rule_matches(&allow_rule.attributes, request_attrs) {
                    return Decision::Allow;
                }
            }

            Decision::Deny
        } else {
            // Slow path: heap allocation for policies with >8 dimensions
            let request_attrs: Vec<Option<(&AttributeType, &[AttributeType])>> = self
                .dimensions
                .iter()
                .map(|dim| request.get_attribute(dim).map(|(v, g)| (v, g.as_slice())))
                .collect();

            // Phase 1: Check deny rules
            for deny_rule in &self.deny_rules {
                if self.rule_matches(&deny_rule.attributes, &request_attrs) {
                    return Decision::Deny;
                }
            }

            // Phase 2: Universal allow fast-path
            if self.has_universal_allow {
                return Decision::Allow;
            }

            // Phase 3: Check allow rules
            for allow_rule in &self.allow_rules {
                if self.rule_matches(&allow_rule.attributes, &request_attrs) {
                    return Decision::Allow;
                }
            }

            Decision::Deny
        }
    }

    /// Check if a compiled rule matches the pre-extracted request attributes.
    ///
    /// This is the hot path - uses array indexing instead of HashMap lookups.
    #[inline(always)]
    fn rule_matches(
        &self,
        rule_attrs: &[Option<AttributeValue>],
        request_attrs: &[Option<(&AttributeType, &[AttributeType])>],
    ) -> bool {
        // Check each dimension in order
        for (rule_attr, request_attr) in rule_attrs.iter().zip(request_attrs.iter()) {
            match (rule_attr, request_attr) {
                (None, _) | (_, None) => return false, // Missing dimension
                (Some(AttributeValue::All), Some(_)) => continue, // All matches anything
                (Some(AttributeValue::Specific(set)), Some((value, groups))) => {
                    // Fast path: check primary value first (most common case)
                    if set.contains(value) {
                        continue;
                    }
                    // Slow path: check groups only if primary didn't match
                    if groups.is_empty() || !groups.iter().any(|g| set.contains(g)) {
                        return false;
                    }
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHashSet as HashSet;

    #[test]
    fn test_detect_common_dimensions() {
        let mut rules = Vec::new();

        let mut rule1 = AbacRule::new("rule1");
        rule1.add_dimension("user", AttributeValue::All);
        rule1.add_dimension("resource", AttributeValue::All);
        rule1.add_dimension("action", AttributeValue::All);
        rule1.enable();
        rules.push(rule1);

        let mut rule2 = AbacRule::new("rule2");
        rule2.add_dimension("user", AttributeValue::All);
        rule2.add_dimension("resource", AttributeValue::All);
        rule2.add_dimension("action", AttributeValue::All);
        rule2.enable();
        rules.push(rule2);

        let dims = CompiledEvaluator::detect_common_dimensions(&rules);
        assert!(dims.is_some());
        let dims = dims.unwrap();
        assert_eq!(dims.len(), 3);
        assert!(dims.contains(&"user".to_string()));
        assert!(dims.contains(&"resource".to_string()));
        assert!(dims.contains(&"action".to_string()));
    }

    #[test]
    fn test_compiled_evaluator() {
        let mut rules = Vec::new();

        // Allow rule: alice can read anything
        let mut rule1 = AbacRule::new("allow-alice");
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("alice".into()));
        rule1.add_dimension("user", AttributeValue::Specific(user_set));
        rule1.add_dimension("resource", AttributeValue::All);
        rule1.add_dimension("action", AttributeValue::All);
        rule1.enable();
        rules.push(rule1);

        let evaluator = CompiledEvaluator::try_build(&rules).expect("Should compile");

        // Test matching request
        let mut request = AbacRequest::new();
        request
            .add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();
        request
            .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
            .unwrap();
        request
            .add_attribute("action", AttributeType::String("read".into()), vec![])
            .unwrap();

        assert_eq!(evaluator.evaluate(&request), Decision::Allow);

        // Test non-matching request
        let mut request2 = AbacRequest::new();
        request2
            .add_attribute("user", AttributeType::String("bob".into()), vec![])
            .unwrap();
        request2
            .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
            .unwrap();
        request2
            .add_attribute("action", AttributeType::String("read".into()), vec![])
            .unwrap();

        assert_eq!(evaluator.evaluate(&request2), Decision::Deny);
    }
}
