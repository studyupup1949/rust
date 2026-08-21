//! # Abaco
//!
//! Math engine for Rust — expression evaluation, unit conversion, and numeric types.
//!
//! Abaco provides three core capabilities:
//!
//! - **Expression evaluation** — arithmetic, 28+ math functions, variables, scientific notation
//! - **Unit conversion** — 95+ built-in units across 14 categories
//! - **Natural language parsing** — "what is 15% of 230", "convert 5 km to miles" (feature-gated)
//!
//! ## Quick Start
//!
//! ```rust
//! use abaco::{Evaluator, UnitRegistry};
//!
//! // Evaluate expressions
//! let eval = Evaluator::new();
//! let result = eval.eval("2 + 3 * 4").unwrap();
//! assert_eq!(result.to_string(), "14");
//!
//! // Convert units
//! let registry = UnitRegistry::new();
//! let result = registry.convert(100.0, "celsius", "fahrenheit").unwrap();
//! assert!((result.to_value - 212.0).abs() < 0.1);
//! ```
//!
//! ## Feature Flags
//!
//! - `ai` — enables natural language math parsing (adds reqwest, tokio)
//! - `full` — enables all optional features

pub mod core;
pub mod eval;
pub mod units;

#[cfg(feature = "ai")]
pub mod ai;

// Re-export primary types for convenience
pub use crate::core::{ConversionResult, Currency, Unit, UnitCategory, Value};
pub use crate::eval::{EvalError, Evaluator, Token, tokenize};
pub use crate::units::{UnitError, UnitRegistry};

#[cfg(feature = "ai")]
pub use crate::ai::{AiError, CalculationHistory, HistoryEntry, NlParser, ParsedQuery};
