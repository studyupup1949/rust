//! Experiment management for A/B tests.

use serde::{Deserialize, Serialize};

use crate::stats::{chi_squared_test, welch_t_test, ChiSquaredResult, WelchTResult};

/// A variant in an A/B test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    pub name: String,
    pub successes: usize,
    pub trials: usize,
    pub values: Vec<f64>,
}

impl Variant {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            successes: 0,
            trials: 0,
            values: Vec::new(),
        }
    }

    pub fn with_conversion(mut self, successes: usize, trials: usize) -> Self {
        self.successes = successes;
        self.trials = trials;
        self
    }

    pub fn with_values(mut self, values: Vec<f64>) -> Self {
        self.values = values;
        self
    }

    pub fn conversion_rate(&self) -> f64 {
        if self.trials == 0 {
            0.0
        } else {
            self.successes as f64 / self.trials as f64
        }
    }

    pub fn mean(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.values.iter().sum::<f64>() / self.values.len() as f64
        }
    }

    pub fn std_dev(&self) -> f64 {
        if self.values.len() < 2 {
            return 0.0;
        }
        let mean = self.mean();
        let var = self.values.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
            / (self.values.len() - 1) as f64;
        var.sqrt()
    }

    pub fn add_conversion(&mut self) {
        self.successes += 1;
        self.trials += 1;
    }

    pub fn add_non_conversion(&mut self) {
        self.trials += 1;
    }

    pub fn add_value(&mut self, v: f64) {
        self.values.push(v);
    }
}

/// Summary statistics for a variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantSummary {
    pub name: String,
    pub conversion_rate: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub n: usize,
}

impl From<&Variant> for VariantSummary {
    fn from(v: &Variant) -> Self {
        Self {
            name: v.name.clone(),
            conversion_rate: v.conversion_rate(),
            mean: v.mean(),
            std_dev: v.std_dev(),
            n: v.trials.max(v.values.len()),
        }
    }
}

/// Full experiment report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentReport {
    pub name: String,
    pub variant_summaries: Vec<VariantSummary>,
    pub chi_squared: Option<ChiSquaredResult>,
    pub welch_t: Option<WelchTResult>,
    pub winner: Option<String>,
    pub recommendation: String,
}

/// An A/B testing experiment.
#[derive(Debug, Clone)]
pub struct Experiment {
    pub name: String,
    pub variants: Vec<Variant>,
}

impl Experiment {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            variants: Vec::new(),
        }
    }

    pub fn add_variant(&mut self, variant: Variant) {
        self.variants.push(variant);
    }

    pub fn variant(&self, name: &str) -> Option<&Variant> {
        self.variants.iter().find(|v| v.name == name)
    }

    pub fn variant_mut(&mut self, name: &str) -> Option<&mut Variant> {
        self.variants.iter_mut().find(|v| v.name == name)
    }

    /// Run chi-squared test on first two variants (conversion data).
    pub fn run_chi_squared(&self) -> Option<ChiSquaredResult> {
        if self.variants.len() < 2 {
            return None;
        }
        let a = &self.variants[0];
        let b = &self.variants[1];
        if a.trials == 0 || b.trials == 0 {
            return None;
        }
        Some(chi_squared_test(
            a.successes,
            a.trials,
            b.successes,
            b.trials,
        ))
    }

    /// Run Welch's t-test on first two variants (continuous data).
    pub fn run_welch_t(&self) -> Option<WelchTResult> {
        if self.variants.len() < 2 {
            return None;
        }
        let a = &self.variants[0];
        let b = &self.variants[1];
        if a.values.is_empty() || b.values.is_empty() {
            return None;
        }
        Some(welch_t_test(&a.values, &b.values))
    }

    /// Generate full experiment report.
    pub fn report(&self) -> ExperimentReport {
        let chi2 = self.run_chi_squared();
        let welch = self.run_welch_t();

        let summaries: Vec<VariantSummary> =
            self.variants.iter().map(VariantSummary::from).collect();

        let winner = if let Some(ref w) = welch {
            if w.significant {
                Some(if w.mean_a > w.mean_b {
                    self.variants[0].name.clone()
                } else {
                    self.variants[1].name.clone()
                })
            } else {
                None
            }
        } else if let Some(ref c) = chi2 {
            if c.significant {
                let rate_a = self.variants[0].conversion_rate();
                let rate_b = self.variants[1].conversion_rate();
                Some(if rate_a > rate_b {
                    self.variants[0].name.clone()
                } else {
                    self.variants[1].name.clone()
                })
            } else {
                None
            }
        } else {
            None
        };

        let recommendation = match &winner {
            Some(w) => format!("Winner: {} — statistically significant difference detected", w),
            None => "No statistically significant difference. Continue collecting data or stop the test.".into(),
        };

        ExperimentReport {
            name: self.name.clone(),
            variant_summaries: summaries,
            chi_squared: chi2,
            welch_t: welch,
            winner,
            recommendation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variant_conversion_rate() {
        let v = Variant::new("A").with_conversion(80, 100);
        assert!((v.conversion_rate() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_variant_mean() {
        let v = Variant::new("A").with_values(vec![10.0, 20.0, 30.0]);
        assert!((v.mean() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_experiment_chi_squared() {
        let mut exp = Experiment::new("test");
        exp.add_variant(Variant::new("A").with_conversion(100, 200));
        exp.add_variant(Variant::new("B").with_conversion(150, 200));
        let result = exp.run_chi_squared().unwrap();
        assert!(result.significant);
    }

    #[test]
    fn test_experiment_welch_t() {
        let mut exp = Experiment::new("test");
        exp.add_variant(Variant::new("A").with_values(vec![10.0, 11.0, 12.0, 13.0, 14.0]));
        exp.add_variant(Variant::new("B").with_values(vec![20.0, 21.0, 22.0, 23.0, 24.0]));
        let result = exp.run_welch_t().unwrap();
        assert!((result.mean_a - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_report() {
        let mut exp = Experiment::new("test");
        exp.add_variant(Variant::new("control").with_conversion(100, 200));
        exp.add_variant(Variant::new("treatment").with_conversion(150, 200));
        let report = exp.report();
        assert!(report.winner.is_some());
        assert_eq!(report.winner.as_deref(), Some("treatment"));
    }

    #[test]
    fn test_add_conversion() {
        let mut v = Variant::new("A");
        v.add_conversion();
        v.add_conversion();
        v.add_non_conversion();
        assert_eq!(v.successes, 2);
        assert_eq!(v.trials, 3);
    }
}
