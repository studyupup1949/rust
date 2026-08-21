//! Composable rule evaluation pipeline.
//!
//! The pipeline connects sources, filters, and the policy into
//! a complete system for fetching, filtering, caching, and evaluating rules.

use crate::cache::{AcceptAllFilter, RuleFilter, RuleSource, RuleSourceError};
use crate::{AbacPolicy, AbacRequest, AbacRule, Decision, PolicyError};
use acls_rs::permission::current_timestamp_millis;

/// Errors that can occur during pipeline operations.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    /// Error from the rule source.
    #[error("rule source error: {0}")]
    Source(#[from] RuleSourceError),
    /// Error from the policy.
    #[error("policy error: {0}")]
    Policy(#[from] PolicyError),
}

/// A complete rule evaluation pipeline.
///
/// Combines source → filter → policy evaluation into a single component.
///
/// # Examples
///
/// ```rust
/// use abac_rs::cache::*;
/// use abac_rs::{AbacRule, AbacRequest, AttributeType};
///
/// let mut pipeline = RulePipeline::builder()
///     .with_filter(Box::new(ApplicabilityFilter::all_enabled()))
///     .build().unwrap();
///
/// // Load rules
/// let rule = AbacRule::builder("allow_read")
///     .dimension_all("action")
///     .enabled(true)
///     .build();
///
/// pipeline.load(vec![rule]).unwrap();
///
/// // Evaluate
/// let mut request = AbacRequest::new();
/// request.add_attribute("action", AttributeType::String("read".into()), vec![]).unwrap();
/// let result = pipeline.evaluate(&request);
///
/// assert!(result.is_allowed());
/// ```
pub struct RulePipeline {
    source: Option<Box<dyn RuleSource>>,
    filter: Box<dyn RuleFilter>,
    policy: AbacPolicy,
    last_update: u64,
}

impl std::fmt::Debug for RulePipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RulePipeline")
            .field("source", &self.source.as_ref().map(|_| "<RuleSource>"))
            .field("filter", &"<RuleFilter>")
            .field("policy", &self.policy)
            .field("last_update", &self.last_update)
            .finish()
    }
}

impl RulePipeline {
    /// Creates a new pipeline builder.
    pub fn builder() -> RulePipelineBuilder {
        RulePipelineBuilder::new()
    }

    /// Loads rules directly into the policy (bypass source).
    ///
    /// Filters rules before loading into the policy.
    ///
    /// # Errors
    ///
    /// Returns [`PipelineError::Policy`] if the rule count exceeds limits.
    pub fn load(&mut self, rules: Vec<AbacRule>) -> Result<(), PipelineError> {
        let filtered = self.filter.filter_cloned(&rules);
        self.policy.load_rules(filtered)?;
        self.last_update = current_timestamp_millis();
        Ok(())
    }

    /// Fetches all rules from the source and loads them.
    ///
    /// This operation is atomic: if loading fails, the old rules are preserved.
    ///
    /// # Errors
    ///
    /// Returns [`PipelineError::Source`] if fetching fails or [`PipelineError::Policy`]
    /// if the rule count exceeds limits.
    pub fn refresh(&mut self) -> Result<usize, PipelineError> {
        if let Some(source) = &mut self.source {
            let rules = source.fetch_all()?;
            let filtered = self.filter.filter_cloned(&rules);
            let count = filtered.len();

            // Backup current rules for rollback on failure
            let backup: Vec<AbacRule> = self.policy.rules().cloned().collect();

            // Clear and load new rules
            self.policy.clear();
            if let Err(e) = self.policy.load_rules(filtered) {
                // Restore backup on failure
                log::error!(
                    "Pipeline refresh failed, restoring {} old rules: {}",
                    backup.len(),
                    e
                );
                self.policy.clear();
                if let Err(restore_err) = self.policy.load_rules(backup) {
                    eprintln!(
                        "CRITICAL: failed to restore backup rules after refresh failure. \
                         Policy is now empty (all requests will be denied). \
                         Restore error: {}, original error: {}",
                        restore_err, e
                    );
                }
                return Err(e.into());
            }

            self.last_update = current_timestamp_millis();
            log::info!("Pipeline refresh succeeded, loaded {} rules", count);
            Ok(count)
        } else {
            Err(PipelineError::Source(RuleSourceError::Unavailable(
                "No source configured".to_string(),
            )))
        }
    }

    /// Performs an incremental update from the source.
    ///
    /// # Errors
    ///
    /// Returns [`PipelineError::Source`] if fetching fails.
    pub fn incremental_update(&mut self) -> Result<usize, PipelineError> {
        if let Some(source) = &mut self.source {
            let rules = source.fetch_updated_since(self.last_update)?;
            let filtered = self.filter.filter_cloned(&rules);
            let count = filtered.len();

            // Merge with existing rules
            for rule in filtered {
                // Remove old version if exists, add new version
                self.policy.remove_rule(&rule.name);
                self.policy.add_rule(rule)?;
            }

            self.last_update = current_timestamp_millis();
            Ok(count)
        } else {
            Err(PipelineError::Source(RuleSourceError::Unavailable(
                "No source configured".to_string(),
            )))
        }
    }

    /// Evaluates a request against loaded rules.
    ///
    /// # Design Note: Why Pipeline Delegates to Policy
    ///
    /// Unlike hbac-rs where pipeline evaluation was optimized to inline the hot path,
    /// abac-rs deliberately delegates to `policy.evaluate()` because:
    ///
    /// 1. **Complex evaluation logic**: ABAC evaluation involves dimension-specific
    ///    matchers, composite indexing, Bloom filters, and pluggable matching strategies.
    ///    Duplicating this logic in the pipeline would create a maintenance burden.
    ///
    /// 2. **Multi-layer caching**: The policy already implements a 4-layer evaluation
    ///    pipeline (constant-result fast path, LRU cache, Bloom filters, composite index).
    ///    The pipeline's role is rule lifecycle management, not evaluation optimization.
    ///
    /// 3. **Performance is adequate**: ABAC evaluation is fundamentally more complex
    ///    than HBAC's fixed three-dimension model. The delegation overhead is negligible
    ///    compared to the dimension matching and index lookup costs.
    ///
    /// For workloads where this delegation becomes a bottleneck, access the policy
    /// directly via `policy()` or `policy_mut()` to bypass the pipeline wrapper.
    pub fn evaluate(&mut self, request: &AbacRequest) -> Decision {
        self.policy.evaluate(request)
    }

    /// Gets the underlying policy for direct access.
    pub fn policy(&self) -> &AbacPolicy {
        &self.policy
    }

    /// Gets a mutable reference to the underlying policy.
    pub fn policy_mut(&mut self) -> &mut AbacPolicy {
        &mut self.policy
    }

    /// Returns the number of cached rules.
    pub fn rule_count(&self) -> usize {
        self.policy.rule_count()
    }

    /// Returns cache statistics.
    pub fn stats(&self) -> crate::CacheStats {
        self.policy.stats()
    }
}

/// Builder for creating a rule pipeline.
///
/// # Examples
///
/// ```rust
/// use abac_rs::cache::*;
///
/// let pipeline = RulePipeline::builder()
///     .with_source(Box::new(MemorySource::new(vec![])))
///     .with_filter(Box::new(AcceptAllFilter))
///     .with_cache_size(2048)
///     .build();
/// ```
pub struct RulePipelineBuilder {
    source: Option<Box<dyn RuleSource>>,
    filter: Option<Box<dyn RuleFilter>>,
    cache_size: Option<usize>,
    rules_capacity: Option<usize>,
}

impl std::fmt::Debug for RulePipelineBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RulePipelineBuilder")
            .field("source", &self.source.as_ref().map(|_| "<RuleSource>"))
            .field("filter", &self.filter.as_ref().map(|_| "<RuleFilter>"))
            .field("cache_size", &self.cache_size)
            .field("rules_capacity", &self.rules_capacity)
            .finish()
    }
}

impl RulePipelineBuilder {
    /// Creates a new pipeline builder.
    pub fn new() -> Self {
        Self {
            source: None,
            filter: None,
            cache_size: None,
            rules_capacity: None,
        }
    }

    /// Sets the rule source.
    pub fn with_source(mut self, source: Box<dyn RuleSource>) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the rule filter.
    pub fn with_filter(mut self, filter: Box<dyn RuleFilter>) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Sets the LRU cache size for the policy.
    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.cache_size = Some(size);
        self
    }

    /// Sets the initial capacity for rules HashMap.
    pub fn with_rules_capacity(mut self, capacity: usize) -> Self {
        self.rules_capacity = Some(capacity);
        self
    }

    /// Builds the pipeline.
    pub fn build(self) -> Result<RulePipeline, PolicyError> {
        let filter = self.filter.unwrap_or_else(|| Box::new(AcceptAllFilter));

        let policy = match (self.rules_capacity, self.cache_size) {
            (Some(rules_cap), Some(cache_size)) => {
                AbacPolicy::with_capacity(rules_cap, cache_size)?
            }
            (Some(rules_cap), None) => AbacPolicy::with_capacity(rules_cap, 1024)?,
            (None, Some(cache_size)) => AbacPolicy::with_cache_size(cache_size)?,
            (None, None) => AbacPolicy::new(),
        };

        Ok(RulePipeline {
            source: self.source,
            filter,
            policy,
            last_update: 0,
        })
    }
}

impl Default for RulePipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AttributeType, AttributeValue};

    #[test]
    fn test_pipeline_basic() {
        let mut pipeline = RulePipeline::builder().build().unwrap();

        let mut rule = AbacRule::new("test");
        rule.add_dimension("action", AttributeValue::All);
        rule.enable();

        pipeline.load(vec![rule]).unwrap();

        let mut request = AbacRequest::new();
        request
            .add_attribute("action", AttributeType::String("read".into()), vec![])
            .unwrap();

        let result = pipeline.evaluate(&request);
        assert!(result.is_allowed());
    }

    #[test]
    fn test_pipeline_with_filter() {
        use crate::cache::ApplicabilityFilter;

        let mut pipeline = RulePipeline::builder()
            .with_filter(Box::new(ApplicabilityFilter::all_enabled()))
            .build()
            .unwrap();

        // Enabled rule should be loaded
        let mut rule1 = AbacRule::new("enabled");
        rule1.add_dimension("action", AttributeValue::All);
        rule1.enable();

        // Disabled rule should be filtered out
        let mut rule2 = AbacRule::new("disabled");
        rule2.add_dimension("action", AttributeValue::All);
        // Not enabled

        pipeline.load(vec![rule1, rule2]).unwrap();

        // Only one rule should be loaded
        assert_eq!(pipeline.rule_count(), 1);
    }

    #[test]
    fn test_pipeline_with_source() {
        use crate::cache::MemorySource;

        let mut rule = AbacRule::new("from-source");
        rule.add_dimension("action", AttributeValue::All);
        rule.enable();

        let source = MemorySource::new(vec![rule]);

        let mut pipeline = RulePipeline::builder()
            .with_source(Box::new(source))
            .build()
            .unwrap();

        let count = pipeline.refresh().unwrap();
        assert_eq!(count, 1);
        assert_eq!(pipeline.rule_count(), 1);
    }
}
