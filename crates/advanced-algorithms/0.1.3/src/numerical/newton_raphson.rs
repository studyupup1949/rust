//! Newton-Raphson Method for Root Finding
//!
//! An iterative numerical method for finding roots (zeros) of real-valued functions.
//! The method uses the function's derivative to converge quadratically to a root.
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::numerical::newton_raphson;
//!
//! // Find the square root of 2 by finding root of f(x) = x² - 2
//! let f = |x: f64| x * x - 2.0;
//! let df = |x: f64| 2.0 * x;
//!
//! let root = newton_raphson::find_root(f, df, 1.0, 1e-10, 100).unwrap();
//! assert!((root - 2.0_f64.sqrt()).abs() < 1e-10);
//! ```

use crate::{AlgorithmError, Result};

/// Finds a root of a function using the Newton-Raphson method
///
/// # Arguments
///
/// * `f` - The function to find the root of
/// * `df` - The derivative of the function
/// * `x0` - Initial guess
/// * `tolerance` - Convergence tolerance
/// * `max_iterations` - Maximum number of iterations
///
/// # Returns
///
/// The approximate root of the function
///
/// # Errors
///
/// Returns error if:
/// - Maximum iterations reached without convergence
/// - Derivative is zero (division by zero)
/// - Solution diverges
pub fn find_root<F, G>(
    f: F,
    df: G,
    x0: f64,
    tolerance: f64,
    max_iterations: usize,
) -> Result<f64>
where
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    let mut x = x0;
    
    for iteration in 0..max_iterations {
        let fx = f(x);
        let dfx = df(x);
        
        // Check for zero derivative
        if dfx.abs() < 1e-14 {
            return Err(AlgorithmError::NumericalInstability(
                format!("Derivative too close to zero at x = {}", x)
            ));
        }
        
        // Newton-Raphson update
        let x_new = x - fx / dfx;
        
        // Check for divergence
        if x_new.is_nan() || x_new.is_infinite() {
            return Err(AlgorithmError::ConvergenceFailure(
                format!("Solution diverged at iteration {}", iteration)
            ));
        }
        
        // Check for convergence
        if (x_new - x).abs() < tolerance && fx.abs() < tolerance {
            return Ok(x_new);
        }
        
        x = x_new;
    }
    
    Err(AlgorithmError::ConvergenceFailure(
        format!("Failed to converge after {} iterations", max_iterations)
    ))
}

/// Configuration for Newton-Raphson solver
pub struct NewtonRaphsonConfig {
    /// Convergence tolerance
    pub tolerance: f64,
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Verbose output
    pub verbose: bool,
}

impl Default for NewtonRaphsonConfig {
    fn default() -> Self {
        NewtonRaphsonConfig {
            tolerance: 1e-10,
            max_iterations: 100,
            verbose: false,
        }
    }
}

/// Advanced Newton-Raphson solver with detailed results
pub struct NewtonRaphsonSolver<F, G>
where
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    f: F,
    df: G,
    config: NewtonRaphsonConfig,
}

impl<F, G> NewtonRaphsonSolver<F, G>
where
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    /// Creates a new Newton-Raphson solver
    pub fn new(f: F, df: G) -> Self {
        NewtonRaphsonSolver {
            f,
            df,
            config: NewtonRaphsonConfig::default(),
        }
    }
    
    /// Sets the configuration
    pub fn with_config(mut self, config: NewtonRaphsonConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Solves for the root starting from x0
    pub fn solve(&self, x0: f64) -> Result<SolutionResult> {
        let mut x = x0;
        let mut history = Vec::new();
        
        for iteration in 0..self.config.max_iterations {
            let fx = (self.f)(x);
            let dfx = (self.df)(x);
            
            if self.config.verbose {
                history.push((x, fx));
            }
            
            if dfx.abs() < 1e-14 {
                return Err(AlgorithmError::NumericalInstability(
                    format!("Derivative too close to zero at x = {}", x)
                ));
            }
            
            let x_new = x - fx / dfx;
            
            if x_new.is_nan() || x_new.is_infinite() {
                return Err(AlgorithmError::ConvergenceFailure(
                    format!("Solution diverged at iteration {}", iteration)
                ));
            }
            
            if (x_new - x).abs() < self.config.tolerance && fx.abs() < self.config.tolerance {
                return Ok(SolutionResult {
                    root: x_new,
                    iterations: iteration + 1,
                    final_error: fx.abs(),
                    history,
                });
            }
            
            x = x_new;
        }
        
        Err(AlgorithmError::ConvergenceFailure(
            format!("Failed to converge after {} iterations", self.config.max_iterations)
        ))
    }
}

/// Result of Newton-Raphson solving
#[derive(Debug, Clone)]
pub struct SolutionResult {
    /// The found root
    pub root: f64,
    /// Number of iterations taken
    pub iterations: usize,
    /// Final error value
    pub final_error: f64,
    /// History of (x, f(x)) pairs if verbose mode enabled
    pub history: Vec<(f64, f64)>,
}

/// Finds all roots in an interval using multiple starting points
///
/// # Arguments
///
/// * `f` - The function to find roots of
/// * `df` - The derivative of the function
/// * `start` - Start of the interval
/// * `end` - End of the interval
/// * `num_points` - Number of starting points to try
/// * `tolerance` - Convergence tolerance
///
/// # Returns
///
/// Vector of unique roots found
pub fn find_roots_in_interval<F, G>(
    f: F,
    df: G,
    start: f64,
    end: f64,
    num_points: usize,
    tolerance: f64,
) -> Vec<f64>
where
    F: Fn(f64) -> f64 + Copy,
    G: Fn(f64) -> f64 + Copy,
{
    let step = (end - start) / (num_points as f64);
    let mut roots = Vec::new();
    
    for i in 0..num_points {
        let x0 = start + (i as f64) * step;
        
        if let Ok(root) = find_root(f, df, x0, tolerance, 100) {
            // Check if this is a new root
            let is_new = roots.iter().all(|&r: &f64| (r - root).abs() > tolerance * 10.0);
            
            if is_new && root >= start && root <= end {
                roots.push(root);
            }
        }
    }
    
    roots.sort_by(|a, b| a.partial_cmp(b).unwrap());
    roots
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_square_root() {
        // Find sqrt(2) by solving x^2 - 2 = 0
        let f = |x: f64| x * x - 2.0;
        let df = |x: f64| 2.0 * x;
        
        let root = find_root(f, df, 1.0, 1e-10, 100).unwrap();
        assert!((root - 2.0_f64.sqrt()).abs() < 1e-10);
    }
    
    #[test]
    fn test_cubic() {
        // Find root of x^3 - x - 2 = 0 (root is approximately 1.5214)
        let f = |x: f64| x * x * x - x - 2.0;
        let df = |x: f64| 3.0 * x * x - 1.0;
        
        let root = find_root(f, df, 1.5, 1e-10, 100).unwrap();
        assert!(f(root).abs() < 1e-10);
    }
    
    #[test]
    fn test_solver() {
        let f = |x: f64| x * x - 4.0;
        let df = |x: f64| 2.0 * x;
        
        let solver = NewtonRaphsonSolver::new(f, df);
        let result = solver.solve(1.0).unwrap();
        
        assert!((result.root - 2.0).abs() < 1e-10);
        assert!(result.iterations > 0);
    }
    
    #[test]
    fn test_multiple_roots() {
        // x^2 - 1 = 0 has roots at -1 and 1
        let f = |x: f64| x * x - 1.0;
        let df = |x: f64| 2.0 * x;
        
        let roots = find_roots_in_interval(f, df, -2.0, 2.0, 10, 1e-10);
        assert_eq!(roots.len(), 2);
    }
}
