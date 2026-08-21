//! `adaptive-card-core` — pure Rust library for Adaptive Cards v1.6.

#![doc(html_root_url = "https://docs.rs/adaptive-card-core/0.1.0")]

pub mod analyze;
pub mod data_to_card;
pub mod error;
pub mod host;
pub mod knowledge;
pub mod optimize;
pub mod prompt;
pub mod schema;
pub mod template;
pub mod transform;
pub mod types;
mod validate;

pub use analyze::accessibility::check_accessibility;
pub use analyze::{analyze_card, count_elements, find_duplicate_ids};
pub use data_to_card::data_to_card;
pub use error::{Error, Result};
pub use host::adapt::adapt_for_host;
pub use host::check_compatibility;
pub use knowledge::{KnowledgeBase, KnowledgeEntry};
pub use optimize::optimize_card;
pub use prompt::{PromptOpts, build_example_showcase, build_system_prompt};
pub use template::template_card;
pub use transform::transform_card;
pub use types::{
    A11yIssue, A11ySeverity, AccessibilityReport, CardAnalysis, CardVersion, DataToCardOpts, Host,
    HostCompatReport, OptimizeOpts, Presentation, SchemaError, TemplateResult, TransformReport,
    TransformTarget, ValidationReport,
};
pub use validate::validate_card;
