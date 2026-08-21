//! # aae — Accountable Agentic Execution protocol SDK
//!
//! This crate provides serde-derived types, hash chain helpers, and reference
//! adapters for building AAE-conformant agent runtimes in Rust.
//!
//! See <https://github.com/r3moteBee/aae> for the protocol specification.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]

pub mod hashchain;
pub mod models;
#[cfg(feature = "signing")]
pub mod signing;

pub use models::{
    AuditEvent, BlastRadius, CapabilityScope, CapabilityToken, Confidence, Context, CostEstimate,
    Decision, Effect, EventSignature, PolicyDecision, Preview, Proposal, Reversibility, Step,
    StepPreview, ToolRegistration, EVENT_TYPE_ARTIFACT_ATTESTED, STRICTNESS_STRICT_LITERAL,
    STRICTNESS_STRICT_TEMPLATE,
};

/// Protocol version supported by this SDK.
pub const PROTOCOL_VERSION: &str = "0.8";

/// SDK version (matches Cargo.toml).
pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");
