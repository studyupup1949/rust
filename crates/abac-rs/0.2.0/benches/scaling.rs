//! Scaling benchmarks for abac-rs.
//!
//! Measures performance across different rule counts to validate that we
//! achieve similar performance characteristics to hbac-rs.

use ahash::AHashSet as HashSet;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use abac_rs::{AbacPolicy, AbacRequest, AbacRule, AttributeType, AttributeValue, Decision};

/// Create a policy with N rules.
///
/// Rules follow a realistic pattern:
/// - 70% allow rules, 30% deny rules
/// - Mix of specific values and category=all
/// - Multi-dimensional (user, resource, action)
fn create_policy(rule_count: usize) -> AbacPolicy {
    let mut policy = AbacPolicy::new();

    for i in 0..rule_count {
        let mut rule = AbacRule::new(format!("rule-{}", i));

        // Vary the rule type
        if i % 10 < 3 {
            rule.set_deny();
        } else {
            rule.set_allow();
        }

        // User dimension
        if i % 5 == 0 {
            // 20% of rules: any user
            rule.add_dimension("user", AttributeValue::All);
        } else {
            // 80% of rules: specific users/groups
            let mut user_set = HashSet::new();
            user_set.insert(AttributeType::String(format!("user-{}", i % 100)));
            user_set.insert(AttributeType::String(format!("group:team-{}", i % 20)));
            rule.add_dimension("user", AttributeValue::Specific(user_set));
        }

        // Resource dimension
        if i % 7 == 0 {
            // ~14% of rules: any resource
            rule.add_dimension("resource", AttributeValue::All);
        } else {
            let mut resource_set = HashSet::new();
            resource_set.insert(AttributeType::String(format!("resource-{}", i % 50)));
            rule.add_dimension("resource", AttributeValue::Specific(resource_set));
        }

        // Action dimension
        if i % 3 == 0 {
            let mut action_set = HashSet::new();
            action_set.insert(AttributeType::String("read".into()));
            rule.add_dimension("action", AttributeValue::Specific(action_set));
        } else {
            rule.add_dimension("action", AttributeValue::All);
        }

        rule.enable();
        policy.add_rule(rule).unwrap();
    }

    policy
}

/// Create a matching request (should match some rules).
fn create_matching_request() -> AbacRequest {
    let mut request = AbacRequest::new();
    request
        .add_attribute(
            "user",
            AttributeType::String("user-42".into()),
            vec![AttributeType::String("group:team-2".into())],
        )
        .unwrap();
    request
        .add_attribute(
            "resource",
            AttributeType::String("resource-10".into()),
            vec![],
        )
        .unwrap();
    request
        .add_attribute("action", AttributeType::String("read".into()), vec![])
        .unwrap();
    request
}

/// Create a non-matching request (should not match any rules).
fn create_non_matching_request(i: usize) -> AbacRequest {
    let mut request = AbacRequest::new();
    request
        .add_attribute(
            "user",
            AttributeType::String(format!("unknown-user-{}", i)),
            vec![AttributeType::String(format!("unknown-group-{}", i))],
        )
        .unwrap();
    request
        .add_attribute(
            "resource",
            AttributeType::String(format!("unknown-resource-{}", i)),
            vec![],
        )
        .unwrap();
    request
        .add_attribute(
            "action",
            AttributeType::String("unknown-action".into()),
            vec![],
        )
        .unwrap();
    request
}

/// Benchmark: Cached evaluation (LRU warm).
///
/// Repeatedly evaluates the same request to measure cache hit performance.
fn bench_cached_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cached_evaluation");

    for rule_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(1));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_rules", rule_count)),
            rule_count,
            |b, &rule_count| {
                let mut policy = create_policy(rule_count);
                let request = create_matching_request();

                // Warm up the cache
                for _ in 0..100 {
                    policy.evaluate(&request);
                }

                b.iter(|| {
                    let decision = policy.evaluate(black_box(&request));
                    black_box(decision);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Uncached evaluation (unique requests).
///
/// Evaluates unique requests to measure full evaluation path with cache misses.
fn bench_uncached_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("uncached_evaluation");

    for rule_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(1));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_rules", rule_count)),
            rule_count,
            |b, &rule_count| {
                let mut policy = create_policy(rule_count);

                let mut counter = 0;
                b.iter(|| {
                    let request = create_non_matching_request(counter);
                    counter += 1;

                    let decision = policy.evaluate(black_box(&request));
                    black_box(decision);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Mixed workload (80% cached, 20% uncached).
///
/// Simulates realistic workload with cache hits and misses.
fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");

    for rule_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(1));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_rules", rule_count)),
            rule_count,
            |b, &rule_count| {
                let mut policy = create_policy(rule_count);
                let matching_request = create_matching_request();

                // Warm up
                for _ in 0..100 {
                    policy.evaluate(&matching_request);
                }

                let mut counter = 0;
                b.iter(|| {
                    let request = if counter % 5 == 0 {
                        // 20% uncached
                        create_non_matching_request(counter)
                    } else {
                        // 80% cached
                        matching_request.clone()
                    };
                    counter += 1;

                    let decision = policy.evaluate(black_box(&request));
                    black_box(decision);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Policy build time.
///
/// Measures time to build indexes (Bloom filters, composite index).
fn bench_policy_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("policy_build");

    for rule_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*rule_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_rules", rule_count)),
            rule_count,
            |b, &rule_count| {
                b.iter(|| {
                    let policy = create_policy(rule_count);
                    black_box(policy);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Constant result fast path.
///
/// Tests Layer 0 optimization (universal allow/deny).
fn bench_constant_result(c: &mut Criterion) {
    let mut group = c.benchmark_group("constant_result");

    let request = create_matching_request();

    group.bench_function("universal_allow", |b| {
        // Universal allow
        let mut policy = AbacPolicy::new();
        let mut allow_rule = AbacRule::new("allow-all");
        allow_rule.add_dimension("user", AttributeValue::All);
        allow_rule.add_dimension("resource", AttributeValue::All);
        allow_rule.add_dimension("action", AttributeValue::All);
        allow_rule.enable();
        policy.add_rule(allow_rule).unwrap();

        b.iter(|| {
            let decision = policy.evaluate(black_box(&request));
            assert_eq!(decision, Decision::Allow);
            black_box(decision);
        });
    });

    group.bench_function("universal_deny", |b| {
        // Universal deny
        let mut policy = AbacPolicy::new();
        let mut deny_rule = AbacRule::new("deny-all");
        deny_rule.add_dimension("user", AttributeValue::All);
        deny_rule.set_deny();
        deny_rule.enable();
        policy.add_rule(deny_rule).unwrap();

        b.iter(|| {
            let decision = policy.evaluate(black_box(&request));
            assert_eq!(decision, Decision::Deny);
            black_box(decision);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cached_evaluation,
    bench_uncached_evaluation,
    bench_mixed_workload,
    bench_policy_build,
    bench_constant_result,
);
criterion_main!(benches);
