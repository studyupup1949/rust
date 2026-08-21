//! Rule filtering based on applicability criteria.
//!
//! Filters determine which rules should be cached based on criteria like
//! whether they apply to specific dimension values. This minimizes memory usage by
//! only caching relevant rules.

use crate::{AbacRule, AttributeType, AttributeValue};

/// Trait for filtering rules based on applicability criteria.
///
/// Filters can be composed to create complex filtering logic.
pub trait RuleFilter {
    /// Returns `true` if the rule should be cached.
    fn should_cache(&self, rule: &AbacRule) -> bool;

    /// Filters a collection of rules.
    fn filter<'a>(&self, rules: &'a [AbacRule]) -> Vec<&'a AbacRule> {
        rules.iter().filter(|r| self.should_cache(r)).collect()
    }

    /// Filters and clones rules.
    fn filter_cloned(&self, rules: &[AbacRule]) -> Vec<AbacRule> {
        rules
            .iter()
            .filter(|r| self.should_cache(r))
            .cloned()
            .collect()
    }
}

/// Filter that accepts all rules.
///
/// # Examples
///
/// ```rust
/// use abac_rs::cache::{RuleFilter, AcceptAllFilter};
/// use abac_rs::AbacRule;
///
/// let filter = AcceptAllFilter;
/// let rule = AbacRule::builder("test").build();
///
/// assert!(filter.should_cache(&rule));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AcceptAllFilter;

impl RuleFilter for AcceptAllFilter {
    fn should_cache(&self, _rule: &AbacRule) -> bool {
        true
    }
}

/// Filter that caches rules applicable to specific dimension values.
///
/// Caches rules if:
/// - Dimension category is "all", OR
/// - Any of the specified values match, OR
/// - Any of the specified groups match
///
/// # Examples
///
/// ```rust
/// use abac_rs::cache::{RuleFilter, DimensionFilter};
/// use abac_rs::{AbacRule, AttributeType};
///
/// // Filter for "resource" dimension matching "web01.example.com"
/// let filter = DimensionFilter::new("resource")
///     .with_value(AttributeType::String("web01.example.com".into()));
///
/// let rule = AbacRule::builder("test")
///     .dimension_values("resource", vec![
///         AttributeType::String("web01.example.com".into()),
///     ])
///     .build();
/// assert!(filter.should_cache(&rule));
/// ```
#[derive(Debug, Clone)]
pub struct DimensionFilter {
    dimension: String,
    values: Vec<AttributeType>,
}

impl DimensionFilter {
    /// Creates a filter for the given dimension with no values.
    pub fn new(dimension: impl Into<String>) -> Self {
        Self {
            dimension: dimension.into(),
            values: Vec::new(),
        }
    }

    /// Adds a value to match in the dimension.
    pub fn with_value(mut self, value: AttributeType) -> Self {
        self.values.push(value);
        self
    }

    /// Adds multiple values.
    pub fn with_values(mut self, values: impl IntoIterator<Item = AttributeType>) -> Self {
        self.values.extend(values);
        self
    }
}

impl RuleFilter for DimensionFilter {
    fn should_cache(&self, rule: &AbacRule) -> bool {
        match rule.dimensions.get(&self.dimension) {
            None => false,                     // Rule doesn't have this dimension
            Some(AttributeValue::All) => true, // Rule applies to all values
            Some(AttributeValue::Specific(set)) => {
                // Check if any of our values match
                for value in &self.values {
                    if set.contains(value) {
                        return true;
                    }
                }
                false
            }
        }
    }
}

/// Filter that only caches rules applicable everywhere (category="all" on all dimensions).
///
/// This is useful for caching global rules that always apply.
///
/// # Examples
///
/// ```rust
/// use abac_rs::cache::{RuleFilter, ApplicabilityFilter};
/// use abac_rs::AbacRule;
///
/// let filter = ApplicabilityFilter::global_only();
///
/// let rule = AbacRule::builder("global")
///     .dimension_all("user")
///     .dimension_all("resource")
///     .dimension_all("action")
///     .build();
///
/// assert!(filter.should_cache(&rule));
/// ```
#[derive(Debug, Clone, Copy)]
pub enum ApplicabilityFilter {
    /// Only cache global rules (all dimensions are "all")
    GlobalOnly,
    /// Cache all enabled rules
    AllEnabled,
}

impl ApplicabilityFilter {
    /// Creates a filter that only caches global rules.
    pub fn global_only() -> Self {
        Self::GlobalOnly
    }

    /// Creates a filter that caches all enabled rules.
    pub fn all_enabled() -> Self {
        Self::AllEnabled
    }
}

impl RuleFilter for ApplicabilityFilter {
    fn should_cache(&self, rule: &AbacRule) -> bool {
        match self {
            Self::GlobalOnly => rule
                .dimensions
                .values()
                .all(|v| matches!(v, AttributeValue::All)),
            Self::AllEnabled => rule.is_enabled(),
        }
    }
}

/// Composite filter that requires all child filters to pass (AND logic).
///
/// # Examples
///
/// ```rust
/// use abac_rs::cache::{RuleFilter, AndFilter, ApplicabilityFilter};
/// use abac_rs::AbacRule;
///
/// let filter = AndFilter::new()
///     .with(Box::new(ApplicabilityFilter::all_enabled()));
///
/// let rule = AbacRule::builder("test")
///     .enabled(true)
///     .build();
///
/// assert!(filter.should_cache(&rule));
/// ```
pub struct AndFilter {
    filters: Vec<Box<dyn RuleFilter>>,
}

impl std::fmt::Debug for AndFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AndFilter")
            .field("filters", &format_args!("<{} filters>", self.filters.len()))
            .finish()
    }
}

impl AndFilter {
    /// Creates a new AND filter with no children.
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    /// Adds a filter to the AND chain.
    pub fn with(mut self, filter: Box<dyn RuleFilter>) -> Self {
        self.filters.push(filter);
        self
    }
}

impl Default for AndFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleFilter for AndFilter {
    fn should_cache(&self, rule: &AbacRule) -> bool {
        self.filters.iter().all(|f| f.should_cache(rule))
    }
}

/// Composite filter that requires any child filter to pass (OR logic).
///
/// # Examples
///
/// ```rust
/// use abac_rs::cache::{RuleFilter, OrFilter, AcceptAllFilter};
/// use abac_rs::AbacRule;
///
/// let filter = OrFilter::new()
///     .with(Box::new(AcceptAllFilter));
///
/// let rule = AbacRule::builder("test").build();
/// assert!(filter.should_cache(&rule));
/// ```
pub struct OrFilter {
    filters: Vec<Box<dyn RuleFilter>>,
}

impl std::fmt::Debug for OrFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrFilter")
            .field("filters", &format_args!("<{} filters>", self.filters.len()))
            .finish()
    }
}

impl OrFilter {
    /// Creates a new OR filter with no children.
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    /// Adds a filter to the OR chain.
    pub fn with(mut self, filter: Box<dyn RuleFilter>) -> Self {
        self.filters.push(filter);
        self
    }
}

impl Default for OrFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleFilter for OrFilter {
    fn should_cache(&self, rule: &AbacRule) -> bool {
        self.filters.iter().any(|f| f.should_cache(rule))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHashSet as HashSet;

    #[test]
    fn test_accept_all_filter() {
        let filter = AcceptAllFilter;
        let rule = AbacRule::new("test");
        assert!(filter.should_cache(&rule));
    }

    #[test]
    fn test_dimension_filter() {
        let filter =
            DimensionFilter::new("resource").with_value(AttributeType::String("web01".into()));

        // Rule with matching value
        let mut rule = AbacRule::new("match");
        let mut set = HashSet::new();
        set.insert(AttributeType::String("web01".into()));
        rule.add_dimension("resource", AttributeValue::Specific(set));
        assert!(filter.should_cache(&rule));

        // Rule with non-matching value
        let mut rule2 = AbacRule::new("no-match");
        let mut set2 = HashSet::new();
        set2.insert(AttributeType::String("web02".into()));
        rule2.add_dimension("resource", AttributeValue::Specific(set2));
        assert!(!filter.should_cache(&rule2));

        // Rule with "all"
        let mut rule3 = AbacRule::new("all");
        rule3.add_dimension("resource", AttributeValue::All);
        assert!(filter.should_cache(&rule3));
    }

    #[test]
    fn test_applicability_filter_global() {
        let filter = ApplicabilityFilter::global_only();

        let mut rule = AbacRule::new("global");
        rule.add_dimension("user", AttributeValue::All);
        rule.add_dimension("resource", AttributeValue::All);
        rule.add_dimension("action", AttributeValue::All);

        assert!(filter.should_cache(&rule));

        // Rule with specific value should not pass
        let mut rule2 = AbacRule::new("specific");
        let mut set = HashSet::new();
        set.insert(AttributeType::String("alice".into()));
        rule2.add_dimension("user", AttributeValue::Specific(set));
        rule2.add_dimension("resource", AttributeValue::All);

        assert!(!filter.should_cache(&rule2));
    }

    #[test]
    fn test_applicability_filter_enabled() {
        let filter = ApplicabilityFilter::all_enabled();

        let mut rule = AbacRule::new("enabled");
        rule.enable();
        assert!(filter.should_cache(&rule));

        let rule2 = AbacRule::new("disabled");
        assert!(!filter.should_cache(&rule2));
    }

    #[test]
    fn test_and_filter() {
        let filter = AndFilter::new()
            .with(Box::new(ApplicabilityFilter::all_enabled()))
            .with(Box::new(AcceptAllFilter));

        let mut rule = AbacRule::new("test");
        rule.enable();
        assert!(filter.should_cache(&rule));

        let rule2 = AbacRule::new("disabled");
        assert!(!filter.should_cache(&rule2));
    }

    #[test]
    fn test_or_filter() {
        let filter = OrFilter::new().with(Box::new(AcceptAllFilter));

        let rule = AbacRule::new("test");
        assert!(filter.should_cache(&rule));
    }
}
