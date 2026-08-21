//! ABAC policy types and evaluation.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;

use lru::LruCache;

#[cfg(feature = "bloom")]
use crate::cache::DimensionBloom;
use crate::cache::RequestKey;
use crate::index::CompositeIndex;
use crate::temporal::TemporalAbacRule;
use crate::{AbacRequest, AbacRule, Decision, Matcher};
use acls_rs::permission::Timestamp;

// PolicyError is now provided by acls_rs::policy::PolicyError
// Re-export for backward compatibility
pub use acls_rs::policy::PolicyError;

// CacheLock is now provided by acls_rs::sync::SyncStrategy
// Re-export for backward compatibility
pub use acls_rs::sync::SyncStrategy as CacheLock;

/// Cache statistics for monitoring.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CacheStats {
    /// Number of rules in the policy
    pub rule_count: usize,
    /// Number of enabled rules
    pub enabled_rule_count: usize,
    /// Estimated memory usage in bytes
    pub memory_bytes: usize,
    /// Cache capacity (maximum entries)
    pub cache_capacity: usize,
    /// Current number of entries in cache
    pub cache_entries: usize,
}

impl CacheStats {
    /// Returns the cache fill rate (0.0 to 1.0).
    pub fn cache_fill_rate(&self) -> f64 {
        if self.cache_capacity == 0 {
            0.0
        } else {
            self.cache_entries as f64 / self.cache_capacity as f64
        }
    }
}

/// ABAC policy containing rules and evaluation logic.
///
/// The policy evaluates requests against rules using a multi-layer optimization
/// pipeline for high performance.
///
/// Generic over the cache locking strategy `L` to support both single-threaded
/// ([`RefCell`]) and multi-threaded ([`Mutex`]) use cases.
pub struct AbacPolicyCore<L: CacheLock<LruCache<RequestKey, Decision>>> {
    /// Rules stored in a dense vector for fast indexed access
    rules: Vec<AbacRule>,

    /// Name → index mapping for rule lookup by name
    rule_index: HashMap<String, usize>,

    /// Temporal rules
    temporal_rules: Vec<TemporalAbacRule>,

    /// Per-dimension matchers
    matchers: HashMap<String, Box<dyn Matcher>>,

    /// Layer 0: Constant result (if all rules collapse to single decision)
    constant_result: Option<Decision>,

    /// Layer 1: LRU memoization cache (1,024 entries)
    cache: L,

    /// Layer 2: Bloom filters per dimension (for exact matchers only)
    #[cfg(feature = "bloom")]
    bloom_filters: HashMap<String, DimensionBloom>,

    /// Layer 3: Composite index for fast candidate selection
    composite_index: CompositeIndex,

    /// Layer 3.5: Pre-compiled evaluator (if all rules have consistent dimensions)
    compiled_evaluator: Option<crate::compiled::CompiledEvaluator>,

    /// Bitmap-based deny index for fast deny rule matching when universal allow exists.
    deny_index: Option<crate::cache::deny_index::AbacDenyIndex>,

    /// Fast-path: whether a universal allow rule exists (all dimensions = All)
    has_universal_allow: bool,

    /// Whether the policy has been modified since last index build
    dirty: bool,

    /// Maximum number of rules allowed (0 = no limit)
    max_rules: usize,
}

/// Thread-safe ABAC policy using [`Mutex`] for cache access.
///
/// Safe to share across threads via `Arc<AbacPolicy>`.
pub type AbacPolicy = AbacPolicyCore<Mutex<LruCache<RequestKey, Decision>>>;

/// Single-threaded ABAC policy using [`RefCell`] for zero-overhead cache access.
///
/// Compile-time `!Sync` — cannot be shared across threads.
/// Use this when evaluation happens on a single thread (e.g. benchmarks, embedded systems).
pub type AbacPolicyLocal = AbacPolicyCore<RefCell<LruCache<RequestKey, Decision>>>;

impl<L: CacheLock<LruCache<RequestKey, Decision>>> acls_rs::policy::RuleLimitedPolicy
    for AbacPolicyCore<L>
{
    fn max_limit(&self) -> usize {
        self.max_rules
    }

    fn current_count(&self) -> usize {
        self.rules.len()
    }
}

impl<L: CacheLock<LruCache<RequestKey, Decision>>> std::fmt::Debug for AbacPolicyCore<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let matcher_count = self.matchers.len();
        let mut debug_struct = f.debug_struct("AbacPolicy");
        debug_struct
            .field("rules", &self.rules)
            .field("matchers", &format_args!("<{} matchers>", matcher_count))
            .field("constant_result", &self.constant_result)
            .field("cache", &"<LRU cache>")
            .field(
                "bloom_filters",
                #[cfg(feature = "bloom")]
                &format_args!("<{} blooms>", self.bloom_filters.len()),
                #[cfg(not(feature = "bloom"))]
                &format_args!("<disabled>"),
            )
            .field("composite_index", &self.composite_index)
            .field("has_universal_allow", &self.has_universal_allow)
            .field("dirty", &self.dirty);

        debug_struct.field(
            "temporal_rules",
            &format_args!("<{} temporal>", self.temporal_rules.len()),
        );

        debug_struct.finish()
    }
}

impl<L: CacheLock<LruCache<RequestKey, Decision>>> AbacPolicyCore<L> {
    /// Default LRU cache size (1,024 entries, like hbac-rs)
    const DEFAULT_CACHE_SIZE: usize = 1024;

    /// Default initial capacity for rules HashMap
    const DEFAULT_RULES_CAPACITY: usize = 100;

    /// Default initial capacity for matchers HashMap
    const DEFAULT_MATCHERS_CAPACITY: usize = 10;

    /// Default maximum number of rules allowed in a policy (DoS protection).
    pub const DEFAULT_MAX_RULES: usize = 1_000_000;

    /// Create a new empty policy with default cache size and default rule limit.
    pub fn new() -> Self {
        Self::with_cache_size(Self::DEFAULT_CACHE_SIZE).expect("default capacity is valid")
    }

    /// Create a new empty policy with a custom rule limit.
    ///
    /// # Arguments
    ///
    /// * `max_rules` - Maximum number of rules allowed (0 = no limit)
    pub fn with_max_rules(max_rules: usize) -> Self {
        let mut policy = Self::new();
        policy.max_rules = max_rules;
        policy
    }

    /// Returns the maximum number of rules allowed (0 = no limit).
    pub fn max_rules(&self) -> usize {
        self.max_rules
    }

    /// Create a new policy with a custom cache size.
    ///
    /// # Arguments
    ///
    /// * `cache_size` - Number of LRU cache entries (recommended: 512-4096)
    pub fn with_cache_size(cache_size: usize) -> Result<Self, PolicyError> {
        Self::with_capacity(Self::DEFAULT_RULES_CAPACITY, cache_size)
    }

    /// Create a new policy with custom rule capacity and cache size.
    ///
    /// Pre-allocates space for the expected number of rules to avoid reallocations
    /// during policy loading.
    ///
    /// # Arguments
    ///
    /// * `rules_capacity` - Expected number of rules (e.g., 100, 1000, 10000)
    /// * `cache_size` - Number of LRU cache entries (recommended: 512-4096)
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError::TooManyRules`] if `rules_capacity` exceeds the
    /// configured maximum.
    pub fn with_capacity(rules_capacity: usize, cache_size: usize) -> Result<Self, PolicyError> {
        let max_rules = Self::DEFAULT_MAX_RULES;
        // Check capacity against default max (static method, can't use trait here)
        if max_rules > 0 && rules_capacity > max_rules {
            return Err(PolicyError::TooManyRules {
                requested: rules_capacity,
                maximum: max_rules,
            });
        }

        let cache_size = cache_size.max(1);
        let cache = LruCache::new(std::num::NonZeroUsize::new(cache_size).unwrap());
        Ok(Self {
            rules: Vec::with_capacity(rules_capacity),
            rule_index: HashMap::with_capacity(rules_capacity),
            temporal_rules: Vec::new(),
            matchers: HashMap::with_capacity(Self::DEFAULT_MATCHERS_CAPACITY),
            constant_result: None,
            cache: L::new(cache),
            #[cfg(feature = "bloom")]
            bloom_filters: HashMap::new(),
            composite_index: CompositeIndex::new(),
            compiled_evaluator: None,
            deny_index: None,
            has_universal_allow: false,
            dirty: true,
            max_rules,
        })
    }

    /// Load multiple rules into the policy at once.
    ///
    /// More efficient than calling `add_rule` in a loop as it only marks
    /// dirty once and invalidates the cache once.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError::TooManyRules`] if the total number of rules
    /// (existing + new) would exceed the configured maximum.
    pub fn load_rules(&mut self, rules: Vec<AbacRule>) -> Result<(), PolicyError> {
        use acls_rs::policy::RuleLimitedPolicy;
        // Check if the incoming batch alone exceeds the limit
        self.check_total(rules.len())?;
        // Check if adding the batch to existing rules would exceed the limit
        self.check_limit(rules.len())?;

        for rule in rules {
            let index = self.rules.len();
            self.rule_index.insert(rule.name.clone(), index);
            self.rules.push(rule);
        }
        self.dirty = true;
        self.invalidate_cache();
        Ok(())
    }

    /// Add a rule to the policy.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError::TooManyRules`] if adding this rule would exceed
    /// the configured maximum.
    pub fn add_rule(&mut self, rule: AbacRule) -> Result<(), PolicyError> {
        use acls_rs::policy::RuleLimitedPolicy;
        self.check_limit(1)?;
        let index = self.rules.len();
        self.rule_index.insert(rule.name.clone(), index);
        self.rules.push(rule);
        self.dirty = true;
        self.invalidate_cache();
        Ok(())
    }

    /// Add a temporal rule to the policy.
    ///
    /// Temporal rules are evaluated at the current time or at a specific timestamp.
    pub fn add_temporal_rule(&mut self, rule: TemporalAbacRule) {
        self.temporal_rules.push(rule);
        self.dirty = true;
        self.invalidate_cache();
    }

    /// Remove a rule from the policy.
    ///
    /// Note: This disables the rule instead of removing it to maintain index stability.
    /// Disabled rules are not evaluated.
    pub fn remove_rule(&mut self, name: &str) -> Option<AbacRule> {
        if let Some(&idx) = self.rule_index.get(name) {
            let was_enabled = self.rules[idx].is_enabled();
            if was_enabled {
                self.rules[idx].disable();
                let cloned = self.rules[idx].clone();
                self.dirty = true;
                self.invalidate_cache();
                Some(cloned)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Enable a rule by name.
    ///
    /// Returns `true` if the rule was found and enabled, `false` if not found.
    pub fn enable_rule(&mut self, name: &str) -> bool {
        if let Some(&idx) = self.rule_index.get(name) {
            self.rules[idx].enable();
            self.dirty = true;
            self.invalidate_cache();
            true
        } else {
            false
        }
    }

    /// Disable a rule by name.
    ///
    /// Returns `true` if the rule was found and disabled, `false` if not found.
    pub fn disable_rule(&mut self, name: &str) -> bool {
        if let Some(&idx) = self.rule_index.get(name) {
            self.rules[idx].disable();
            self.dirty = true;
            self.invalidate_cache();
            true
        } else {
            false
        }
    }

    /// Clear the LRU cache.
    fn invalidate_cache(&self) {
        self.cache.with(|cache| cache.clear());
    }

    /// Build optimization indexes (Bloom filters, composite index, constant result).
    ///
    /// Called lazily on first evaluation after rules change.
    fn build_indexes(&mut self) {
        if !self.dirty {
            return;
        }

        // Detect constant result
        self.constant_result = self.detect_constant_result();

        // Detect universal allow rule (all dimensions = All)
        // Must be done before building index to enable deny-only optimization
        self.has_universal_allow = self.rules.iter().any(|r| {
            r.is_enabled()
                && r.is_allow()
                && !r.dimensions.is_empty()
                && r.dimensions
                    .values()
                    .all(|v| matches!(v, crate::AttributeValue::All))
        });

        // Try to build compiled evaluator (if all rules have consistent dimensions).
        // Skip when custom matchers are registered — the compiled evaluator uses
        // exact set membership and would bypass registered matchers.
        self.compiled_evaluator = if self.matchers.is_empty() {
            crate::compiled::CompiledEvaluator::try_build(&self.rules)
        } else {
            None
        };

        // Build bitmap-based deny index when universal allow exists.
        // Skip when custom matchers are registered — the deny index uses exact
        // value lookups and would bypass registered matchers.
        self.deny_index = if self.has_universal_allow && self.matchers.is_empty() {
            crate::cache::deny_index::AbacDenyIndex::try_build(&self.rules)
        } else {
            None
        };

        // Build composite index (fallback if compiled evaluator not available)
        // If universal allow exists, only index deny rules (allow rules won't be checked)
        // Dimensions with custom matchers are treated as opaque — their values
        // cannot be used for exact-match candidate pruning.
        let opaque_dims: std::collections::HashSet<String> =
            self.matchers.keys().cloned().collect();
        self.composite_index.build_from_rules_with_opaque_dims(
            &self.rules,
            self.has_universal_allow,
            &opaque_dims,
        );

        // Build Bloom filters for each dimension
        #[cfg(feature = "bloom")]
        self.bloom_filters.clear();
        let enabled_rules: Vec<&AbacRule> = self.rules.iter().filter(|r| r.is_enabled()).collect();

        if !enabled_rules.is_empty() {
            // Collect all dimensions across rules
            let mut dimensions = std::collections::HashSet::new();
            for rule in &enabled_rules {
                for dim in rule.dimension_names() {
                    dimensions.insert(dim.to_string());
                }
            }

            // Build Bloom filter for each dimension
            #[cfg(feature = "bloom")]
            for dimension in dimensions {
                // Only build Bloom filter if the matcher supports it
                let supports_bloom = self
                    .matchers
                    .get(&dimension)
                    .map(|m| m.supports_bloom_filter())
                    .unwrap_or(true); // Default ExactMatcher supports Bloom

                if supports_bloom {
                    let bloom = DimensionBloom::from_rules(&dimension, &enabled_rules, 0.01);
                    self.bloom_filters.insert(dimension, bloom);
                }
            }
        }

        self.dirty = false;
    }

    /// Detect if all rules collapse to a single constant decision.
    fn detect_constant_result(&self) -> Option<Decision> {
        let enabled: Vec<_> = self.rules.iter().filter(|r| r.is_enabled()).collect();

        if enabled.is_empty() {
            return Some(Decision::Deny); // No rules = deny
        }

        // Check if all rules are deny
        if enabled.iter().all(|r| r.is_deny()) {
            return Some(Decision::Deny);
        }

        // Check if there's a universal allow with no deny rules
        let has_universal_allow = enabled
            .iter()
            .any(|r| r.is_allow() && r.dimensions.values().all(|v| v.is_all()));

        let has_any_deny = enabled.iter().any(|r| r.is_deny());

        if has_universal_allow && !has_any_deny {
            return Some(Decision::Allow);
        }

        None // No constant result
    }

    /// Get a rule by name.
    pub fn get_rule(&self, name: &str) -> Option<&AbacRule> {
        self.rule_index.get(name).map(|&idx| &self.rules[idx])
    }

    /// Get all rules in the policy.
    ///
    /// Returns an iterator over all rules (enabled and disabled).
    pub fn rules(&self) -> impl Iterator<Item = &AbacRule> {
        self.rules.iter()
    }

    /// Get the number of rules in the policy.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Get cache statistics for monitoring and observability.
    pub fn stats(&self) -> CacheStats {
        let enabled_rule_count = self.rules.iter().filter(|r| r.is_enabled()).count();

        // Estimate memory usage
        let rules_bytes = self.rules.len() * std::mem::size_of::<AbacRule>();
        let (cache_capacity, cache_entries) =
            self.cache.with(|cache| (cache.cap().get(), cache.len()));
        let cache_bytes = cache_entries * std::mem::size_of::<(RequestKey, Decision)>();
        #[cfg(feature = "bloom")]
        let bloom_bytes: usize = self.bloom_filters.values().map(|_| 1024).sum(); // Rough estimate
        #[cfg(not(feature = "bloom"))]
        let bloom_bytes: usize = 0;

        CacheStats {
            rule_count: self.rules.len(),
            enabled_rule_count,
            memory_bytes: rules_bytes + cache_bytes + bloom_bytes,
            cache_capacity,
            cache_entries,
        }
    }

    /// Clear all rules from the policy.
    pub fn clear(&mut self) {
        self.rules.clear();
        self.rule_index.clear();
        self.deny_index = None;
        self.dirty = true;
        self.invalidate_cache();
    }

    /// Register a matcher for a specific dimension.
    ///
    /// If no matcher is registered for a dimension, the default `ExactMatcher`
    /// will be used.
    pub fn register_matcher(&mut self, dimension: impl Into<String>, matcher: Box<dyn Matcher>) {
        self.matchers.insert(dimension.into(), matcher);
        self.dirty = true;
    }

    /// Evaluate a request against the policy.
    ///
    /// Returns `Decision::Allow` if at least one allow rule matches and no deny
    /// rules match. Returns `Decision::Deny` otherwise.
    ///
    /// # Optimization Layers (Same as hbac-rs)
    ///
    /// 0. **Constant Result**: If all rules collapse to one decision (~15 ns)
    /// 1. **LRU Cache**: Check if this exact request was recently evaluated (sub-μs)
    /// 2. **Bloom Filters**: Fast negative check (O(1) guaranteed rejection, ~50 ns)
    /// 3. **Composite Index**: Find candidate rules via dimension intersection (O(log n))
    /// 4. **Sequential Evaluation**: Check candidates only (deny-override semantics)
    ///
    /// # Evaluation Order (Deny-Override Semantics)
    ///
    /// 1. Check all enabled deny rules - if any match, return Deny immediately
    /// 2. Check all enabled allow rules - if any match, return Allow
    /// 3. If no rules match, return Deny (secure by default)
    ///
    /// # Temporal Rules
    ///
    /// If temporal rules are present, they are evaluated at the current time.
    /// When no temporal rules exist, this method operates efficiently without
    /// timestamp overhead.
    pub fn evaluate(&mut self, request: &AbacRequest) -> Decision {
        use acls_rs::permission::current_timestamp_millis;
        self.evaluate_at(request, current_timestamp_millis())
    }

    /// Evaluate a request at a specific timestamp.
    ///
    /// This method is useful for:
    /// - Testing temporal rules with specific times
    /// - Batch processing of historical requests
    /// - Simulating future access control decisions
    ///
    /// When no temporal rules exist, the timestamp parameter is unused and
    /// this behaves identically to `evaluate()`.
    ///
    /// Temporal rules are evaluated separately without modifying the policy state,
    /// preventing cache invalidation and state corruption.
    pub fn evaluate_at(&mut self, request: &AbacRequest, timestamp: Timestamp) -> Decision {
        // Find valid temporal rules at this timestamp (clone to avoid borrow conflicts)
        let valid_temporal: Vec<AbacRule> = self
            .temporal_rules
            .iter()
            .filter(|tr| tr.is_valid_at(timestamp))
            .map(|tr| tr.rule.clone())
            .collect();

        // If no temporal rules are valid, use normal evaluation
        if valid_temporal.is_empty() {
            return self.evaluate_internal(request);
        }

        // Evaluate regular rules first
        let regular_decision = self.evaluate_internal(request);

        // Check temporal deny rules first (deny-override semantics)
        for temporal_rule in &valid_temporal {
            if temporal_rule.is_enabled()
                && temporal_rule.is_deny()
                && self.rule_matches(temporal_rule, request)
            {
                return Decision::Deny;
            }
        }

        // If regular decision is deny, check temporal allow rules
        if regular_decision == Decision::Deny {
            for temporal_rule in &valid_temporal {
                if temporal_rule.is_enabled()
                    && temporal_rule.is_allow()
                    && self.rule_matches(temporal_rule, request)
                {
                    return Decision::Allow;
                }
            }
        }

        // Return regular decision (temporal rules didn't override it)
        regular_decision
    }

    /// Internal evaluation without temporal rule handling.
    fn evaluate_internal(&mut self, request: &AbacRequest) -> Decision {
        // Build indexes if dirty
        self.build_indexes();

        // Layer 0: Constant result fast path
        if let Some(decision) = self.constant_result {
            return decision;
        }

        // Layer 1: Bitmap deny index (skip cache — O(dims) lookups + bitmap AND)
        if let Some(ref mut deny_index) = self.deny_index {
            return if deny_index.has_deny_match(request) {
                Decision::Deny
            } else {
                Decision::Allow
            };
        }

        // Layer 2: Compiled evaluator (skip cache — linear deny scan)
        if let Some(ref compiled) = self.compiled_evaluator {
            return compiled.evaluate(request);
        }

        // Layer 3: Check LRU cache (only when compiled/deny_index unavailable)
        let cache_key = RequestKey::from(request);
        if let Some(cached) = self.cache.with(|cache| cache.get(&cache_key).copied()) {
            return cached;
        }

        // Layer 4: Bloom filter pre-screening
        #[cfg(feature = "bloom")]
        for (dimension, bloom) in &self.bloom_filters {
            if let Some((value, groups)) = request.get_attribute(dimension) {
                if bloom.definitely_no_match(value, groups) {
                    // Definitely no match - cache and return Deny
                    self.cache
                        .with(|cache| cache.put(cache_key, Decision::Deny));
                    return Decision::Deny;
                }
            }
        }

        // Layer 3: Sequential evaluation (cache miss + Bloom says "maybe")
        let decision = self.evaluate_uncached(request);

        // Store in cache
        self.cache.with(|cache| cache.put(cache_key, decision));

        decision
    }

    /// Evaluate without cache lookup (always checks rules).
    ///
    /// Note: Compiled evaluator is called from evaluate_internal directly,
    /// so this only handles composite index + sequential evaluation fallback.
    fn evaluate_uncached(&self, request: &AbacRequest) -> Decision {
        // Use composite index to find candidate rules
        let candidates = if !self.composite_index.is_empty() {
            self.composite_index.find_candidates(request)
        } else {
            // No index - check all rules (fallback)
            (0..self.rules.len()).collect()
        };

        // Phase 1: Check deny rules first (deny-override semantics)
        // Note: Composite index only includes enabled rules, so no need to check is_enabled()
        for &rule_idx in &candidates {
            let rule = &self.rules[rule_idx];

            if rule.is_deny() && self.rule_matches(rule, request) {
                return Decision::Deny;
            }
        }

        // Fast path: if universal allow exists and no deny matched, return allow
        // This skips checking all allow rules when a catch-all rule exists
        if self.has_universal_allow {
            return Decision::Allow;
        }

        // Phase 2: Check allow rules (only if no universal allow)
        for &rule_idx in &candidates {
            let rule = &self.rules[rule_idx];

            if rule.is_allow() && self.rule_matches(rule, request) {
                return Decision::Allow;
            }
        }

        // Phase 3: No matching rules -> deny by default
        Decision::Deny
    }

    /// Check if a rule matches a request across all dimensions.
    #[inline]
    fn rule_matches(&self, rule: &AbacRule, request: &AbacRequest) -> bool {
        // Fast path: if no custom matchers, inline ExactMatcher logic
        if self.matchers.is_empty() {
            for (dimension, rule_value) in &rule.dimensions {
                let Some((request_value, request_groups)) = request.get_attribute(dimension) else {
                    return false;
                };

                // Inline ExactMatcher logic with optimized checks
                match rule_value {
                    crate::AttributeValue::All => continue,
                    crate::AttributeValue::Specific(set) => {
                        // Fast path: check primary value first
                        if set.contains(request_value) {
                            continue;
                        }
                        // Slow path: check groups only if needed
                        if request_groups.is_empty()
                            || !request_groups.iter().any(|g| set.contains(g))
                        {
                            return false;
                        }
                    }
                }
            }
            return true;
        }

        // Slow path: use custom matchers
        for (dimension, rule_value) in &rule.dimensions {
            let Some((request_value, request_groups)) = request.get_attribute(dimension) else {
                return false;
            };

            let matcher = self
                .matchers
                .get(dimension.as_str())
                .map(|m| m.as_ref())
                .unwrap_or(&crate::ExactMatcher as &dyn Matcher);

            if !matcher.matches(rule_value, request_value, request_groups) {
                return false;
            }
        }

        true
    }
}

impl<L: CacheLock<LruCache<RequestKey, Decision>>> Default for AbacPolicyCore<L> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
}
