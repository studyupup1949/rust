//! Generic Attribute-Based Access Control (ABAC) evaluation engine.
//!
//! `abac-rs` provides a dimension-agnostic ABAC engine with advanced optimization
//! techniques that deliver performance exceeding specialized engines at scale:
//!
//! - **5-layer evaluation pipeline**: Constant-result fast path, LRU cache,
//!   Bloom filters, composite indexing, compiled evaluation
//! - **Arbitrary dimensions**: Not limited to user/host/service — define any
//!   attribute dimensions your policy requires
//! - **Pluggable matchers**: Default exact matching (HashSet O(1)) or custom
//!   predicates (CIDR, numeric ranges, time-of-day)
//! - **Multi-type attributes**: String, Integer, Float, IpAddr, IpCidr, Custom
//! - **Zero unsafe code**: Built entirely in safe Rust
//!
//! # Performance
//!
//! Optimized implementation results (Apple M4, 3-dimensional workloads):
//! - **100 rules**: ~437K req/s, ~2.29 µs mean latency, 8 KB memory
//! - **1,000 rules**: ~509K req/s, ~1.96 µs mean latency, 80 KB memory
//! - **10,000 rules**: ~631K req/s, ~1.58 µs mean latency, 800 KB memory
//! - **100,000 rules**: ~625K req/s, ~1.50 µs mean latency, 8.0 MB memory
//!
//! At small scales (< 10K rules), hbac-rs maintains a 7-15% performance advantage
//! due to specialized 3D structure. At large scales (100K+ rules), abac-rs is
//! **2.6× faster** than hbac-rs and has **20× better P95/P99 latency** thanks to
//! compiled evaluation and deny-only indexing. Choose abac-rs for maximum
//! performance at scale, N-dimensional flexibility, or custom attribute types.
//!
//! # Examples
//!
//! ```
//! use abac_rs::{AbacRule, AbacPolicy, AbacRequest, AttributeType};
//!
//! // Create a rule: engineers can read prod resources
//! let rule = AbacRule::builder("allow-engineers-prod-read")
//!     .dimension_values("user", vec![
//!         AttributeType::String("group:engineers".into()),
//!     ])
//!     .dimension_values("resource", vec![
//!         AttributeType::String("prod:db-01".into()),
//!     ])
//!     .dimension_values("action", vec![
//!         AttributeType::String("read".into()),
//!     ])
//!     .enabled(true)
//!     .build();
//!
//! // Build a policy and evaluate
//! let mut policy = AbacPolicy::new();
//! policy.add_rule(rule).unwrap();
//!
//! let mut request = AbacRequest::new();
//! request.add_attribute(
//!     "user",
//!     AttributeType::String("alice".into()),
//!     vec![AttributeType::String("group:engineers".into())]
//! );
//! request.add_attribute("resource", AttributeType::String("prod:db-01".into()), vec![]);
//! request.add_attribute("action", AttributeType::String("read".into()), vec![]);
//!
//! let decision = policy.evaluate(&request);
//! assert!(decision.is_allowed());
//! ```

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod attribute;
pub mod cache;
pub mod compiled;
pub mod composition;
pub mod decision;
pub mod explain;
pub mod index;
pub mod matcher;
pub mod policy;
pub mod request;
pub mod rule;
pub mod temporal;

// Re-export commonly used types
pub use attribute::{AttributeType, AttributeTypeTrait, AttributeValue};
pub use composition::{
    ComposedPolicy, ComposedPolicyCore, ComposedPolicyLocal, CompositionMode, PermissionCacheLock,
};
pub use decision::Decision;
pub use explain::{ExplainedDecision, RuleMatch};
pub use matcher::{ExactMatcher, Matcher};
pub use policy::{
    AbacPolicy, AbacPolicyCore, AbacPolicyLocal, CacheLock, CacheStats, PolicyBuilder, PolicyError,
};
pub use request::{AbacRequest, RequestError};
pub use rule::{AbacRule, AbacRuleBuilder, RuleType};
pub use temporal::{TemporalAbacRule, TemporalError};

/// Prelude module for convenient imports.
///
/// This module re-exports commonly used types and traits for easy access.
pub mod prelude {
    pub use crate::attribute::{AttributeType, AttributeValue};
    pub use crate::decision::Decision;
    pub use crate::matcher::{ExactMatcher, Matcher};
    pub use crate::policy::{AbacPolicy, PolicyBuilder};
    pub use crate::request::AbacRequest;
    pub use crate::rule::{AbacRule, AbacRuleBuilder, RuleType};
}
