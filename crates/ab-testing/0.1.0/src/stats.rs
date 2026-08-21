//! Statistical tests: chi-squared, Welch's t-test, confidence intervals.

use serde::{Deserialize, Serialize};

/// Chi-squared test result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChiSquaredResult {
    pub chi2: f64,
    pub p_value: f64,
    pub degrees_of_freedom: usize,
    pub significant: bool,
    pub critical_value: f64,
}

/// Welch's t-test result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelchTResult {
    pub t_statistic: f64,
    pub p_value: f64,
    pub degrees_of_freedom: f64,
    pub significant: bool,
    pub mean_a: f64,
    pub mean_b: f64,
    pub difference: f64,
    pub ci_lower: f64,
    pub ci_upper: f64,
}

/// Chi-squared test for two proportions.
pub fn chi_squared_test(
    successes_a: usize,
    trials_a: usize,
    successes_b: usize,
    trials_b: usize,
) -> ChiSquaredResult {
    let a_fail = trials_a - successes_a;
    let b_fail = trials_b - successes_b;
    let total = trials_a + trials_b;

    let _prop_a = successes_a as f64 / trials_a as f64;
    let _prop_b = successes_b as f64 / trials_b as f64;
    let pooled = (successes_a + successes_b) as f64 / total as f64;

    let expected_a_s = pooled * trials_a as f64;
    let expected_a_f = (1.0 - pooled) * trials_a as f64;
    let expected_b_s = pooled * trials_b as f64;
    let expected_b_f = (1.0 - pooled) * trials_b as f64;

    let chi2 =
        if expected_a_s > 0.0 && expected_a_f > 0.0 && expected_b_s > 0.0 && expected_b_f > 0.0 {
            (successes_a as f64 - expected_a_s).powi(2) / expected_a_s
                + (a_fail as f64 - expected_a_f).powi(2) / expected_a_f
                + (successes_b as f64 - expected_b_s).powi(2) / expected_b_s
                + (b_fail as f64 - expected_b_f).powi(2) / expected_b_f
        } else {
            0.0
        };

    let df = 1;
    let critical = 3.841; // chi2 critical for df=1, alpha=0.05
    let p_value = chi2_p_value(chi2, df);
    let significant = chi2 > critical;

    ChiSquaredResult {
        chi2,
        p_value,
        degrees_of_freedom: df,
        significant,
        critical_value: critical,
    }
}

/// Welch's t-test for two independent samples.
pub fn welch_t_test(a: &[f64], b: &[f64]) -> WelchTResult {
    let n_a = a.len() as f64;
    let n_b = b.len() as f64;
    let mean_a = a.iter().sum::<f64>() / n_a;
    let mean_b = b.iter().sum::<f64>() / n_b;

    let var_a = if n_a > 1.0 {
        a.iter().map(|x| (x - mean_a).powi(2)).sum::<f64>() / (n_a - 1.0)
    } else {
        0.0
    };
    let var_b = if n_b > 1.0 {
        b.iter().map(|x| (x - mean_b).powi(2)).sum::<f64>() / (n_b - 1.0)
    } else {
        0.0
    };

    let se = (var_a / n_a + var_b / n_b).sqrt();
    let t = if se > 0.0 {
        (mean_a - mean_b) / se
    } else {
        0.0
    };

    // Welch-Satterthwaite degrees of freedom
    let num = (var_a / n_a + var_b / n_b).powi(2);
    let den = if var_a > 0.0 && n_a > 1.0 && var_b > 0.0 && n_b > 1.0 {
        (var_a / n_a).powi(2) / (n_a - 1.0) + (var_b / n_b).powi(2) / (n_b - 1.0)
    } else {
        1.0
    };
    let df = if den > 0.0 {
        num / den
    } else {
        n_a + n_b - 2.0
    };

    let p_value = t_test_p_value(t.abs(), df);
    let significant = p_value < 0.05;
    let diff = mean_a - mean_b;

    let (ci_lower, ci_upper) = if se > 0.0 {
        let margin = 1.96 * se; // approximate 95% CI
        (diff - margin, diff + margin)
    } else {
        (diff, diff)
    };

    WelchTResult {
        t_statistic: t,
        p_value,
        degrees_of_freedom: df,
        significant,
        mean_a,
        mean_b,
        difference: diff,
        ci_lower,
        ci_upper,
    }
}

/// Confidence interval for a mean.
pub fn confidence_interval(data: &[f64], confidence: f64) -> (f64, f64, f64) {
    let n = data.len() as f64;
    if n == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let mean = data.iter().sum::<f64>() / n;
    let var = if n > 1.0 {
        data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)
    } else {
        0.0
    };
    let se = (var / n).sqrt();
    // z-value approximation for confidence level
    let z = match confidence {
        c if c >= 0.99 => 2.576,
        c if c >= 0.95 => 1.96,
        c if c >= 0.90 => 1.645,
        _ => 1.96,
    };
    let margin = z * se;
    (mean, mean - margin, mean + margin)
}

/// Approximate chi-squared p-value (df=1 only).
fn chi2_p_value(chi2: f64, _df: usize) -> f64 {
    if chi2 <= 0.0 {
        return 1.0;
    }
    // Using the regularized incomplete gamma function approximation
    let x = chi2 / 2.0;
    let p = (-x + x.ln() * 0.5).exp();
    p.clamp(0.0, 1.0)
}

/// Approximate two-tailed p-value from t-statistic.
fn t_test_p_value(t_abs: f64, df: f64) -> f64 {
    if t_abs <= 0.0 {
        return 1.0;
    }
    // Very rough approximation
    let p = 2.0 * (1.0 - (t_abs / (df + t_abs * t_abs).sqrt()).atan() * 2.0 / std::f64::consts::PI);
    p.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chi_squared_significant() {
        let result = chi_squared_test(100, 200, 150, 200);
        assert!(result.chi2 > 0.0);
        assert!(result.significant);
    }

    #[test]
    fn test_chi_squared_not_significant() {
        let result = chi_squared_test(100, 200, 105, 200);
        assert!(!result.significant);
    }

    #[test]
    fn test_welch_t_different_means() {
        let a = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let b = vec![20.0, 21.0, 22.0, 23.0, 24.0];
        let result = welch_t_test(&a, &b);
        assert!(result.t_statistic < 0.0); // a < b
        assert!((result.mean_a - 12.0).abs() < 1e-10);
        assert!((result.mean_b - 22.0).abs() < 1e-10);
    }

    #[test]
    fn test_welch_t_same_means() {
        let a = vec![10.0, 11.0, 12.0];
        let b = vec![10.0, 11.0, 12.0];
        let result = welch_t_test(&a, &b);
        assert!((result.difference).abs() < 1e-10);
    }

    #[test]
    fn test_confidence_interval() {
        let data = vec![10.0, 12.0, 14.0, 16.0, 18.0];
        let (mean, lo, hi) = confidence_interval(&data, 0.95);
        assert!((mean - 14.0).abs() < 1e-10);
        assert!(lo < mean);
        assert!(hi > mean);
    }
}
