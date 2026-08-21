//! Gradient Descent Optimization
//!
//! Gradient descent is a first-order iterative optimization algorithm for finding
//! local minima of differentiable functions. It's the foundation of training neural
//! networks and machine learning models.
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::optimization::gradient_descent::{GradientDescent, LearningRate};
//!
//! // Minimize f(x) = x^2 + 2x + 1
//! let f = |x: &[f64]| x[0] * x[0] + 2.0 * x[0] + 1.0;
//! let grad_f = |x: &[f64]| vec![2.0 * x[0] + 2.0];
//!
//! let gd = GradientDescent::new()
//!     .with_learning_rate(LearningRate::Constant(0.1))
//!     .with_max_iterations(1000);
//!
//! let result = gd.minimize(f, grad_f, &[10.0]).unwrap();
//! // Should converge to x ≈ -1
//! ```

use crate::Result;

/// Learning rate strategy
#[derive(Debug, Clone)]
pub enum LearningRate {
    /// Constant learning rate
    Constant(f64),
    /// Decreasing: initial_rate / (1 + decay * iteration)
    Decreasing { initial: f64, decay: f64 },
    /// Exponential decay: initial_rate * (decay ^ iteration)
    Exponential { initial: f64, decay: f64 },
    /// Adaptive (AdaGrad-like): adapts per parameter
    Adaptive { initial: f64, epsilon: f64 },
}

impl LearningRate {
    fn get_rate(&self, iteration: usize) -> f64 {
        match self {
            LearningRate::Constant(rate) => *rate,
            LearningRate::Decreasing { initial, decay } => {
                initial / (1.0 + decay * iteration as f64)
            }
            LearningRate::Exponential { initial, decay } => {
                initial * decay.powi(iteration as i32)
            }
            LearningRate::Adaptive { initial, .. } => *initial,
        }
    }
}

/// Gradient descent optimizer configuration
pub struct GradientDescent {
    learning_rate: LearningRate,
    max_iterations: usize,
    tolerance: f64,
    momentum: f64,
    verbose: bool,
}

impl Default for GradientDescent {
    fn default() -> Self {
        GradientDescent {
            learning_rate: LearningRate::Constant(0.01),
            max_iterations: 1000,
            tolerance: 1e-6,
            momentum: 0.0,
            verbose: false,
        }
    }
}

impl GradientDescent {
    /// Creates a new gradient descent optimizer with default settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Sets the learning rate strategy
    pub fn with_learning_rate(mut self, lr: LearningRate) -> Self {
        self.learning_rate = lr;
        self
    }
    
    /// Sets the maximum number of iterations
    pub fn with_max_iterations(mut self, max_iter: usize) -> Self {
        self.max_iterations = max_iter;
        self
    }
    
    /// Sets the convergence tolerance
    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = tol;
        self
    }
    
    /// Sets the momentum coefficient (0.0 = no momentum)
    pub fn with_momentum(mut self, momentum: f64) -> Self {
        self.momentum = momentum;
        self
    }
    
    /// Enables verbose output
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
    
    /// Minimizes the given function
    ///
    /// # Arguments
    ///
    /// * `f` - The objective function to minimize
    /// * `grad_f` - The gradient of the objective function
    /// * `initial` - Initial parameter values
    ///
    /// # Returns
    ///
    /// Optimization result containing the final parameters
    pub fn minimize<F, G>(
        &self,
        f: F,
        grad_f: G,
        initial: &[f64],
    ) -> Result<OptimizationResult>
    where
        F: Fn(&[f64]) -> f64,
        G: Fn(&[f64]) -> Vec<f64>,
    {
        let mut x = initial.to_vec();
        let mut velocity = vec![0.0; x.len()];
        let mut accumulated_squared_gradients = vec![0.0; x.len()];
        
        let mut history = Vec::new();
        
        for iteration in 0..self.max_iterations {
            let current_value = f(&x);
            let gradient = grad_f(&x);
            
            if self.verbose {
                history.push((x.clone(), current_value));
            }
            
            // Check gradient norm for convergence
            let grad_norm: f64 = gradient.iter().map(|g| g * g).sum::<f64>().sqrt();
            
            if grad_norm < self.tolerance {
                return Ok(OptimizationResult {
                    parameters: x,
                    value: current_value,
                    iterations: iteration,
                    gradient_norm: grad_norm,
                    converged: true,
                    history,
                });
            }
            
            // Update parameters based on learning rate strategy
            let lr = self.learning_rate.get_rate(iteration);
            
            match &self.learning_rate {
                LearningRate::Adaptive { epsilon, .. } => {
                    // AdaGrad-style update
                    for i in 0..x.len() {
                        accumulated_squared_gradients[i] += gradient[i] * gradient[i];
                        let adjusted_lr = lr / (accumulated_squared_gradients[i].sqrt() + epsilon);
                        
                        velocity[i] = self.momentum * velocity[i] - adjusted_lr * gradient[i];
                        x[i] += velocity[i];
                    }
                }
                _ => {
                    // Standard update with momentum
                    for i in 0..x.len() {
                        velocity[i] = self.momentum * velocity[i] - lr * gradient[i];
                        x[i] += velocity[i];
                    }
                }
            }
        }
        
        let final_value = f(&x);
        let final_gradient = grad_f(&x);
        let grad_norm: f64 = final_gradient.iter().map(|g| g * g).sum::<f64>().sqrt();
        
        Ok(OptimizationResult {
            parameters: x,
            value: final_value,
            iterations: self.max_iterations,
            gradient_norm: grad_norm,
            converged: grad_norm < self.tolerance,
            history,
        })
    }
}

/// Result of optimization
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Final parameter values
    pub parameters: Vec<f64>,
    /// Final objective function value
    pub value: f64,
    /// Number of iterations performed
    pub iterations: usize,
    /// Final gradient norm
    pub gradient_norm: f64,
    /// Whether the algorithm converged
    pub converged: bool,
    /// History of (parameters, value) if verbose mode enabled
    pub history: Vec<(Vec<f64>, f64)>,
}

/// Stochastic Gradient Descent for batch optimization
pub struct StochasticGD {
    learning_rate: LearningRate,
    max_epochs: usize,
    batch_size: usize,
    tolerance: f64,
    momentum: f64,
}

impl StochasticGD {
    /// Creates a new SGD optimizer
    pub fn new(batch_size: usize) -> Self {
        StochasticGD {
            learning_rate: LearningRate::Decreasing {
                initial: 0.01,
                decay: 0.001,
            },
            max_epochs: 100,
            batch_size,
            tolerance: 1e-6,
            momentum: 0.9,
        }
    }
    
    /// Minimizes using mini-batch gradient descent
    ///
    /// # Arguments
    ///
    /// * `grad_f` - Gradient function that takes parameters and data indices
    /// * `initial` - Initial parameter values
    /// * `n_samples` - Total number of training samples
    pub fn minimize<G>(
        &self,
        grad_f: G,
        initial: &[f64],
        n_samples: usize,
    ) -> Result<OptimizationResult>
    where
        G: Fn(&[f64], &[usize]) -> Vec<f64> + Sync,
    {
        let mut x = initial.to_vec();
        let mut velocity = vec![0.0; x.len()];
        
        let mut total_iterations = 0;
        
        for _epoch in 0..self.max_epochs {
            // Shuffle indices (simplified - using sequential batches here)
            let n_batches = n_samples.div_ceil(self.batch_size);
            
            for batch_idx in 0..n_batches {
                let start = batch_idx * self.batch_size;
                let end = (start + self.batch_size).min(n_samples);
                let batch_indices: Vec<usize> = (start..end).collect();
                
                let gradient = grad_f(&x, &batch_indices);
                
                let lr = self.learning_rate.get_rate(total_iterations);
                
                for i in 0..x.len() {
                    velocity[i] = self.momentum * velocity[i] - lr * gradient[i];
                    x[i] += velocity[i];
                }
                
                total_iterations += 1;
            }
            
            // Check convergence (simplified - would need objective function)
            let grad_norm: f64 = velocity.iter().map(|v| v * v).sum::<f64>().sqrt();
            
            if grad_norm < self.tolerance {
                return Ok(OptimizationResult {
                    parameters: x,
                    value: 0.0,
                    iterations: total_iterations,
                    gradient_norm: grad_norm,
                    converged: true,
                    history: Vec::new(),
                });
            }
        }
        
        Ok(OptimizationResult {
            parameters: x,
            value: 0.0,
            iterations: total_iterations,
            gradient_norm: 0.0,
            converged: false,
            history: Vec::new(),
        })
    }
}

/// Numerical gradient computation (for testing)
pub fn numerical_gradient<F>(f: F, x: &[f64], epsilon: f64) -> Vec<f64>
where
    F: Fn(&[f64]) -> f64,
{
    let mut gradient = Vec::with_capacity(x.len());
    
    for i in 0..x.len() {
        let mut x_plus = x.to_vec();
        let mut x_minus = x.to_vec();
        
        x_plus[i] += epsilon;
        x_minus[i] -= epsilon;
        
        let grad_i = (f(&x_plus) - f(&x_minus)) / (2.0 * epsilon);
        gradient.push(grad_i);
    }
    
    gradient
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quadratic() {
        // Minimize f(x) = (x - 3)^2, minimum at x = 3
        let f = |x: &[f64]| (x[0] - 3.0) * (x[0] - 3.0);
        let grad_f = |x: &[f64]| vec![2.0 * (x[0] - 3.0)];
        
        let gd = GradientDescent::new()
            .with_learning_rate(LearningRate::Constant(0.1))
            .with_max_iterations(1000);
        
        let result = gd.minimize(f, grad_f, &[0.0]).unwrap();
        
        assert!(result.converged);
        assert!((result.parameters[0] - 3.0).abs() < 0.01);
    }
    
    #[test]
    fn test_rosenbrock() {
        // Rosenbrock function: f(x,y) = (1-x)^2 + 100(y-x^2)^2
        let f = |x: &[f64]| {
            let a = 1.0 - x[0];
            let b = x[1] - x[0] * x[0];
            a * a + 100.0 * b * b
        };
        
        let grad_f = |x: &[f64]| {
            vec![
                -2.0 * (1.0 - x[0]) - 400.0 * x[0] * (x[1] - x[0] * x[0]),
                200.0 * (x[1] - x[0] * x[0]),
            ]
        };
        
        let gd = GradientDescent::new()
            .with_learning_rate(LearningRate::Constant(0.001))
            .with_momentum(0.9)
            .with_max_iterations(10000);
        
        let result = gd.minimize(f, grad_f, &[0.0, 0.0]).unwrap();
        
        // Should converge near (1, 1)
        assert!(result.value < 0.1);
    }
    
    #[test]
    fn test_numerical_gradient() {
        let f = |x: &[f64]| x[0] * x[0] + 2.0 * x[1] * x[1];
        
        let num_grad = numerical_gradient(f, &[1.0, 2.0], 1e-5);
        let analytical_grad = vec![2.0 * 1.0, 4.0 * 2.0];
        
        for (n, a) in num_grad.iter().zip(analytical_grad.iter()) {
            assert!((n - a).abs() < 1e-4);
        }
    }
}
