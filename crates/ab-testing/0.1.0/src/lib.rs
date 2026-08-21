//! ab-testing — A/B testing with chi-squared and Welch's t-test.

mod experiment;
mod stats;

pub use experiment::{Experiment, ExperimentReport, Variant, VariantSummary};
pub use stats::{
    chi_squared_test, confidence_interval, welch_t_test, ChiSquaredResult, WelchTResult,
};
