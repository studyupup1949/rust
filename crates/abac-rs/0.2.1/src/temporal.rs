//! Temporal ABAC rules with time-based validity.
//!
//! This module provides [`TemporalAbacRule`] which wraps an [`AbacRule`] with
//! time-based activation windows using [`acls_rs::permission::Timestamp`].
//!
//! # Time-Based Access Control
//!
//! Temporal rules are only active within their validity window:
//! - `valid_from`: Optional start timestamp (None = always valid from past)
//! - `valid_until`: Optional end timestamp (None = never expires)
//!
//! # Examples
//!
//! ```rust
//! use abac_rs::{AbacRule, TemporalAbacRule, AbacPolicy};
//!
//! let rule = AbacRule::builder("contractor_access")
//!     .enabled(true)
//!     .build();
//!
//! // Valid for 24 hours
//! let one_day = 24 * 60 * 60 * 1000;
//! let temporal = TemporalAbacRule::valid_for_duration(rule, one_day).unwrap();
//!
//! let mut policy = AbacPolicy::new();
//! policy.add_temporal_rule(temporal);
//! ```

use crate::AbacRule;
use acls_rs::permission::{current_timestamp_millis, Timestamp};

/// Errors that can occur when creating temporal rules.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TemporalError {
    /// Invalid time window: valid_from >= valid_until
    #[error("invalid time window: from={from} >= until={until}")]
    InvalidTimeWindow {
        /// The start timestamp
        from: Timestamp,
        /// The end timestamp
        until: Timestamp,
    },
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A temporal ABAC rule with time-based validity.
///
/// Integrates with acls-rs temporal permissions to provide time-limited
/// access control. Rules are only active within their validity window.
///
/// # Example
///
/// ```
/// use abac_rs::{AbacRule, TemporalAbacRule};
/// use acls_rs::permission::current_timestamp_millis;
///
/// let rule = AbacRule::builder("temp_access")
///     .enabled(true)
///     .build();
///
/// let now = current_timestamp_millis();
/// let one_hour = 60 * 60 * 1000;
///
/// // Rule valid for one hour
/// let temporal_rule = TemporalAbacRule::new(
///     rule,
///     Some(now),
///     Some(now + one_hour)
/// ).unwrap();
///
/// assert!(temporal_rule.is_currently_valid());
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TemporalAbacRule {
    /// The underlying ABAC rule
    pub rule: AbacRule,
    /// Timestamp from which this rule is valid (None = no start limit)
    pub valid_from: Option<Timestamp>,
    /// Timestamp until which this rule is valid (None = no end limit)
    pub valid_until: Option<Timestamp>,
}

impl TemporalAbacRule {
    /// Creates a new temporal ABAC rule.
    ///
    /// # Arguments
    ///
    /// * `rule` - The underlying ABAC rule
    /// * `valid_from` - Optional start timestamp (milliseconds since Unix epoch)
    /// * `valid_until` - Optional end timestamp (milliseconds since Unix epoch)
    ///
    /// # Errors
    ///
    /// Returns [`TemporalError::InvalidTimeWindow`] if both timestamps are set
    /// and `valid_from` >= `valid_until`.
    pub fn new(
        rule: AbacRule,
        valid_from: Option<Timestamp>,
        valid_until: Option<Timestamp>,
    ) -> Result<Self, TemporalError> {
        if let (Some(from), Some(until)) = (valid_from, valid_until) {
            if from >= until {
                return Err(TemporalError::InvalidTimeWindow { from, until });
            }
        }

        Ok(Self {
            rule,
            valid_from,
            valid_until,
        })
    }

    /// Creates a temporal rule valid from now until the given timestamp.
    ///
    /// # Errors
    ///
    /// Returns [`TemporalError::InvalidTimeWindow`] if `until` is in the past.
    pub fn valid_until(rule: AbacRule, until: Timestamp) -> Result<Self, TemporalError> {
        Self::new(rule, Some(current_timestamp_millis()), Some(until))
    }

    /// Creates a temporal rule valid from now for the given duration in milliseconds.
    pub fn valid_for_duration(rule: AbacRule, duration_ms: u64) -> Result<Self, TemporalError> {
        let now = current_timestamp_millis();
        Self::new(rule, Some(now), Some(now + duration_ms))
    }

    /// Creates a temporal rule valid from the given timestamp onwards.
    pub fn valid_from(rule: AbacRule, from: Timestamp) -> Result<Self, TemporalError> {
        Self::new(rule, Some(from), None)
    }

    /// Checks if this rule is currently valid based on the current time.
    pub fn is_currently_valid(&self) -> bool {
        self.is_valid_at(current_timestamp_millis())
    }

    /// Checks if this rule is valid at a specific timestamp.
    pub fn is_valid_at(&self, timestamp: Timestamp) -> bool {
        // Check lower bound
        if let Some(from) = self.valid_from {
            if timestamp < from {
                return false;
            }
        }

        // Check upper bound
        if let Some(until) = self.valid_until {
            if timestamp >= until {
                return false;
            }
        }

        true
    }

    /// Gets a reference to the underlying rule.
    pub fn inner(&self) -> &AbacRule {
        &self.rule
    }

    /// Gets a mutable reference to the underlying rule.
    pub fn inner_mut(&mut self) -> &mut AbacRule {
        &mut self.rule
    }

    /// Consumes this temporal rule and returns the underlying rule.
    pub fn into_inner(self) -> AbacRule {
        self.rule
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_rule_new() {
        let rule = AbacRule::new("test");
        let now = current_timestamp_millis();
        let later = now + 1000;

        let temporal = TemporalAbacRule::new(rule, Some(now), Some(later)).unwrap();

        assert_eq!(temporal.valid_from, Some(now));
        assert_eq!(temporal.valid_until, Some(later));
    }

    #[test]
    fn test_temporal_rule_valid_until() {
        let rule = AbacRule::new("test");
        let future = current_timestamp_millis() + 10000;

        let temporal = TemporalAbacRule::valid_until(rule, future).unwrap();

        assert!(temporal.is_currently_valid());
        assert!(!temporal.is_valid_at(future));
    }

    #[test]
    fn test_temporal_rule_valid_for_duration() {
        let rule = AbacRule::new("test");
        let one_hour = 60 * 60 * 1000;

        let temporal = TemporalAbacRule::valid_for_duration(rule, one_hour).unwrap();

        assert!(temporal.is_currently_valid());
    }

    #[test]
    fn test_temporal_rule_valid_from() {
        let rule = AbacRule::new("test");
        let past = current_timestamp_millis() - 1000;

        let temporal = TemporalAbacRule::valid_from(rule, past).unwrap();

        assert!(temporal.is_currently_valid());
    }

    #[test]
    fn test_temporal_rule_is_valid_at() {
        let rule = AbacRule::new("test");
        let start = 1000;
        let end = 2000;

        let temporal = TemporalAbacRule::new(rule, Some(start), Some(end)).unwrap();

        assert!(!temporal.is_valid_at(500)); // Before start
        assert!(temporal.is_valid_at(1000)); // At start
        assert!(temporal.is_valid_at(1500)); // In window
        assert!(!temporal.is_valid_at(2000)); // At end (exclusive)
        assert!(!temporal.is_valid_at(2500)); // After end
    }

    #[test]
    fn test_temporal_rule_no_bounds() {
        let rule = AbacRule::new("test");
        let temporal = TemporalAbacRule::new(rule, None, None).unwrap();

        // Always valid with no bounds
        assert!(temporal.is_valid_at(0));
        assert!(temporal.is_valid_at(current_timestamp_millis()));
        assert!(temporal.is_valid_at(u64::MAX));
    }

    #[test]
    fn test_temporal_rule_only_from() {
        let rule = AbacRule::new("test");
        let start = current_timestamp_millis();
        let temporal = TemporalAbacRule::new(rule, Some(start), None).unwrap();

        assert!(!temporal.is_valid_at(start - 1));
        assert!(temporal.is_valid_at(start));
        assert!(temporal.is_valid_at(start + 1000));
    }

    #[test]
    fn test_temporal_rule_only_until() {
        let rule = AbacRule::new("test");
        let end = current_timestamp_millis() + 1000;
        let temporal = TemporalAbacRule::new(rule, None, Some(end)).unwrap();

        assert!(temporal.is_valid_at(0));
        assert!(temporal.is_valid_at(end - 1));
        assert!(!temporal.is_valid_at(end));
    }

    #[test]
    fn test_temporal_rule_inner() {
        let rule = AbacRule::new("test");
        let temporal = TemporalAbacRule::valid_for_duration(rule, 1000).unwrap();

        assert_eq!(temporal.inner().name, "test");
    }

    #[test]
    fn test_temporal_rule_into_inner() {
        let rule = AbacRule::new("test");
        let temporal = TemporalAbacRule::valid_for_duration(rule, 1000).unwrap();

        let inner = temporal.into_inner();
        assert_eq!(inner.name, "test");
    }
}
