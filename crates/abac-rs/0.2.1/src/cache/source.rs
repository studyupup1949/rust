//! Rule source traits and implementations.
//!
//! Sources provide the origin of ABAC rules - whether from LDAP, files,
//! or other backends. The [`RuleSource`] trait enables pluggable backends.

use crate::AbacRule;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Errors that can occur when fetching rules from a source.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum RuleSourceError {
    /// Connection to the source failed
    ConnectionFailed(String),
    /// Authentication failed
    AuthenticationFailed(String),
    /// Failed to parse rules
    ParseError(String),
    /// Source not available
    Unavailable(String),
    /// All fallback sources failed
    AllSourcesFailed {
        /// Number of sources attempted
        attempts: usize,
        /// Individual error messages
        errors: Vec<String>,
    },
    /// Generic error
    Other(String),
}

impl fmt::Display for RuleSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionFailed(s) => write!(f, "Connection failed: {}", s),
            Self::AuthenticationFailed(s) => write!(f, "Authentication failed: {}", s),
            Self::ParseError(s) => write!(f, "Parse error: {}", s),
            Self::Unavailable(s) => write!(f, "Source unavailable: {}", s),
            Self::AllSourcesFailed { attempts, errors } => {
                write!(
                    f,
                    "All {} rule sources failed: {}",
                    attempts,
                    errors.join("; ")
                )
            }
            Self::Other(s) => write!(f, "Error: {}", s),
        }
    }
}

impl Error for RuleSourceError {}

/// Trait for sources that can provide ABAC rules.
///
/// Implementations can fetch rules from LDAP, files, databases, or any other backend.
///
/// # Examples
///
/// ```rust
/// use abac_rs::cache::{RuleSource, RuleSourceError};
/// use abac_rs::AbacRule;
///
/// struct FileSource {
///     path: String,
/// }
///
/// impl RuleSource for FileSource {
///     fn fetch_all(&mut self) -> Result<Vec<AbacRule>, RuleSourceError> {
///         // Read rules from file
///         Ok(vec![])
///     }
///
///     fn fetch_updated_since(&mut self, _timestamp: u64)
///         -> Result<Vec<AbacRule>, RuleSourceError>
///     {
///         // Incremental updates
///         Ok(vec![])
///     }
/// }
/// ```
pub trait RuleSource {
    /// Fetch all rules from the source.
    fn fetch_all(&mut self) -> Result<Vec<AbacRule>, RuleSourceError>;

    /// Fetch rules updated since the given timestamp (milliseconds since Unix epoch).
    ///
    /// This enables incremental updates to avoid re-fetching all rules.
    fn fetch_updated_since(&mut self, timestamp: u64) -> Result<Vec<AbacRule>, RuleSourceError>;

    /// Check if the source is available.
    fn is_available(&self) -> bool {
        true
    }
}

/// An in-memory rule source for testing.
///
/// Uses Arc for efficient sharing of rule data without cloning large vectors.
///
/// # Examples
///
/// ```rust
/// use abac_rs::cache::{MemorySource, RuleSource};
/// use abac_rs::AbacRule;
///
/// let rule = AbacRule::builder("test")
///     .enabled(true)
///     .build();
///
/// let mut source = MemorySource::new(vec![rule]);
/// let rules = source.fetch_all().unwrap();
/// assert_eq!(rules.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct MemorySource {
    rules: Arc<Vec<AbacRule>>,
}

impl MemorySource {
    /// Creates a new memory source with the given rules.
    pub fn new(rules: Vec<AbacRule>) -> Self {
        Self {
            rules: Arc::new(rules),
        }
    }

    /// Adds a rule to the source.
    ///
    /// Note: This clones the existing Arc'd vector, adds the rule, and wraps it in a new Arc.
    /// If you need to add many rules, consider building the full vector first and calling `new()`.
    pub fn add_rule(&mut self, rule: AbacRule) {
        let mut rules = (*self.rules).clone();
        rules.push(rule);
        self.rules = Arc::new(rules);
    }
}

impl RuleSource for MemorySource {
    fn fetch_all(&mut self) -> Result<Vec<AbacRule>, RuleSourceError> {
        Ok((*self.rules).clone())
    }

    fn fetch_updated_since(&mut self, _timestamp: u64) -> Result<Vec<AbacRule>, RuleSourceError> {
        // Memory source doesn't track timestamps, return all
        Ok((*self.rules).clone())
    }
}

/// A composite source that tries multiple sources in order.
///
/// If the primary source fails, it falls back to secondary sources.
///
/// # Examples
///
/// ```rust
/// use abac_rs::cache::{CompositeSource, MemorySource, RuleSource};
/// use abac_rs::AbacRule;
///
/// let primary = MemorySource::new(vec![]);
/// let fallback = MemorySource::new(vec![]);
///
/// let mut composite = CompositeSource::new()
///     .with_source(Box::new(primary))
///     .with_source(Box::new(fallback));
///
/// let rules = composite.fetch_all().unwrap();
/// ```
pub struct CompositeSource {
    sources: Vec<Box<dyn RuleSource>>,
}

impl std::fmt::Debug for CompositeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeSource")
            .field("sources", &format_args!("<{} sources>", self.sources.len()))
            .finish()
    }
}

impl CompositeSource {
    /// Creates a new composite source with no sources.
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// Adds a source to the composite.
    pub fn with_source(mut self, source: Box<dyn RuleSource>) -> Self {
        self.sources.push(source);
        self
    }

    /// Adds a source to the composite.
    pub fn add_source(&mut self, source: Box<dyn RuleSource>) {
        self.sources.push(source);
    }
}

impl Default for CompositeSource {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleSource for CompositeSource {
    fn fetch_all(&mut self) -> Result<Vec<AbacRule>, RuleSourceError> {
        let mut errors = Vec::new();

        for (idx, source) in self.sources.iter_mut().enumerate() {
            match source.fetch_all() {
                Ok(rules) => {
                    log::info!(
                        "CompositeSource: source {} succeeded with {} rules",
                        idx,
                        rules.len()
                    );
                    return Ok(rules);
                }
                Err(e) => {
                    log::warn!("CompositeSource: source {} failed: {}", idx, e);
                    log::debug!("CompositeSource: source {} error details: {:?}", idx, e);
                    errors.push(format!("source {}: {}", idx, e));
                }
            }
        }

        Err(RuleSourceError::AllSourcesFailed {
            attempts: errors.len(),
            errors,
        })
    }

    fn fetch_updated_since(&mut self, timestamp: u64) -> Result<Vec<AbacRule>, RuleSourceError> {
        let mut errors = Vec::new();

        for (idx, source) in self.sources.iter_mut().enumerate() {
            match source.fetch_updated_since(timestamp) {
                Ok(rules) => {
                    log::info!(
                        "CompositeSource: source {} succeeded with {} rules updated since {}",
                        idx,
                        rules.len(),
                        timestamp
                    );
                    return Ok(rules);
                }
                Err(e) => {
                    log::warn!("CompositeSource: source {} failed: {}", idx, e);
                    log::debug!("CompositeSource: source {} error details: {:?}", idx, e);
                    errors.push(format!("source {}: {}", idx, e));
                }
            }
        }

        Err(RuleSourceError::AllSourcesFailed {
            attempts: errors.len(),
            errors,
        })
    }

    fn is_available(&self) -> bool {
        self.sources.iter().any(|s| s.is_available())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_source() {
        let mut rule = AbacRule::new("test");
        rule.enable();

        let mut source = MemorySource::new(vec![rule]);
        let rules = source.fetch_all().unwrap();

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "test");
    }

    #[test]
    fn test_composite_source() {
        let mut rule1 = AbacRule::new("primary");
        rule1.enable();
        let primary = MemorySource::new(vec![rule1]);

        let mut fallback_rule = AbacRule::new("fallback");
        fallback_rule.enable();
        let fallback = MemorySource::new(vec![fallback_rule]);

        let mut composite = CompositeSource::new()
            .with_source(Box::new(primary))
            .with_source(Box::new(fallback));

        let rules = composite.fetch_all().unwrap();
        // Should get primary source results
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "primary");
    }
}
