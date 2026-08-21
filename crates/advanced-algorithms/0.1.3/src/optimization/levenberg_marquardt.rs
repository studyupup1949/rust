//! Levenberg-Marquardt Algorithm
//!
//! A sophisticated algorithm for solving non-linear least squares problems,
//! combining Gauss-Newton method with gradient descent. It's widely used in
//! curve fitting, computer vision, and parameter estimation.
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::optimization::levenberg_marquardt::LevenbergMarquardt;
//!
//! // Fit data to y = a*exp(b*x)
//! let x_data = vec![0.0, 1.0, 2.0, 3.0, 4.0];
//! let y_data = vec![1.0, 2.7, 7.4, 20.1, 54.6];
//!
//! // Residual function
//! let residual = |params: &[f64], i: usize| {
//!     let prediction = params[0] * (params[1] * x_data[i]).exp();
//!     y_data[i] - prediction
//! };
//!
//! let lm = LevenbergMarquardt::new();
//! let result = lm.fit(residual, &[1.0, 1.0], x_data.len()).unwrap();
//! ```

use crate::{AlgorithmError, Result};

/// Levenberg-Marquardt optimizer for non-linear least squares
pub struct LevenbergMarquardt {
    max_iterations: usize,
    tolerance: f64,
    lambda_initial: f64,
    lambda_factor: f64,
    epsilon: f64,
}

impl Default for LevenbergMarquardt {
    fn default() -> Self {
        LevenbergMarquardt {
            max_iterations: 100,
            tolerance: 1e-8,
            lambda_initial: 1e-3,
            lambda_factor: 10.0,
            epsilon: 1e-8,
        }
    }
}

impl LevenbergMarquardt {
    /// Creates a new Levenberg-Marquardt optimizer
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Sets maximum iterations
    pub fn with_max_iterations(mut self, max_iter: usize) -> Self {
        self.max_iterations = max_iter;
        self
    }
    
    /// Sets convergence tolerance
    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = tol;
        self
    }
    
    /// Fits parameters to minimize sum of squared residuals
    ///
    /// # Arguments
    ///
    /// * `residual_fn` - Function that computes residual for data point i
    /// * `initial_params` - Initial parameter guess
    /// * `n_data` - Number of data points
    ///
    /// # Returns
    ///
    /// Fitted parameters
    pub fn fit<F>(
        &self,
        residual_fn: F,
        initial_params: &[f64],
        n_data: usize,
    ) -> Result<FitResult>
    where
        F: Fn(&[f64], usize) -> f64,
    {
        let n_params = initial_params.len();
        let mut params = initial_params.to_vec();
        let mut lambda = self.lambda_initial;
        
        let mut prev_cost = self.compute_cost(&residual_fn, &params, n_data);
        
        for iteration in 0..self.max_iterations {
            // Compute Jacobian and residuals
            let (jacobian, residuals) = self.compute_jacobian_and_residuals(
                &residual_fn,
                &params,
                n_data,
            );
            
            // Compute J^T * J and J^T * r
            let jtj = self.multiply_jt_j(&jacobian);
            let jtr = self.multiply_jt_r(&jacobian, &residuals);
            
            // Add lambda to diagonal (Levenberg-Marquardt modification)
            let mut jtj_lambda = jtj.clone();
            for i in 0..n_params {
                jtj_lambda[i][i] += lambda * (1.0 + jtj[i][i]);
            }
            
            // Solve (J^T * J + λI) * Δ = -J^T * r
            let delta = match self.solve_linear_system(&jtj_lambda, &jtr) {
                Ok(d) => d,
                Err(_) => {
                    lambda *= self.lambda_factor;
                    continue;
                }
            };
            
            // Try new parameters
            let mut new_params = Vec::with_capacity(n_params);
            for i in 0..n_params {
                new_params.push(params[i] - delta[i]);
            }
            
            let new_cost = self.compute_cost(&residual_fn, &new_params, n_data);
            
            if new_cost < prev_cost {
                // Accept the step
                let improvement = (prev_cost - new_cost) / prev_cost;
                
                params = new_params;
                prev_cost = new_cost;
                lambda /= self.lambda_factor;
                
                // Check for convergence
                if improvement < self.tolerance {
                    return Ok(FitResult {
                        parameters: params,
                        cost: prev_cost,
                        iterations: iteration + 1,
                        converged: true,
                    });
                }
            } else {
                // Reject the step and increase lambda
                lambda *= self.lambda_factor;
            }
        }
        
        Ok(FitResult {
            parameters: params,
            cost: prev_cost,
            iterations: self.max_iterations,
            converged: false,
        })
    }
    
    fn compute_cost<F>(&self, residual_fn: &F, params: &[f64], n_data: usize) -> f64
    where
        F: Fn(&[f64], usize) -> f64,
    {
        let mut sum = 0.0;
        for i in 0..n_data {
            let r = residual_fn(params, i);
            sum += r * r;
        }
        sum / 2.0
    }
    
    fn compute_jacobian_and_residuals<F>(
        &self,
        residual_fn: &F,
        params: &[f64],
        n_data: usize,
    ) -> (Vec<Vec<f64>>, Vec<f64>)
    where
        F: Fn(&[f64], usize) -> f64,
    {
        let n_params = params.len();
        let mut jacobian = vec![vec![0.0; n_params]; n_data];
        let mut residuals = vec![0.0; n_data];
        
        for i in 0..n_data {
            residuals[i] = residual_fn(params, i);
            
            // Numerical Jacobian
            for j in 0..n_params {
                let mut params_plus = params.to_vec();
                params_plus[j] += self.epsilon;
                
                let r_plus = residual_fn(&params_plus, i);
                jacobian[i][j] = (r_plus - residuals[i]) / self.epsilon;
            }
        }
        
        (jacobian, residuals)
    }
    
    fn multiply_jt_j(&self, jacobian: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n_data = jacobian.len();
        let n_params = jacobian[0].len();
        
        let mut result = vec![vec![0.0; n_params]; n_params];
        
        for i in 0..n_params {
            for j in 0..n_params {
                #[allow(clippy::needless_range_loop)]
                for k in 0..n_data {
                    result[i][j] += jacobian[k][i] * jacobian[k][j];
                }
            }
        }
        
        result
    }
    
    fn multiply_jt_r(&self, jacobian: &[Vec<f64>], residuals: &[f64]) -> Vec<f64> {
        let n_data = jacobian.len();
        let n_params = jacobian[0].len();
        
        let mut result = vec![0.0; n_params];
        
        for i in 0..n_params {
            for j in 0..n_data {
                result[i] += jacobian[j][i] * residuals[j];
            }
        }
        
        result
    }
    
    fn solve_linear_system(&self, a: &[Vec<f64>], b: &[f64]) -> Result<Vec<f64>> {
        // Simple Gaussian elimination with partial pivoting
        let n = a.len();
        let mut a = a.to_vec();
        let mut b = b.to_vec();
        
        // Forward elimination
        for k in 0..n - 1 {
            // Find pivot
            let mut max_idx = k;
            for i in k + 1..n {
                if a[i][k].abs() > a[max_idx][k].abs() {
                    max_idx = i;
                }
            }
            
            // Swap rows
            if max_idx != k {
                a.swap(k, max_idx);
                b.swap(k, max_idx);
            }
            
            if a[k][k].abs() < 1e-14 {
                return Err(AlgorithmError::NumericalInstability(
                    "Matrix is singular".to_string()
                ));
            }
            
            // Eliminate
            for i in k + 1..n {
                let factor = a[i][k] / a[k][k];
                #[allow(clippy::needless_range_loop)]
                for j in k..n {
                    a[i][j] -= factor * a[k][j];
                }
                b[i] -= factor * b[k];
            }
        }
        
        // Back substitution
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            if a[i][i].abs() < 1e-14 {
                return Err(AlgorithmError::NumericalInstability(
                    "Matrix is singular".to_string()
                ));
            }
            
            let mut sum = b[i];
            for j in i + 1..n {
                sum -= a[i][j] * x[j];
            }
            x[i] = sum / a[i][i];
        }
        
        Ok(x)
    }
}

/// Result of curve fitting
#[derive(Debug, Clone)]
pub struct FitResult {
    /// Fitted parameter values
    pub parameters: Vec<f64>,
    /// Final cost (sum of squared residuals / 2)
    pub cost: f64,
    /// Number of iterations performed
    pub iterations: usize,
    /// Whether the algorithm converged
    pub converged: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_linear_fit() {
        // Fit y = a*x + b to nearly perfect line with slight noise
        let x_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y_data = vec![2.01, 3.98, 6.02, 7.97, 10.03]; // y ≈ 2x with tiny noise
        
        let residual = |params: &[f64], i: usize| {
            let prediction = params[0] * x_data[i] + params[1];
            y_data[i] - prediction
        };
        
        let lm = LevenbergMarquardt::new();
        let result = lm.fit(residual, &[1.0, 0.0], x_data.len()).unwrap();
        
        assert!(result.converged);
        assert!((result.parameters[0] - 2.0).abs() < 0.1); // slope
        assert!(result.parameters[1].abs() < 0.1); // intercept
    }
    
    #[test]
    fn test_exponential_fit() {
        // Fit y = a*exp(b*x)
        let x_data = vec![0.0, 1.0, 2.0, 3.0];
        let y_data = vec![1.0, 2.7183, 7.3891, 20.0855]; // approximately e^x
        
        let residual = |params: &[f64], i: usize| {
            let prediction = params[0] * (params[1] * x_data[i]).exp();
            y_data[i] - prediction
        };
        
        let lm = LevenbergMarquardt::new().with_max_iterations(200);
        let result = lm.fit(residual, &[1.0, 1.0], x_data.len()).unwrap();
        
        assert!((result.parameters[0] - 1.0).abs() < 0.1);
        assert!((result.parameters[1] - 1.0).abs() < 0.1);
    }
}
