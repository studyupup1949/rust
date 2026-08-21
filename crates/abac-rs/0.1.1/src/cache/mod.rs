//! Caching infrastructure for high-performance evaluation.
//!
//! This module provides a composable architecture for efficiently caching and evaluating
//! ABAC rules, designed for deployments where rules are periodically fetched
//! from external sources and cached locally.
//!
//! # Architecture
//!
//! The caching system has three main components:
//!
//! 1. **Sources** ([`RuleSource`]): Where rules come from (LDAP, files, etc.)
//! 2. **Filters** ([`RuleFilter`]): Determine which rules to cache
//! 3. **Pipeline** ([`RulePipeline`]): Orchestrates source → filter → evaluate
//!
//! # Example
//!
//! ```rust
//! use abac_rs::cache::*;
//! use abac_rs::{AbacRule, AbacRequest, AttributeType};
//!
//! // Create a pipeline
//! let mut pipeline = RulePipeline::builder()
//!     .with_filter(Box::new(ApplicabilityFilter::all_enabled()))
//!     .build().unwrap();
//!
//! // Load rules
//! let rule = AbacRule::builder("test")
//!     .enabled(true)
//!     .build();
//! pipeline.load(vec![rule]).unwrap();
//!
//! // Evaluate
//! let mut request = AbacRequest::new();
//! request.add_attribute("user", AttributeType::String("alice".into()), vec![]).unwrap();
//! let result = pipeline.evaluate(&request);
//! ```

#[cfg(feature = "bloom")]
pub mod bloom;
pub(crate) mod deny_index;
pub mod filter;
pub mod pipeline;
pub mod request_key;
pub mod source;

#[cfg(feature = "bloom")]
pub use bloom::DimensionBloom;
pub use filter::{
    AcceptAllFilter, AndFilter, ApplicabilityFilter, DimensionFilter, OrFilter, RuleFilter,
};
pub use pipeline::{PipelineError, RulePipeline, RulePipelineBuilder};
pub use request_key::RequestKey;
pub use source::{CompositeSource, MemorySource, RuleSource, RuleSourceError};
