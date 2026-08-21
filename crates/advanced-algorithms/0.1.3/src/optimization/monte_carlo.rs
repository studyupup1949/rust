//! Monte Carlo Integration
//!
//! Uses random sampling to estimate integrals, especially useful for
//! high-dimensional integrals that are difficult to compute analytically.
//!
//! Supports parallel processing for improved performance.
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::optimization::monte_carlo;
//!
//! // Estimate integral of x^2 from 0 to 1
//! let f = |x: &[f64]| x[0] * x[0];
//! let bounds = vec![(0.0, 1.0)];
//!
//! let result = monte_carlo::integrate(f, &bounds, 100000).unwrap();
//! // Should be close to 1/3 ≈ 0.333...
//! assert!((result.value - 0.333).abs() < 0.01);
//! ```

use crate::{AlgorithmError, Result};
use rand::Rng;
use rayon::prelude::*;

/// Performs Monte Carlo integration
///
/// # Arguments
///
/// * `f` - The function to integrate
/// * `bounds` - Integration bounds for each dimension [(min, max), ...]
/// * `n_samples` - Number of random samples to use
///
/// # Returns
///
/// Integration result with estimate and error
pub fn integrate<F>(
    f: F,
    bounds: &[(f64, f64)],
    n_samples: usize,
) -> Result<IntegrationResult>
where
    F: Fn(&[f64]) -> f64 + Sync,
{
    if bounds.is_empty() {
        return Err(AlgorithmError::InvalidInput(
            "Must provide at least one dimension".to_string()
        ));
    }
    
    if n_samples == 0 {
        return Err(AlgorithmError::InvalidInput(
            "Number of samples must be positive".to_string()
        ));
    }
    
    // Validate bounds
    for (min, max) in bounds {
        if min >= max {
            return Err(AlgorithmError::InvalidInput(
                format!("Invalid bounds: {} >= {}", min, max)
            ));
        }
    }
    
    // Compute volume
    let volume: f64 = bounds.iter()
        .map(|(min, max)| max - min)
        .product();
    
    // Use parallel sampling for large n_samples
    if n_samples >= 10000 {
        integrate_parallel(f, bounds, n_samples, volume)
    } else {
        integrate_sequential(f, bounds, n_samples, volume)
    }
}

fn integrate_sequential<F>(
    f: F,
    bounds: &[(f64, f64)],
    n_samples: usize,
    volume: f64,
) -> Result<IntegrationResult>
where
    F: Fn(&[f64]) -> f64,
{
    let _dim = bounds.len();
    let mut rng = rand::thread_rng();
    
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    
    for _ in 0..n_samples {
        let point: Vec<f64> = bounds.iter()
            .map(|(min, max)| rng.gen_range(*min..*max))
            .collect();
        
        let value = f(&point);
        sum += value;
        sum_sq += value * value;
    }
    
    let mean = sum / n_samples as f64;
    let variance = (sum_sq / n_samples as f64) - (mean * mean);
    
    let integral = volume * mean;
    let error = volume * (variance / n_samples as f64).sqrt();
    
    Ok(IntegrationResult {
        value: integral,
        error,
        n_samples,
        variance,
    })
}

fn integrate_parallel<F>(
    f: F,
    bounds: &[(f64, f64)],
    n_samples: usize,
    volume: f64,
) -> Result<IntegrationResult>
where
    F: Fn(&[f64]) -> f64 + Sync,
{
    let _dim = bounds.len();
    
    // Divide work among threads
    let chunk_size = (n_samples / rayon::current_num_threads()).max(1000);
    let n_chunks = n_samples.div_ceil(chunk_size);
    
    let results: Vec<(f64, f64, usize)> = (0..n_chunks)
        .into_par_iter()
        .map(|chunk_idx| {
            let mut rng = rand::thread_rng();
            let start = chunk_idx * chunk_size;
            let end = (start + chunk_size).min(n_samples);
            let chunk_n = end - start;
            
            let mut sum = 0.0;
            let mut sum_sq = 0.0;
            
            for _ in 0..chunk_n {
                let point: Vec<f64> = bounds.iter()
                    .map(|(min, max)| rng.gen_range(*min..*max))
                    .collect();
                
                let value = f(&point);
                sum += value;
                sum_sq += value * value;
            }
            
            (sum, sum_sq, chunk_n)
        })
        .collect();
    
    // Combine results
    let total_sum: f64 = results.iter().map(|(s, _, _)| s).sum();
    let total_sum_sq: f64 = results.iter().map(|(_, sq, _)| sq).sum();
    let total_n: usize = results.iter().map(|(_, _, n)| n).sum();
    
    let mean = total_sum / total_n as f64;
    let variance = (total_sum_sq / total_n as f64) - (mean * mean);
    
    let integral = volume * mean;
    let error = volume * (variance / total_n as f64).sqrt();
    
    Ok(IntegrationResult {
        value: integral,
        error,
        n_samples: total_n,
        variance,
    })
}

/// Monte Carlo integration with importance sampling
///
/// # Arguments
///
/// * `f` - The function to integrate
/// * `proposal` - Proposal distribution for sampling
/// * `pdf` - Probability density function of proposal distribution
/// * `bounds` - Integration bounds
/// * `n_samples` - Number of samples
pub fn integrate_importance_sampling<F, P, Q>(
    f: F,
    proposal: P,
    pdf: Q,
    bounds: &[(f64, f64)],
    n_samples: usize,
) -> Result<IntegrationResult>
where
    F: Fn(&[f64]) -> f64 + Sync,
    P: Fn() -> Vec<f64> + Sync,
    Q: Fn(&[f64]) -> f64 + Sync,
{
    let _volume: f64 = bounds.iter()
        .map(|(min, max)| max - min)
        .product();
    
    let results: Vec<f64> = (0..n_samples)
        .into_par_iter()
        .map(|_| {
            let point = proposal();
            let f_val = f(&point);
            let p_val = pdf(&point);
            
            if p_val > 1e-10 {
                f_val / p_val
            } else {
                0.0
            }
        })
        .collect();
    
    let sum: f64 = results.iter().sum();
    let sum_sq: f64 = results.iter().map(|x| x * x).sum();
    
    let mean = sum / n_samples as f64;
    let variance = (sum_sq / n_samples as f64) - (mean * mean);
    
    let integral = mean;
    let error = (variance / n_samples as f64).sqrt();
    
    Ok(IntegrationResult {
        value: integral,
        error,
        n_samples,
        variance,
    })
}

/// Result of Monte Carlo integration
#[derive(Debug, Clone)]
pub struct IntegrationResult {
    /// Estimated integral value
    pub value: f64,
    /// Estimated error (standard deviation)
    pub error: f64,
    /// Number of samples used
    pub n_samples: usize,
    /// Variance of samples
    pub variance: f64,
}

impl IntegrationResult {
    /// Returns the relative error (error / value)
    pub fn relative_error(&self) -> f64 {
        if self.value.abs() > 1e-10 {
            self.error / self.value.abs()
        } else {
            self.error
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    
    #[test]
    fn test_simple_integral() {
        // Integrate x^2 from 0 to 1, should be 1/3
        let f = |x: &[f64]| x[0] * x[0];
        let bounds = vec![(0.0, 1.0)];
        
        let result = integrate(f, &bounds, 100000).unwrap();
        
        assert!((result.value - 1.0/3.0).abs() < 0.01);
    }
    
    #[test]
    fn test_multidimensional() {
        // Integrate x*y over [0,1]×[0,1], should be 1/4
        let f = |x: &[f64]| x[0] * x[1];
        let bounds = vec![(0.0, 1.0), (0.0, 1.0)];
        
        let result = integrate(f, &bounds, 100000).unwrap();
        
        assert!((result.value - 0.25).abs() < 0.01);
    }
    
    #[test]
    fn test_circle_area() {
        // Estimate π by integrating over unit circle
        let f = |x: &[f64]| {
            if x[0]*x[0] + x[1]*x[1] <= 1.0 {
                1.0
            } else {
                0.0
            }
        };
        let bounds = vec![(-1.0, 1.0), (-1.0, 1.0)];
        
        let result = integrate(f, &bounds, 100000).unwrap();
        
        // Area of unit circle is π, area of square is 4
        // So integral should be π
        assert!((result.value - PI).abs() < 0.1);
    }
    
    #[test]
    fn test_parallel() {
        let f = |x: &[f64]| x[0] * x[0];
        let bounds = vec![(0.0, 1.0)];
        
        let result = integrate(f, &bounds, 100000).unwrap();
        
        assert!((result.value - 1.0/3.0).abs() < 0.01);
        assert!(result.error > 0.0);
    }
}
