//! # Abaco
//!
//! Math engine for Rust — expression evaluation, unit conversion, DSP math, and numeric types.
//!
//! Abaco provides four core capabilities:
//!
//! - **Expression evaluation** — arithmetic, 35+ math functions, implicit multiplication,
//!   factorial (`5!`), statistics, variables, scientific notation
//! - **Unit conversion** — 120+ built-in units across 18 categories, 80+ aliases
//! - **DSP math** — decibel conversions, MIDI/frequency, panning, envelope, antialiasing
//! - **Natural language parsing** — "what is 15% of 230", "convert 5 km to miles" (feature-gated)
//!
//! ## Quick Start
//!
//! ```rust
//! use abaco::{Evaluator, UnitRegistry, Value};
//!
//! // Evaluate expressions
//! let eval = Evaluator::new();
//! assert_eq!(eval.eval("2 + 3 * 4").unwrap(), Value::Integer(14));
//! assert_eq!(eval.eval("5!").unwrap(), Value::Integer(120));
//! assert_eq!(eval.eval("2pi").unwrap().to_string(), "6.283185307179586");
//!
//! // Convert units (including aliases like °C, kph)
//! let registry = UnitRegistry::new();
//! let result = registry.convert(100.0, "°C", "°F").unwrap();
//! assert!((result.to_value - 212.0).abs() < 0.1);
//!
//! // LaTeX output
//! let val = Value::Fraction(1, 3);
//! assert_eq!(val.to_latex(), "\\frac{1}{3}");
//! ```
//!
//! ## Feature Flags
//!
//! - `ai` — enables natural language math parsing, currency exchange, history persistence
//! - `full` — enables all optional features

pub mod core;
pub mod dsp;
pub mod eval;
pub mod units;

#[cfg(feature = "ai")]
pub mod ai;

// Re-export primary types for convenience
pub use crate::core::{ConversionResult, Currency, Unit, UnitCategory, Value};
pub use crate::eval::{EvalError, Evaluator, Token, tokenize};
pub use crate::units::{UnitError, UnitRegistry};

#[cfg(feature = "ai")]
pub use crate::ai::{
    AiError, CalculationHistory, CurrencyConverter, CurrencyResult, HistoryEntry, NlParser,
    ParsedQuery,
};
