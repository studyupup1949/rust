use super::*;
use crate::{AttributeType, AttributeValue};
use ahash::AHashSet as HashSet;

#[test]
fn test_policy_new() {
    let policy = AbacPolicy::new();
    assert_eq!(policy.rule_count(), 0);
}

#[test]
fn test_policy_add_remove_rule() {
    let mut policy = AbacPolicy::new();

    let mut rule = AbacRule::new("test-rule");
    rule.enable();
    policy.add_rule(rule).unwrap();

    assert_eq!(policy.rule_count(), 1);
    assert!(policy.get_rule("test-rule").is_some());

    let removed = policy.remove_rule("test-rule");
    assert!(removed.is_some());
    // Note: remove_rule disables the rule but keeps it in the vector for index stability
    assert_eq!(policy.rule_count(), 1);
    // Verify the rule is now disabled
    assert!(!policy.get_rule("test-rule").unwrap().is_enabled());
}

#[test]
fn test_policy_evaluate_no_rules() {
    let mut policy = AbacPolicy::new();
    let request = AbacRequest::new();

    // No rules -> deny by default
    assert_eq!(policy.evaluate(&request), Decision::Deny);
}

#[test]
fn test_policy_evaluate_allow_rule_matches() {
    let mut policy = AbacPolicy::new();

    // Create rule: user=alice can read resource=db-01
    let mut rule = AbacRule::new("allow-alice-db");
    let mut user_set = HashSet::new();
    user_set.insert(AttributeType::String("alice".into()));
    rule.add_dimension("user", AttributeValue::Specific(user_set));

    let mut resource_set = HashSet::new();
    resource_set.insert(AttributeType::String("db-01".into()));
    rule.add_dimension("resource", AttributeValue::Specific(resource_set));

    rule.enable();
    policy.add_rule(rule).unwrap();

    // Request: alice accessing db-01
    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();
    request
        .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
        .unwrap();

    assert_eq!(policy.evaluate(&request), Decision::Allow);
}

#[test]
fn test_policy_evaluate_allow_rule_no_match() {
    let mut policy = AbacPolicy::new();

    // Create rule: user=alice can read resource=db-01
    let mut rule = AbacRule::new("allow-alice-db");
    let mut user_set = HashSet::new();
    user_set.insert(AttributeType::String("alice".into()));
    rule.add_dimension("user", AttributeValue::Specific(user_set));

    let mut resource_set = HashSet::new();
    resource_set.insert(AttributeType::String("db-01".into()));
    rule.add_dimension("resource", AttributeValue::Specific(resource_set));

    rule.enable();
    policy.add_rule(rule).unwrap();

    // Request: bob accessing db-01 (user doesn't match)
    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("bob".into()), vec![])
        .unwrap();
    request
        .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
        .unwrap();

    assert_eq!(policy.evaluate(&request), Decision::Deny);
}

#[test]
fn test_policy_evaluate_deny_override() {
    let mut policy = AbacPolicy::new();

    // Allow rule: user=alice can access resource=db-01
    let mut allow_rule = AbacRule::new("allow-alice-db");
    let mut user_set = HashSet::new();
    user_set.insert(AttributeType::String("alice".into()));
    allow_rule.add_dimension("user", AttributeValue::Specific(user_set.clone()));

    let mut resource_set = HashSet::new();
    resource_set.insert(AttributeType::String("db-01".into()));
    allow_rule.add_dimension("resource", AttributeValue::Specific(resource_set.clone()));

    allow_rule.enable();
    policy.add_rule(allow_rule).unwrap();

    // Deny rule: user=alice is denied access to resource=db-01
    let mut deny_rule = AbacRule::new("deny-alice-db");
    deny_rule.add_dimension("user", AttributeValue::Specific(user_set));
    deny_rule.add_dimension("resource", AttributeValue::Specific(resource_set));
    deny_rule.set_deny();
    deny_rule.enable();
    policy.add_rule(deny_rule).unwrap();

    // Request: alice accessing db-01
    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();
    request
        .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
        .unwrap();

    // Deny rule should override allow rule
    assert_eq!(policy.evaluate(&request), Decision::Deny);
}

#[test]
fn test_policy_evaluate_with_groups() {
    let mut policy = AbacPolicy::new();

    // Create rule: group:engineers can read resource=db-01
    let mut rule = AbacRule::new("allow-engineers-db");
    let mut user_set = HashSet::new();
    user_set.insert(AttributeType::String("group:engineers".into()));
    rule.add_dimension("user", AttributeValue::Specific(user_set));

    let mut resource_set = HashSet::new();
    resource_set.insert(AttributeType::String("db-01".into()));
    rule.add_dimension("resource", AttributeValue::Specific(resource_set));

    rule.enable();
    policy.add_rule(rule).unwrap();

    // Request: alice in group:engineers accessing db-01
    let mut request = AbacRequest::new();
    request
        .add_attribute(
            "user",
            AttributeType::String("alice".into()),
            vec![AttributeType::String("group:engineers".into())],
        )
        .unwrap();
    request
        .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
        .unwrap();

    assert_eq!(policy.evaluate(&request), Decision::Allow);
}

#[test]
fn test_policy_evaluate_category_all() {
    let mut policy = AbacPolicy::new();

    // Create rule: any user can read any resource
    let mut rule = AbacRule::new("allow-all");
    rule.add_dimension("user", AttributeValue::All);
    rule.add_dimension("resource", AttributeValue::All);

    rule.enable();
    policy.add_rule(rule).unwrap();

    // Request: alice accessing db-01
    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();
    request
        .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
        .unwrap();

    assert_eq!(policy.evaluate(&request), Decision::Allow);
}

#[test]
fn test_policy_evaluate_disabled_rule() {
    let mut policy = AbacPolicy::new();

    // Create rule but leave it disabled
    let mut rule = AbacRule::new("allow-alice-db");
    let mut user_set = HashSet::new();
    user_set.insert(AttributeType::String("alice".into()));
    rule.add_dimension("user", AttributeValue::Specific(user_set));

    let mut resource_set = HashSet::new();
    resource_set.insert(AttributeType::String("db-01".into()));
    rule.add_dimension("resource", AttributeValue::Specific(resource_set));

    // DO NOT enable the rule
    policy.add_rule(rule).unwrap();

    // Request: alice accessing db-01
    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();
    request
        .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
        .unwrap();

    // Disabled rule should not match
    assert_eq!(policy.evaluate(&request), Decision::Deny);
}

#[test]
fn test_max_rules_default() {
    let policy = AbacPolicy::new();
    assert_eq!(policy.max_rules(), AbacPolicy::DEFAULT_MAX_RULES);
}

#[test]
fn test_max_rules_limit() {
    let mut policy = AbacPolicy::with_max_rules(2);
    assert_eq!(policy.max_rules(), 2);

    let mut r1 = AbacRule::new("r1");
    r1.enable();
    policy.add_rule(r1).unwrap();

    let mut r2 = AbacRule::new("r2");
    r2.enable();
    policy.add_rule(r2).unwrap();

    let mut r3 = AbacRule::new("r3");
    r3.enable();
    let err = policy.add_rule(r3).unwrap_err();
    assert!(matches!(
        err,
        PolicyError::TooManyRules {
            requested: 3,
            maximum: 2
        }
    ));
}

#[test]
fn test_max_rules_load_rules() {
    let mut policy = AbacPolicy::with_max_rules(2);

    let rules = vec![
        AbacRule::new("r1"),
        AbacRule::new("r2"),
        AbacRule::new("r3"),
    ];
    let err = policy.load_rules(rules).unwrap_err();
    assert!(matches!(
        err,
        PolicyError::TooManyRules {
            requested: 3,
            maximum: 2
        }
    ));

    let rules = vec![AbacRule::new("r1"), AbacRule::new("r2")];
    policy.load_rules(rules).unwrap();
    assert_eq!(policy.rule_count(), 2);
}

#[test]
fn test_max_rules_zero_means_unlimited() {
    let mut policy = AbacPolicy::with_max_rules(0);
    for i in 0..100 {
        policy.add_rule(AbacRule::new(format!("r{i}"))).unwrap();
    }
    assert_eq!(policy.rule_count(), 100);
}

#[test]
fn test_builder_basic() {
    let policy: AbacPolicy = AbacPolicy::builder()
        .max_rules(10_000)
        .cache_size(2048)
        .rules(vec![AbacRule::builder("allow-read")
            .dimension_all("user")
            .dimension_values("action", vec![AttributeType::String("read".into())])
            .enabled(true)
            .build()])
        .build()
        .unwrap();

    assert_eq!(policy.rule_count(), 1);
    assert_eq!(policy.max_rules(), 10_000);
}

#[test]
fn test_builder_with_matcher() {
    struct AlwaysMatch;
    impl crate::Matcher for AlwaysMatch {
        fn matches(
            &self,
            _rule_value: &AttributeValue,
            _request_value: &AttributeType,
            _request_groups: &[AttributeType],
        ) -> bool {
            true
        }
        fn supports_bloom_filter(&self) -> bool {
            false
        }
        fn name(&self) -> &str {
            "always"
        }
    }

    let mut policy: AbacPolicy = AbacPolicy::builder()
        .matcher("user", Box::new(AlwaysMatch))
        .rule(
            AbacRule::builder("allow-all-users")
                .dimension_values("user", vec![AttributeType::String("any".into())])
                .dimension_values("resource", vec![AttributeType::String("db-01".into())])
                .enabled(true)
                .build(),
        )
        .build()
        .unwrap();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("anyone".into()), vec![])
        .unwrap();
    request
        .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
        .unwrap();

    assert_eq!(policy.evaluate(&request), Decision::Allow);
}

#[test]
fn test_builder_max_rules_exceeded() {
    let result: Result<AbacPolicy, _> = AbacPolicy::builder()
        .max_rules(1)
        .rules(vec![AbacRule::new("r1"), AbacRule::new("r2")])
        .build();

    assert!(result.is_err());
}

#[test]
fn test_builder_single_rule_method() {
    let policy: AbacPolicy = AbacPolicy::builder()
        .rule(AbacRule::builder("r1").enabled(true).build())
        .rule(AbacRule::builder("r2").enabled(true).build())
        .build()
        .unwrap();

    assert_eq!(policy.rule_count(), 2);
}

#[test]
fn test_builder_default() {
    let policy: AbacPolicy = PolicyBuilder::default().build().unwrap();
    assert_eq!(policy.rule_count(), 0);
    assert_eq!(policy.max_rules(), AbacPolicy::DEFAULT_MAX_RULES);
}

#[test]
fn test_builder_debug() {
    let builder = PolicyBuilder::new();
    let debug = format!("{:?}", builder);
    assert!(debug.contains("PolicyBuilder"));
}

#[test]
fn test_evaluate_explained_all_deny_rules_collected() {
    let mut policy = AbacPolicy::new();

    let rule1 = AbacRule::builder("deny-1")
        .deny()
        .dimension_all("user")
        .enabled(true)
        .build();
    let rule2 = AbacRule::builder("deny-2")
        .deny()
        .dimension_all("user")
        .enabled(true)
        .build();
    policy.add_rule(rule1).unwrap();
    policy.add_rule(rule2).unwrap();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let result = policy.evaluate_explained(&request);
    assert_eq!(result.decision, Decision::Deny);
    assert_eq!(result.matched_rules.len(), 2);
    let names: Vec<&str> = result
        .matched_rules
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    assert!(names.contains(&"deny-1"));
    assert!(names.contains(&"deny-2"));
}

#[test]
fn test_evaluate_explained_first_allow_rule() {
    let mut policy = AbacPolicy::new();

    let rule1 = AbacRule::builder("allow-1")
        .dimension_all("user")
        .enabled(true)
        .build();
    let rule2 = AbacRule::builder("allow-2")
        .dimension_all("user")
        .enabled(true)
        .build();
    policy.add_rule(rule1).unwrap();
    policy.add_rule(rule2).unwrap();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let result = policy.evaluate_explained(&request);
    assert_eq!(result.decision, Decision::Allow);
    // Only one allow rule is returned (first match from candidate iteration)
    assert_eq!(result.matched_rules.len(), 1);
    assert!(result.matched_rules[0].rule_type == crate::RuleType::Allow);
    assert!(!result.matched_rules[0].temporal);
}

#[test]
fn test_evaluate_explained_default_deny_empty() {
    let mut policy = AbacPolicy::new();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let result = policy.evaluate_explained(&request);
    assert_eq!(result.decision, Decision::Deny);
    assert!(result.matched_rules.is_empty());
}

#[test]
fn test_evaluate_explained_temporal_deny() {
    let mut policy = AbacPolicy::new();

    let allow_rule = AbacRule::builder("allow-all")
        .dimension_all("user")
        .enabled(true)
        .build();
    policy.add_rule(allow_rule).unwrap();

    let deny_rule = AbacRule::builder("temporal-deny")
        .deny()
        .dimension_all("user")
        .enabled(true)
        .build();
    let now = acls_rs::permission::current_timestamp_millis();
    let temporal = TemporalAbacRule::new(deny_rule, Some(now - 1000), Some(now + 10000)).unwrap();
    policy.add_temporal_rule(temporal).unwrap();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let result = policy.evaluate_explained(&request);
    assert_eq!(result.decision, Decision::Deny);
    assert!(result
        .matched_rules
        .iter()
        .any(|r| r.temporal && r.name == "temporal-deny"));
}

#[test]
fn test_evaluate_explained_temporal_allow() {
    let mut policy = AbacPolicy::new();

    let allow_rule = AbacRule::builder("temporal-allow")
        .dimension_all("user")
        .enabled(true)
        .build();
    let now = acls_rs::permission::current_timestamp_millis();
    let temporal = TemporalAbacRule::new(allow_rule, Some(now - 1000), Some(now + 10000)).unwrap();
    policy.add_temporal_rule(temporal).unwrap();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let result = policy.evaluate_explained(&request);
    assert_eq!(result.decision, Decision::Allow);
    assert_eq!(result.matched_rules.len(), 1);
    assert!(result.matched_rules[0].temporal);
    assert_eq!(result.matched_rules[0].name, "temporal-allow");
}

#[test]
fn test_evaluate_and_evaluate_explained_agree() {
    let mut policy = AbacPolicy::new();

    let allow_rule = AbacRule::builder("allow-engineers")
        .dimension_values(
            "user",
            vec![AttributeType::String("group:engineers".into())],
        )
        .dimension_values("resource", vec![AttributeType::String("db-01".into())])
        .enabled(true)
        .build();
    let deny_rule = AbacRule::builder("deny-interns")
        .deny()
        .dimension_values("user", vec![AttributeType::String("group:interns".into())])
        .dimension_all("resource")
        .enabled(true)
        .build();
    policy.add_rule(allow_rule).unwrap();
    policy.add_rule(deny_rule).unwrap();

    // Test case 1: engineer accessing db-01 (should allow)
    let mut req = AbacRequest::new();
    req.add_attribute(
        "user",
        AttributeType::String("alice".into()),
        vec![AttributeType::String("group:engineers".into())],
    )
    .unwrap();
    req.add_attribute("resource", AttributeType::String("db-01".into()), vec![])
        .unwrap();

    let fast = policy.evaluate(&req);
    let explained = policy.evaluate_explained(&req);
    assert_eq!(fast, explained.decision);

    // Test case 2: intern accessing db-01 (should deny)
    let mut req2 = AbacRequest::new();
    req2.add_attribute(
        "user",
        AttributeType::String("bob".into()),
        vec![AttributeType::String("group:interns".into())],
    )
    .unwrap();
    req2.add_attribute("resource", AttributeType::String("db-01".into()), vec![])
        .unwrap();

    let fast2 = policy.evaluate(&req2);
    let explained2 = policy.evaluate_explained(&req2);
    assert_eq!(fast2, explained2.decision);

    // Test case 3: unknown user (should deny by default)
    let mut req3 = AbacRequest::new();
    req3.add_attribute("user", AttributeType::String("eve".into()), vec![])
        .unwrap();
    req3.add_attribute("resource", AttributeType::String("db-01".into()), vec![])
        .unwrap();

    let fast3 = policy.evaluate(&req3);
    let explained3 = policy.evaluate_explained(&req3);
    assert_eq!(fast3, explained3.decision);
}

#[test]
fn test_evaluate_explained_populates_rule_id() {
    let mut policy = AbacPolicy::new();

    let rule = AbacRule::builder("allow-read")
        .id("uuid-789")
        .dimension_all("user")
        .enabled(true)
        .build();
    policy.add_rule(rule).unwrap();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let result = policy.evaluate_explained(&request);
    assert_eq!(result.decision, Decision::Allow);
    assert_eq!(result.matched_rules[0].id, Some("uuid-789".to_string()));
    assert_eq!(result.matched_rules[0].name, "allow-read");
}

#[test]
fn test_evaluate_explained_rule_without_id() {
    let mut policy = AbacPolicy::new();

    let rule = AbacRule::builder("allow-read")
        .dimension_all("user")
        .enabled(true)
        .build();
    policy.add_rule(rule).unwrap();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let result = policy.evaluate_explained(&request);
    assert_eq!(result.matched_rules[0].id, None);
}

#[test]
fn test_mixed_matchers_bloom_filters_exact_dims() {
    struct AlwaysMatch;
    impl crate::Matcher for AlwaysMatch {
        fn matches(
            &self,
            _rule_value: &AttributeValue,
            _request_value: &AttributeType,
            _request_groups: &[AttributeType],
        ) -> bool {
            true
        }
        fn supports_bloom_filter(&self) -> bool {
            false
        }
        fn name(&self) -> &str {
            "always"
        }
    }

    let mut policy: AbacPolicy = AbacPolicy::builder()
        .matcher("custom_dim", Box::new(AlwaysMatch))
        .rules(vec![AbacRule::builder("allow-specific")
            .dimension_values("exact_dim", vec![AttributeType::String("allowed".into())])
            .dimension_values("custom_dim", vec![AttributeType::String("anything".into())])
            .enabled(true)
            .build()])
        .build()
        .unwrap();

    // Request with a value NOT in the exact_dim -- Bloom filter on exact_dim
    // should reject this without reaching sequential evaluation
    let mut req = AbacRequest::new();
    req.add_attribute(
        "exact_dim",
        AttributeType::String("not-allowed".into()),
        vec![],
    )
    .unwrap();
    req.add_attribute(
        "custom_dim",
        AttributeType::String("whatever".into()),
        vec![],
    )
    .unwrap();

    assert_eq!(policy.evaluate(&req), Decision::Deny);

    // Request with matching exact_dim -- should pass through Bloom filter
    // and reach custom matcher evaluation
    let mut req2 = AbacRequest::new();
    req2.add_attribute("exact_dim", AttributeType::String("allowed".into()), vec![])
        .unwrap();
    req2.add_attribute(
        "custom_dim",
        AttributeType::String("whatever".into()),
        vec![],
    )
    .unwrap();

    assert_eq!(policy.evaluate(&req2), Decision::Allow);
}

#[test]
fn test_all_custom_matchers_no_bloom() {
    struct NeverMatch;
    impl crate::Matcher for NeverMatch {
        fn matches(
            &self,
            _rule_value: &AttributeValue,
            _request_value: &AttributeType,
            _request_groups: &[AttributeType],
        ) -> bool {
            false
        }
        fn supports_bloom_filter(&self) -> bool {
            false
        }
        fn name(&self) -> &str {
            "never"
        }
    }

    let mut policy: AbacPolicy = AbacPolicy::builder()
        .matcher("dim_a", Box::new(NeverMatch))
        .matcher("dim_b", Box::new(NeverMatch))
        .rules(vec![AbacRule::builder("rule1")
            .dimension_values("dim_a", vec![AttributeType::String("x".into())])
            .dimension_values("dim_b", vec![AttributeType::String("y".into())])
            .enabled(true)
            .build()])
        .build()
        .unwrap();

    let mut req = AbacRequest::new();
    req.add_attribute("dim_a", AttributeType::String("x".into()), vec![])
        .unwrap();
    req.add_attribute("dim_b", AttributeType::String("y".into()), vec![])
        .unwrap();

    // Custom matcher always returns false, so this should deny
    assert_eq!(policy.evaluate(&req), Decision::Deny);
}

#[test]
fn test_no_custom_matchers_normal_behavior() {
    let mut policy: AbacPolicy = AbacPolicy::builder()
        .rules(vec![AbacRule::builder("allow-exact")
            .dimension_values("user", vec![AttributeType::String("alice".into())])
            .enabled(true)
            .build()])
        .build()
        .unwrap();

    let mut req = AbacRequest::new();
    req.add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();
    assert_eq!(policy.evaluate(&req), Decision::Allow);

    let mut req2 = AbacRequest::new();
    req2.add_attribute("user", AttributeType::String("bob".into()), vec![])
        .unwrap();
    assert_eq!(policy.evaluate(&req2), Decision::Deny);
}

#[test]
fn test_temporal_allow_overrides_regular_deny_explained() {
    let mut policy = AbacPolicy::new();
    let now = acls_rs::permission::current_timestamp_millis();

    let deny_rule = AbacRule::builder("deny-alice")
        .deny()
        .dimension_all("user")
        .enabled(true)
        .build();
    policy.add_rule(deny_rule).unwrap();

    let allow_rule = AbacRule::builder("temporal-allow-alice")
        .dimension_all("user")
        .enabled(true)
        .build();
    let temporal = TemporalAbacRule::new(allow_rule, Some(now - 1000), Some(now + 10000)).unwrap();
    policy.add_temporal_rule(temporal).unwrap();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let fast = policy.evaluate_at(&request, now);
    let explained = policy.evaluate_explained_at(&request, now);
    assert_eq!(fast, Decision::Allow);
    assert_eq!(explained.decision, Decision::Allow);
    assert_eq!(fast, explained.decision);
    assert!(explained
        .matched_rules
        .iter()
        .any(|r| r.name == "deny-alice" && !r.temporal));
    assert!(explained
        .matched_rules
        .iter()
        .any(|r| r.name == "temporal-allow-alice" && r.temporal));
}

#[test]
fn test_temporal_deny_beats_temporal_allow_explained() {
    let mut policy = AbacPolicy::new();
    let now = acls_rs::permission::current_timestamp_millis();

    let allow_rule = AbacRule::builder("regular-allow")
        .dimension_all("user")
        .enabled(true)
        .build();
    policy.add_rule(allow_rule).unwrap();

    let temporal_deny = AbacRule::builder("temporal-deny")
        .deny()
        .dimension_all("user")
        .enabled(true)
        .build();
    policy
        .add_temporal_rule(
            TemporalAbacRule::new(temporal_deny, Some(now - 1000), Some(now + 10000)).unwrap(),
        )
        .unwrap();

    let temporal_allow = AbacRule::builder("temporal-allow")
        .dimension_all("user")
        .enabled(true)
        .build();
    policy
        .add_temporal_rule(
            TemporalAbacRule::new(temporal_allow, Some(now - 1000), Some(now + 10000)).unwrap(),
        )
        .unwrap();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let fast = policy.evaluate_at(&request, now);
    let explained = policy.evaluate_explained_at(&request, now);
    assert_eq!(fast, Decision::Deny);
    assert_eq!(fast, explained.decision);
}

#[test]
fn test_regular_deny_without_temporal_allow_stays_deny_explained() {
    let mut policy = AbacPolicy::new();
    let now = acls_rs::permission::current_timestamp_millis();

    let deny_rule = AbacRule::builder("deny-all")
        .deny()
        .dimension_all("user")
        .enabled(true)
        .build();
    policy.add_rule(deny_rule).unwrap();

    let allow_rule = AbacRule::builder("allow-all")
        .dimension_all("user")
        .enabled(true)
        .build();
    policy.add_rule(allow_rule).unwrap();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let fast = policy.evaluate_at(&request, now);
    let explained = policy.evaluate_explained_at(&request, now);
    assert_eq!(fast, Decision::Deny);
    assert_eq!(fast, explained.decision);
}

#[test]
fn test_temporal_deny_overrides_regular_allow_explained() {
    let mut policy = AbacPolicy::new();
    let now = acls_rs::permission::current_timestamp_millis();

    let allow_rule = AbacRule::builder("regular-allow")
        .dimension_all("user")
        .enabled(true)
        .build();
    policy.add_rule(allow_rule).unwrap();

    let temporal_deny = AbacRule::builder("temporal-deny")
        .deny()
        .dimension_all("user")
        .enabled(true)
        .build();
    policy
        .add_temporal_rule(
            TemporalAbacRule::new(temporal_deny, Some(now - 1000), Some(now + 10000)).unwrap(),
        )
        .unwrap();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let fast = policy.evaluate_at(&request, now);
    let explained = policy.evaluate_explained_at(&request, now);
    assert_eq!(fast, Decision::Deny);
    assert_eq!(fast, explained.decision);
    assert!(explained
        .matched_rules
        .iter()
        .any(|r| r.name == "temporal-deny" && r.temporal));
}

#[test]
fn test_no_rules_default_deny_explained() {
    let mut policy = AbacPolicy::new();
    let now = acls_rs::permission::current_timestamp_millis();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let fast = policy.evaluate_at(&request, now);
    let explained = policy.evaluate_explained_at(&request, now);
    assert_eq!(fast, Decision::Deny);
    assert_eq!(fast, explained.decision);
    assert!(explained.matched_rules.is_empty());
}

#[test]
fn test_expired_temporal_allow_does_not_override_deny_explained() {
    let mut policy = AbacPolicy::new();
    let now = acls_rs::permission::current_timestamp_millis();

    let deny_rule = AbacRule::builder("deny-all")
        .deny()
        .dimension_all("user")
        .enabled(true)
        .build();
    policy.add_rule(deny_rule).unwrap();

    let allow_rule = AbacRule::builder("expired-allow")
        .dimension_all("user")
        .enabled(true)
        .build();
    policy
        .add_temporal_rule(
            TemporalAbacRule::new(allow_rule, Some(now - 10000), Some(now - 1000)).unwrap(),
        )
        .unwrap();

    let mut request = AbacRequest::new();
    request
        .add_attribute("user", AttributeType::String("alice".into()), vec![])
        .unwrap();

    let fast = policy.evaluate_at(&request, now);
    let explained = policy.evaluate_explained_at(&request, now);
    assert_eq!(fast, Decision::Deny);
    assert_eq!(fast, explained.decision);
}
