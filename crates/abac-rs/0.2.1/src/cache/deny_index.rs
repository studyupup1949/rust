use ahash::AHashMap;

use crate::{AbacRequest, AbacRule, AttributeType, AttributeValue};

#[derive(Clone, Copy)]
struct BitPos {
    word: u32,
    mask: u64,
}

struct DimBitmapIndex {
    value_to_rules: AHashMap<AttributeType, Vec<BitPos>>,
    /// Pre-computed bitmask for rules with All in this dimension.
    all_mask: Vec<u64>,
    /// Whether any bit is set in all_mask (skip memcpy when false).
    has_any_all: bool,
}

struct ScratchBuffers {
    result: Vec<u64>,
    dim_mask: Vec<u64>,
}

pub(crate) struct AbacDenyIndex {
    dimensions: Vec<String>,
    dim_indexes: Vec<DimBitmapIndex>,
    num_rules: usize,
    words: usize,
    tail_mask: u64,
    scratch: ScratchBuffers,
}

impl AbacDenyIndex {
    pub(crate) fn try_build(rules: &[AbacRule]) -> Option<Self> {
        let deny_rules: Vec<(usize, &AbacRule)> = rules
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_enabled() && r.is_deny())
            .collect();

        if deny_rules.is_empty() {
            return None;
        }

        let mut dimensions: Vec<String> = deny_rules[0]
            .1
            .dimension_names()
            .map(|s| s.to_string())
            .collect();
        dimensions.sort();

        for &(_, rule) in &deny_rules[1..] {
            let mut rule_dims: Vec<String> =
                rule.dimension_names().map(|s| s.to_string()).collect();
            rule_dims.sort();
            if rule_dims != dimensions {
                return None;
            }
        }

        let num_rules = deny_rules.len();
        let words = num_rules.div_ceil(64);

        let mut dim_indexes: Vec<DimBitmapIndex> = dimensions
            .iter()
            .map(|_| DimBitmapIndex {
                value_to_rules: AHashMap::new(),
                all_mask: vec![0u64; words],
                has_any_all: false,
            })
            .collect();

        for (local_idx, &(_, rule)) in deny_rules.iter().enumerate() {
            let bp = BitPos {
                word: (local_idx / 64) as u32,
                mask: 1u64 << (local_idx % 64),
            };
            for (dim_pos, dim_name) in dimensions.iter().enumerate() {
                if let Some(attr_value) = rule.dimensions.get(dim_name) {
                    match attr_value {
                        AttributeValue::All => {
                            dim_indexes[dim_pos].all_mask[bp.word as usize] |= bp.mask;
                            dim_indexes[dim_pos].has_any_all = true;
                        }
                        AttributeValue::Specific(set) => {
                            for value in set {
                                dim_indexes[dim_pos]
                                    .value_to_rules
                                    .entry(value.clone())
                                    .or_default()
                                    .push(bp);
                            }
                        }
                    }
                }
            }
        }

        let tail_bits = num_rules % 64;
        let tail_mask = if tail_bits != 0 {
            (1u64 << tail_bits) - 1
        } else {
            u64::MAX
        };

        Some(Self {
            dimensions,
            dim_indexes,
            num_rules,
            words,
            tail_mask,
            scratch: ScratchBuffers {
                result: vec![0u64; words],
                dim_mask: vec![0u64; words],
            },
        })
    }

    #[inline]
    fn set_bits(mask: &mut [u64], positions: &[BitPos]) {
        for bp in positions {
            mask[bp.word as usize] |= bp.mask;
        }
    }

    pub(crate) fn has_deny_match(&mut self, request: &AbacRequest) -> bool {
        if self.num_rules == 0 {
            return false;
        }

        let words = self.words;
        let s = &mut self.scratch;

        for (dim_pos, dim_name) in self.dimensions.iter().enumerate() {
            let dim_idx = &self.dim_indexes[dim_pos];

            let Some((primary_value, groups)) = request.get_attribute(dim_name) else {
                return false;
            };

            if dim_pos == 0 {
                // First dimension: write directly to result (skip dim_mask intermediate)
                if dim_idx.has_any_all {
                    s.result.copy_from_slice(&dim_idx.all_mask);
                } else {
                    s.result.fill(0);
                }

                if let Some(positions) = dim_idx.value_to_rules.get(primary_value) {
                    Self::set_bits(&mut s.result, positions);
                }
                for group in groups {
                    if let Some(positions) = dim_idx.value_to_rules.get(group) {
                        Self::set_bits(&mut s.result, positions);
                    }
                }

                s.result[words - 1] &= self.tail_mask;

                if !s.result.iter().any(|&w| w != 0) {
                    return false;
                }
            } else {
                // Subsequent dimensions: build mask in dim_mask, AND into result
                if dim_idx.has_any_all {
                    s.dim_mask.copy_from_slice(&dim_idx.all_mask);
                } else {
                    s.dim_mask.fill(0);
                }

                if let Some(positions) = dim_idx.value_to_rules.get(primary_value) {
                    Self::set_bits(&mut s.dim_mask, positions);
                }
                for group in groups {
                    if let Some(positions) = dim_idx.value_to_rules.get(group) {
                        Self::set_bits(&mut s.dim_mask, positions);
                    }
                }

                // Iterator zip for aliasing hints → auto-vectorization
                for (r, d) in s.result.iter_mut().zip(s.dim_mask.iter()) {
                    *r &= *d;
                }
                if !s.result[..words].iter().any(|&w| w != 0) {
                    return false;
                }
            }
        }

        s.result.iter().any(|&w| w != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHashSet as HashSet;

    fn make_deny_rule(name: &str) -> AbacRule {
        let mut rule = AbacRule::new(name);
        rule.set_deny();
        rule.enable();
        rule
    }

    #[test]
    fn test_empty_rules_returns_none() {
        let rules: Vec<AbacRule> = vec![];
        assert!(AbacDenyIndex::try_build(&rules).is_none());
    }

    #[test]
    fn test_no_deny_rules_returns_none() {
        let mut rule = AbacRule::new("allow");
        rule.add_dimension("user", AttributeValue::All);
        rule.enable();
        assert!(AbacDenyIndex::try_build(&[rule]).is_none());
    }

    #[test]
    fn test_inconsistent_dimensions_returns_none() {
        let mut r1 = make_deny_rule("deny1");
        r1.add_dimension("user", AttributeValue::All);
        r1.add_dimension("resource", AttributeValue::All);

        let mut r2 = make_deny_rule("deny2");
        r2.add_dimension("user", AttributeValue::All);
        r2.add_dimension("action", AttributeValue::All);

        assert!(AbacDenyIndex::try_build(&[r1, r2]).is_none());
    }

    #[test]
    fn test_basic_deny_match() {
        let mut rule = make_deny_rule("deny_alice");
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("alice".into()));
        rule.add_dimension("user", AttributeValue::Specific(user_set));
        rule.add_dimension("resource", AttributeValue::All);

        let mut idx = AbacDenyIndex::try_build(&[rule]).unwrap();

        let mut req = AbacRequest::new();
        req.add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();
        req.add_attribute("resource", AttributeType::String("db".into()), vec![])
            .unwrap();
        assert!(idx.has_deny_match(&req));
    }

    #[test]
    fn test_no_match_different_value() {
        let mut rule = make_deny_rule("deny_alice");
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("alice".into()));
        rule.add_dimension("user", AttributeValue::Specific(user_set));
        rule.add_dimension("resource", AttributeValue::All);

        let mut idx = AbacDenyIndex::try_build(&[rule]).unwrap();

        let mut req = AbacRequest::new();
        req.add_attribute("user", AttributeType::String("bob".into()), vec![])
            .unwrap();
        req.add_attribute("resource", AttributeType::String("db".into()), vec![])
            .unwrap();
        assert!(!idx.has_deny_match(&req));
    }

    #[test]
    fn test_all_dimension_matching() {
        let mut rule = make_deny_rule("deny_all");
        rule.add_dimension("user", AttributeValue::All);
        rule.add_dimension("resource", AttributeValue::All);

        let mut idx = AbacDenyIndex::try_build(&[rule]).unwrap();

        let mut req = AbacRequest::new();
        req.add_attribute("user", AttributeType::String("anyone".into()), vec![])
            .unwrap();
        req.add_attribute("resource", AttributeType::String("anything".into()), vec![])
            .unwrap();
        assert!(idx.has_deny_match(&req));
    }

    #[test]
    fn test_group_matching() {
        let mut rule = make_deny_rule("deny_admins");
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("group:admins".into()));
        rule.add_dimension("user", AttributeValue::Specific(user_set));
        rule.add_dimension("resource", AttributeValue::All);

        let mut idx = AbacDenyIndex::try_build(&[rule]).unwrap();

        let mut req = AbacRequest::new();
        req.add_attribute(
            "user",
            AttributeType::String("alice".into()),
            vec![AttributeType::String("group:admins".into())],
        )
        .unwrap();
        req.add_attribute("resource", AttributeType::String("db".into()), vec![])
            .unwrap();
        assert!(idx.has_deny_match(&req));

        let mut req_no_group = AbacRequest::new();
        req_no_group
            .add_attribute("user", AttributeType::String("bob".into()), vec![])
            .unwrap();
        req_no_group
            .add_attribute("resource", AttributeType::String("db".into()), vec![])
            .unwrap();
        assert!(!idx.has_deny_match(&req_no_group));
    }

    #[test]
    fn test_request_missing_dimension() {
        let mut rule = make_deny_rule("deny_alice");
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("alice".into()));
        rule.add_dimension("user", AttributeValue::Specific(user_set));
        rule.add_dimension("resource", AttributeValue::All);

        let mut idx = AbacDenyIndex::try_build(&[rule]).unwrap();

        let mut req = AbacRequest::new();
        req.add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();
        // missing "resource" dimension
        assert!(!idx.has_deny_match(&req));
    }

    #[test]
    fn test_multi_dimension_intersection() {
        let mut rule = make_deny_rule("deny_alice_db");
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("alice".into()));
        rule.add_dimension("user", AttributeValue::Specific(user_set));
        let mut res_set = HashSet::new();
        res_set.insert(AttributeType::String("db-01".into()));
        rule.add_dimension("resource", AttributeValue::Specific(res_set));

        let mut idx = AbacDenyIndex::try_build(&[rule]).unwrap();

        let mut req_match = AbacRequest::new();
        req_match
            .add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();
        req_match
            .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
            .unwrap();
        assert!(idx.has_deny_match(&req_match));

        let mut req_wrong_resource = AbacRequest::new();
        req_wrong_resource
            .add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();
        req_wrong_resource
            .add_attribute("resource", AttributeType::String("web-01".into()), vec![])
            .unwrap();
        assert!(!idx.has_deny_match(&req_wrong_resource));

        let mut req_wrong_user = AbacRequest::new();
        req_wrong_user
            .add_attribute("user", AttributeType::String("bob".into()), vec![])
            .unwrap();
        req_wrong_user
            .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
            .unwrap();
        assert!(!idx.has_deny_match(&req_wrong_user));
    }

    #[test]
    fn test_disabled_deny_rules_excluded() {
        let mut enabled = make_deny_rule("deny_alice");
        let mut user_set = HashSet::new();
        user_set.insert(AttributeType::String("alice".into()));
        enabled.add_dimension("user", AttributeValue::Specific(user_set));
        enabled.add_dimension("resource", AttributeValue::All);

        let mut disabled = AbacRule::new("deny_bob");
        disabled.set_deny();
        let mut bob_set = HashSet::new();
        bob_set.insert(AttributeType::String("bob".into()));
        disabled.add_dimension("user", AttributeValue::Specific(bob_set));
        disabled.add_dimension("resource", AttributeValue::All);
        // not enabled

        let mut idx = AbacDenyIndex::try_build(&[enabled, disabled]).unwrap();

        let mut req_bob = AbacRequest::new();
        req_bob
            .add_attribute("user", AttributeType::String("bob".into()), vec![])
            .unwrap();
        req_bob
            .add_attribute("resource", AttributeType::String("db".into()), vec![])
            .unwrap();
        assert!(!idx.has_deny_match(&req_bob));
    }
}
