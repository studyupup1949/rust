//! Composite index for multi-dimensional rule candidate selection.
//!
//! Provides fast lookups by intersecting candidates across multiple dimensions.

use std::collections::{HashMap, HashSet};

use crate::{AbacRequest, AbacRule, AttributeType};

/// Multi-dimensional composite index for fast candidate selection.
///
/// Indexes rules by dimension values to quickly find candidate rules that
/// might match a request.
#[derive(Debug, Clone)]
pub struct CompositeIndex {
    /// Per-dimension index: dimension → (value → rule names)
    indices: HashMap<String, DimensionIndex>,
}

/// Index for a single dimension.
#[derive(Debug, Clone)]
struct DimensionIndex {
    /// Primary values → rule indices
    primary_map: HashMap<AttributeType, HashSet<usize>>,

    /// Group values → rule indices
    group_map: HashMap<AttributeType, HashSet<usize>>,

    /// Rules with `All` category for this dimension (rule indices)
    all_rules: HashSet<usize>,
}

impl CompositeIndex {
    /// Create an empty composite index.
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }

    /// Build the index from rules.
    ///
    /// # Arguments
    ///
    /// * `rules` - All rules in the policy (Vec of rules)
    /// * `only_deny` - If true, only index deny rules (optimization when universal allow exists)
    pub fn build_from_rules(&mut self, rules: &[AbacRule], only_deny: bool) {
        self.build_from_rules_with_opaque_dims(rules, only_deny, &HashSet::new());
    }

    /// Build the index from rules, treating specified dimensions as opaque.
    ///
    /// Dimensions in `opaque_dims` have custom matchers registered, so their
    /// values cannot be used for exact-match candidate pruning. Rules with
    /// values in these dimensions are indexed as if they had `All`, ensuring
    /// they are never falsely eliminated from the candidate set.
    pub fn build_from_rules_with_opaque_dims(
        &mut self,
        rules: &[AbacRule],
        only_deny: bool,
        opaque_dims: &HashSet<String>,
    ) {
        self.indices.clear();

        for (rule_idx, rule) in rules.iter().enumerate() {
            if !rule.is_enabled() {
                continue;
            }

            // Optimization: if universal allow exists, skip indexing allow rules
            // since they'll never be checked (all allow checks bypass when universal allow present)
            if only_deny && rule.is_allow() {
                continue;
            }

            // Index this rule in each dimension
            for (dimension, attr_value) in &rule.dimensions {
                let dim_index =
                    self.indices
                        .entry(dimension.clone())
                        .or_insert_with(|| DimensionIndex {
                            primary_map: HashMap::new(),
                            group_map: HashMap::new(),
                            all_rules: HashSet::new(),
                        });

                let is_opaque = opaque_dims.contains(dimension);

                match attr_value {
                    crate::AttributeValue::All => {
                        dim_index.all_rules.insert(rule_idx);
                    }
                    _ if is_opaque => {
                        dim_index.all_rules.insert(rule_idx);
                    }
                    crate::AttributeValue::Specific(set) => {
                        for value in set {
                            // Heuristic: values starting with "group:" are groups
                            if let AttributeType::String(s) = value {
                                if s.starts_with("group:") {
                                    dim_index
                                        .group_map
                                        .entry(value.clone())
                                        .or_default()
                                        .insert(rule_idx);
                                } else {
                                    dim_index
                                        .primary_map
                                        .entry(value.clone())
                                        .or_default()
                                        .insert(rule_idx);
                                }
                            } else {
                                dim_index
                                    .primary_map
                                    .entry(value.clone())
                                    .or_default()
                                    .insert(rule_idx);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Find candidate rules for a request.
    ///
    /// Returns rule indices that might match by intersecting candidates across
    /// all dimensions present in the request.
    ///
    /// Optimized to minimize allocations by using `retain()` for intersection
    /// instead of creating new HashSets. Returns indices instead of names to
    /// avoid String clones.
    pub fn find_candidates(&self, request: &AbacRequest) -> HashSet<usize> {
        if self.indices.is_empty() {
            return HashSet::new();
        }

        let mut result: Option<HashSet<usize>> = None;

        for (dimension, (value, groups)) in request.attributes() {
            if let Some(dim_index) = self.indices.get(dimension) {
                match result {
                    None => {
                        // First dimension: collect all candidates
                        // Optimization: avoid clone by building directly
                        let mut dim_candidates = HashSet::with_capacity(
                            dim_index.all_rules.len()
                                + dim_index.primary_map.get(value).map_or(0, |s| s.len())
                                + groups
                                    .iter()
                                    .map(|g| dim_index.group_map.get(g).map_or(0, |s| s.len()))
                                    .sum::<usize>(),
                        );

                        dim_candidates.extend(dim_index.all_rules.iter().copied());

                        if let Some(primary_rules) = dim_index.primary_map.get(value) {
                            dim_candidates.extend(primary_rules.iter().copied());
                        }

                        for group in groups {
                            if let Some(group_rules) = dim_index.group_map.get(group) {
                                dim_candidates.extend(group_rules.iter().copied());
                            }
                        }

                        result = Some(dim_candidates);
                    }
                    Some(ref mut prev) => {
                        // Subsequent dimensions: intersect by retaining only matching candidates
                        // Optimization: cache HashMap lookups outside the retain closure
                        let primary_rules = dim_index.primary_map.get(value);
                        let group_rules: Vec<_> = groups
                            .iter()
                            .filter_map(|g| dim_index.group_map.get(g))
                            .collect();

                        prev.retain(|rule_idx| {
                            // Keep if in all_rules, primary_map, or group_map
                            dim_index.all_rules.contains(rule_idx)
                                || primary_rules.is_some_and(|set| set.contains(rule_idx))
                                || group_rules.iter().any(|set| set.contains(rule_idx))
                        });
                    }
                }
            }
        }

        result.unwrap_or_default()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

impl Default for CompositeIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AbacRule, AttributeValue};
    use ahash::AHashSet as HashSet;

    #[test]
    fn test_composite_index_empty() {
        let index = CompositeIndex::new();
        let request = AbacRequest::new();
        let candidates = index.find_candidates(&request);
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_composite_index_single_dimension() {
        let mut rules = Vec::new();

        let mut rule1 = AbacRule::new("rule1");
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("alice".into()));
        rule1.add_dimension("user", AttributeValue::Specific(user_set));
        rule1.enable();
        rules.push(rule1); // index 0

        let mut rule2 = AbacRule::new("rule2");
        let mut user_set2 = HashSet::new();
        user_set2.insert(AttributeType::String("bob".into()));
        rule2.add_dimension("user", AttributeValue::Specific(user_set2));
        rule2.enable();
        rules.push(rule2); // index 1

        let mut index = CompositeIndex::new();
        index.build_from_rules(&rules, false);

        // Request for alice
        let mut request = AbacRequest::new();
        request
            .add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();

        let candidates = index.find_candidates(&request);
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(&0)); // rule1 is at index 0
    }

    #[test]
    fn test_composite_index_multi_dimension_intersection() {
        let mut rules = Vec::new();

        // Rule: alice can access db-01
        let mut rule1 = AbacRule::new("rule1");
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("alice".into()));
        rule1.add_dimension("user", AttributeValue::Specific(user_set));
        let mut resource_set = HashSet::new();
        resource_set.insert(AttributeType::String("db-01".into()));
        rule1.add_dimension("resource", AttributeValue::Specific(resource_set));
        rule1.enable();
        rules.push(rule1);

        // Rule: alice can access web-01
        let mut rule2 = AbacRule::new("rule2");
        let mut user_set2 = HashSet::new();
        user_set2.insert(AttributeType::String("alice".into()));
        rule2.add_dimension("user", AttributeValue::Specific(user_set2));
        let mut resource_set2 = HashSet::new();
        resource_set2.insert(AttributeType::String("web-01".into()));
        rule2.add_dimension("resource", AttributeValue::Specific(resource_set2));
        rule2.enable();
        rules.push(rule2);

        let mut index = CompositeIndex::new();
        index.build_from_rules(&rules, false);

        // Request: alice accessing db-01
        let mut request = AbacRequest::new();
        request
            .add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();
        request
            .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
            .unwrap();

        let candidates = index.find_candidates(&request);
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(&0));
    }

    #[test]
    fn test_composite_index_all_category() {
        let mut rules = Vec::new();

        // Rule: any user can access db-01
        let mut rule = AbacRule::new("rule");
        rule.add_dimension("user", AttributeValue::All);
        let mut resource_set = HashSet::new();
        resource_set.insert(AttributeType::String("db-01".into()));
        rule.add_dimension("resource", AttributeValue::Specific(resource_set));
        rule.enable();
        rules.push(rule);

        let mut index = CompositeIndex::new();
        index.build_from_rules(&rules, false);

        // Request: bob accessing db-01
        let mut request = AbacRequest::new();
        request
            .add_attribute("user", AttributeType::String("bob".into()), vec![])
            .unwrap();
        request
            .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
            .unwrap();

        let candidates = index.find_candidates(&request);
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(&0));
    }

    #[test]
    fn test_composite_index_with_groups() {
        let mut rules = Vec::new();

        // Rule: group:admins can access db-01
        let mut rule = AbacRule::new("rule");
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("group:admins".into()));
        rule.add_dimension("user", AttributeValue::Specific(user_set));
        let mut resource_set = HashSet::new();
        resource_set.insert(AttributeType::String("db-01".into()));
        rule.add_dimension("resource", AttributeValue::Specific(resource_set));
        rule.enable();
        rules.push(rule);

        let mut index = CompositeIndex::new();
        index.build_from_rules(&rules, false);

        // Request: alice in group:admins accessing db-01
        let mut request = AbacRequest::new();
        request
            .add_attribute(
                "user",
                AttributeType::String("alice".into()),
                vec![AttributeType::String("group:admins".into())],
            )
            .unwrap();
        request
            .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
            .unwrap();

        let candidates = index.find_candidates(&request);
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(&0));
    }
}
