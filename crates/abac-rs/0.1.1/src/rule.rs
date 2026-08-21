//! ABAC rule types.

use std::collections::HashMap;

use crate::attribute::{AttributeType, AttributeValue};

/// Rule type: allow or deny access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RuleType {
    /// Allow access if all dimensions match
    Allow,

    /// Deny access if all dimensions match (takes precedence over Allow)
    Deny,
}

/// Multi-dimensional ABAC rule.
///
/// Specifies attribute requirements across arbitrary dimensions. A request
/// matches this rule only when ALL dimension requirements are satisfied.
///
/// # Examples
///
/// ```
/// use abac_rs::{AbacRule, AttributeType, RuleType};
///
/// let rule = AbacRule::builder("allow-engineers-prod")
///     // User dimension: group:engineers
///     .dimension_values("user", vec![
///         AttributeType::String("group:engineers".into()),
///     ])
///     // Resource dimension: specific prod hosts
///     .dimension_values("resource", vec![
///         AttributeType::String("prod:db-01".into()),
///         AttributeType::String("prod:web-01".into()),
///     ])
///     // Action dimension: read-only
///     .dimension_values("action", vec![
///         AttributeType::String("read".into()),
///     ])
///     .enabled(true)
///     .build();
///
/// assert_eq!(rule.rule_type, RuleType::Allow);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbacRule {
    /// Unique rule identifier
    pub name: String,

    /// Map of dimension name → required values
    pub dimensions: HashMap<String, AttributeValue>,

    /// Rule type: allow or deny
    pub rule_type: RuleType,

    /// Whether this rule is active
    pub enabled: bool,
}

impl AbacRule {
    /// Create a new rule with the given name.
    ///
    /// Rules are created as `Allow` type and disabled by default.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dimensions: HashMap::new(),
            rule_type: RuleType::Allow,
            enabled: false,
        }
    }

    /// Enable this rule.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable this rule.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Set the rule type to Allow.
    pub fn set_allow(&mut self) {
        self.rule_type = RuleType::Allow;
    }

    /// Set the rule type to Deny.
    pub fn set_deny(&mut self) {
        self.rule_type = RuleType::Deny;
    }

    /// Check if this rule is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if this is an allow rule.
    pub fn is_allow(&self) -> bool {
        self.rule_type == RuleType::Allow
    }

    /// Check if this is a deny rule.
    pub fn is_deny(&self) -> bool {
        self.rule_type == RuleType::Deny
    }

    /// Add a dimension requirement to this rule.
    pub fn add_dimension(&mut self, dimension: impl Into<String>, value: AttributeValue) {
        self.dimensions.insert(dimension.into(), value);
    }

    /// Get the attribute value for a dimension.
    pub fn get_dimension(&self, dimension: &str) -> Option<&AttributeValue> {
        self.dimensions.get(dimension)
    }

    /// Iterator over dimension names.
    pub fn dimension_names(&self) -> impl Iterator<Item = &str> {
        self.dimensions.keys().map(|s| s.as_str())
    }

    /// Check if a dimension uses `All` (wildcard).
    pub fn dimension_is_all(&self, dimension: &str) -> bool {
        self.dimensions
            .get(dimension)
            .map(|v| v.is_all())
            .unwrap_or(false)
    }

    /// Creates a builder for constructing an ABAC rule.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use abac_rs::{AbacRule, AttributeValue, AttributeType};
    ///
    /// let rule = AbacRule::builder("allow-read")
    ///     .dimension_all("user")
    ///     .dimension_values("action", vec![AttributeType::String("read".into())])
    ///     .enabled(true)
    ///     .build();
    ///
    /// assert!(rule.is_allow());
    /// assert!(rule.is_enabled());
    /// assert!(rule.dimension_is_all("user"));
    /// ```
    pub fn builder(name: impl Into<String>) -> AbacRuleBuilder {
        AbacRuleBuilder {
            name: name.into(),
            rule_type: RuleType::Allow,
            enabled: false,
            dimensions: HashMap::new(),
        }
    }
}

/// Builder for constructing [`AbacRule`] instances.
///
/// Provides a fluent API for rule construction with convenience methods
/// for common dimension patterns.
#[derive(Debug)]
pub struct AbacRuleBuilder {
    name: String,
    rule_type: RuleType,
    enabled: bool,
    dimensions: HashMap<String, AttributeValue>,
}

impl AbacRuleBuilder {
    /// Sets this rule to deny access.
    pub fn deny(mut self) -> Self {
        self.rule_type = RuleType::Deny;
        self
    }

    /// Sets whether the rule is enabled (default: false).
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Adds a dimension with a raw [`AttributeValue`].
    pub fn dimension(mut self, name: impl Into<String>, value: AttributeValue) -> Self {
        self.dimensions.insert(name.into(), value);
        self
    }

    /// Adds a dimension that matches all values.
    pub fn dimension_all(mut self, name: impl Into<String>) -> Self {
        self.dimensions.insert(name.into(), AttributeValue::All);
        self
    }

    /// Adds a dimension with specific values.
    pub fn dimension_values(
        mut self,
        name: impl Into<String>,
        values: impl IntoIterator<Item = AttributeType>,
    ) -> Self {
        self.dimensions
            .insert(name.into(), AttributeValue::from_values(values));
        self
    }

    /// Builds the ABAC rule.
    pub fn build(self) -> AbacRule {
        AbacRule {
            name: self.name,
            dimensions: self.dimensions,
            rule_type: self.rule_type,
            enabled: self.enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AttributeType;
    use ahash::AHashSet as HashSet;

    #[test]
    fn test_rule_new() {
        let rule = AbacRule::new("test-rule");
        assert_eq!(rule.name, "test-rule");
        assert!(!rule.enabled);
        assert_eq!(rule.rule_type, RuleType::Allow);
        assert!(rule.dimensions.is_empty());
    }

    #[test]
    fn test_rule_enable_disable() {
        let mut rule = AbacRule::new("test");
        assert!(!rule.is_enabled());

        rule.enable();
        assert!(rule.is_enabled());

        rule.disable();
        assert!(!rule.is_enabled());
    }

    #[test]
    fn test_rule_type() {
        let mut rule = AbacRule::new("test");

        assert!(rule.is_allow());
        assert!(!rule.is_deny());

        rule.set_deny();
        assert!(!rule.is_allow());
        assert!(rule.is_deny());

        rule.set_allow();
        assert!(rule.is_allow());
        assert!(!rule.is_deny());
    }

    #[test]
    fn test_rule_add_dimension() {
        let mut rule = AbacRule::new("test");

        let mut set = HashSet::new();
        set.insert(AttributeType::String("alice".into()));
        rule.add_dimension("user", AttributeValue::Specific(set));

        rule.add_dimension("resource", AttributeValue::All);

        assert!(rule.get_dimension("user").is_some());
        assert!(rule.get_dimension("resource").is_some());
        assert!(rule.get_dimension("action").is_none());

        assert!(!rule.dimension_is_all("user"));
        assert!(rule.dimension_is_all("resource"));
    }

    #[test]
    fn test_rule_dimension_names() {
        let mut rule = AbacRule::new("test");
        rule.add_dimension("user", AttributeValue::All);
        rule.add_dimension("resource", AttributeValue::All);
        rule.add_dimension("action", AttributeValue::All);

        let names: Vec<_> = rule.dimension_names().collect();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"user"));
        assert!(names.contains(&"resource"));
        assert!(names.contains(&"action"));
    }

    #[test]
    fn test_builder_basic() {
        let rule = AbacRule::builder("allow-read")
            .dimension_all("user")
            .dimension_values("action", vec![AttributeType::String("read".into())])
            .enabled(true)
            .build();

        assert_eq!(rule.name, "allow-read");
        assert!(rule.is_allow());
        assert!(rule.is_enabled());
        assert!(rule.dimension_is_all("user"));
        assert!(!rule.dimension_is_all("action"));
    }

    #[test]
    fn test_builder_deny() {
        let rule = AbacRule::builder("deny-write")
            .deny()
            .dimension_all("user")
            .dimension_all("resource")
            .dimension_values("action", vec![AttributeType::String("write".into())])
            .enabled(true)
            .build();

        assert!(rule.is_deny());
        assert!(rule.is_enabled());
    }

    #[test]
    fn test_builder_dimension_values() {
        let rule = AbacRule::builder("multi")
            .dimension_values(
                "user",
                vec![
                    AttributeType::String("alice".into()),
                    AttributeType::String("bob".into()),
                ],
            )
            .build();

        let dim = rule.get_dimension("user").unwrap();
        assert!(!dim.is_all());
        if let AttributeValue::Specific(set) = dim {
            assert_eq!(set.len(), 2);
        } else {
            panic!("Expected Specific");
        }
    }

    #[test]
    fn test_builder_equivalence() {
        let mut manual = AbacRule::new("test");
        manual.add_dimension("user", AttributeValue::All);
        let mut set = HashSet::new();
        set.insert(AttributeType::String("read".into()));
        manual.add_dimension("action", AttributeValue::Specific(set));
        manual.enable();

        let built = AbacRule::builder("test")
            .dimension_all("user")
            .dimension_values("action", vec![AttributeType::String("read".into())])
            .enabled(true)
            .build();

        assert_eq!(manual.name, built.name);
        assert_eq!(manual.enabled, built.enabled);
        assert_eq!(manual.rule_type, built.rule_type);
        assert_eq!(manual.dimensions, built.dimensions);
    }
}
