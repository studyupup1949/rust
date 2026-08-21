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

        let mut indexed_rules: HashSet<usize> = HashSet::new();
        let mut declared_dims: HashMap<usize, HashSet<String>> = HashMap::new();

        for (rule_idx, rule) in rules.iter().enumerate() {
            if !rule.is_enabled() {
                continue;
            }

            // Optimization: if universal allow exists, skip indexing allow rules
            // since they'll never be checked (all allow checks bypass when universal allow present)
            if only_deny && rule.is_allow() {
                continue;
            }

            indexed_rules.insert(rule_idx);

            // Index this rule in each dimension
            for (dimension, attr_value) in &rule.dimensions {
                declared_dims
                    .entry(rule_idx)
                    .or_default()
                    .insert(dimension.clone());

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

        // A rule that never declares a given dimension is, per
        // `AbacPolicyCore::rule_matches`, unconstrained on that dimension —
        // functionally identical to declaring `AttributeValue::All`. Every
        // dimension index built above only knows about rules that explicitly
        // touched that dimension, so without this pass, an indexed rule that
        // omits a dimension some *other* rule declares would be silently
        // pruned by `find_candidates`'s per-dimension intersection even
        // though `rule_matches` would have matched it (e.g. a zero-dimension
        // global deny-all rule disappearing the moment any other rule in the
        // same policy indexes any dimension). Backfill it into every
        // dimension's `all_rules` set so the index and the evaluator agree.
        for (dimension, dim_index) in self.indices.iter_mut() {
            for &rule_idx in &indexed_rules {
                let declared = declared_dims
                    .get(&rule_idx)
                    .is_some_and(|dims| dims.contains(dimension));
                if !declared {
                    dim_index.all_rules.insert(rule_idx);
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

    /// Regression test: a rule that declares zero dimensions (e.g. a global
    /// deny-all) must remain a candidate for every request, even once another
    /// rule in the same policy causes some dimension to be indexed. Before
    /// this fix, `find_candidates` only ever added a rule to a dimension's
    /// `all_rules` set when the rule *explicitly* used `AttributeValue::All`
    /// for that dimension — a rule that never mentioned the dimension at all
    /// was invisible to that dimension's index and got silently intersected
    /// away, even though `AbacPolicyCore::rule_matches` treats an
    /// undeclared dimension as unconstrained (identical to `All`).
    #[test]
    fn test_zero_dimension_rule_survives_unrelated_dimension_index() {
        let mut rules = Vec::new();

        // Rule 0: global deny-all — declares no dimensions whatsoever.
        let mut deny_all = AbacRule::new("deny-all");
        deny_all.enable();
        rules.push(deny_all);

        // Rule 1: an unrelated allow rule scoped to a single "resource"
        // value. Its mere presence causes a "resource" dimension index to
        // be built.
        let mut allow_one = AbacRule::new("allow-one");
        let mut resource_set = HashSet::new();
        resource_set.insert(AttributeType::String("db-01".into()));
        allow_one.add_dimension("resource", AttributeValue::Specific(resource_set));
        allow_one.enable();
        rules.push(allow_one);

        let mut index = CompositeIndex::new();
        index.build_from_rules(&rules, false);

        // A request naming a "resource" the deny-all rule never mentioned
        // must still surface the deny-all rule as a candidate.
        let mut request = AbacRequest::new();
        request
            .add_attribute("resource", AttributeType::String("web-02".into()), vec![])
            .unwrap();

        let candidates = index.find_candidates(&request);
        assert!(
            candidates.contains(&0),
            "zero-dimension rule must survive pruning on a dimension it never declared"
        );
    }

    /// Same scenario as above but with two indexed dimensions, confirming
    /// the fix generalizes past a single extra dimension.
    #[test]
    fn test_sparse_rule_survives_multi_dimension_index() {
        let mut rules = Vec::new();

        // Rule 0: only constrains "user" — says nothing about "resource".
        let mut sparse = AbacRule::new("sparse-deny");
        sparse.set_deny();
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("mallory".into()));
        sparse.add_dimension("user", AttributeValue::Specific(user_set));
        sparse.enable();
        rules.push(sparse);

        // Rule 1: constrains "resource" only, causing that dimension to be
        // indexed.
        let mut other = AbacRule::new("allow-db");
        let mut resource_set = HashSet::new();
        resource_set.insert(AttributeType::String("db-01".into()));
        other.add_dimension("resource", AttributeValue::Specific(resource_set));
        other.enable();
        rules.push(other);

        let mut index = CompositeIndex::new();
        index.build_from_rules(&rules, false);

        let mut request = AbacRequest::new();
        request
            .add_attribute("user", AttributeType::String("mallory".into()), vec![])
            .unwrap();
        request
            .add_attribute("resource", AttributeType::String("web-02".into()), vec![])
            .unwrap();

        let candidates = index.find_candidates(&request);
        assert!(
            candidates.contains(&0),
            "rule 0 declares 'user' but not 'resource'; it must still be a \
             candidate for a request naming a 'resource' it never mentioned"
        );
    }
}
