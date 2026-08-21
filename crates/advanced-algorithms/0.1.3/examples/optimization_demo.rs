//! Example demonstrating optimization algorithms

use advanced_algorithms::optimization::{levenberg_marquardt, monte_carlo};
use advanced_algorithms::optimization::gradient_descent::{GradientDescent, LearningRate};
use std::f64::consts::PI;

fn main() {
    println!("=== Optimization Algorithms Demo ===\n");
    
    // Gradient Descent Example
    println!("=== Gradient Descent ===");
    
    // Minimize quadratic function: f(x, y) = (x - 3)² + (y + 2)²
    // Minimum is at (3, -2) with value 0
    let quadratic = |x: &[f64]| {
        let dx = x[0] - 3.0;
        let dy = x[1] + 2.0;
        dx * dx + dy * dy
    };
    
    let grad_quadratic = |x: &[f64]| {
        vec![
            2.0 * (x[0] - 3.0),
            2.0 * (x[1] + 2.0),
        ]
    };
    
    let gd = GradientDescent::new()
        .with_learning_rate(LearningRate::Constant(0.1))
        .with_max_iterations(1000)
        .with_tolerance(1e-8);
    
    let result = gd.minimize(quadratic, grad_quadratic, &[0.0, 0.0]).unwrap();
    
    println!("Quadratic minimization:");
    println!("  Starting point: [0.0, 0.0]");
    println!("  Final point: [{:.6}, {:.6}]", result.parameters[0], result.parameters[1]);
    println!("  Final value: {:.10}", result.value);
    println!("  Iterations: {}", result.iterations);
    println!("  Converged: {}", result.converged);
    
    // Rosenbrock function (harder optimization problem)
    println!("\n=== Rosenbrock Function ===");
    
    let rosenbrock = |x: &[f64]| {
        let a = 1.0 - x[0];
        let b = x[1] - x[0] * x[0];
        a * a + 100.0 * b * b
    };
    
    let grad_rosenbrock = |x: &[f64]| {
        vec![
            -2.0 * (1.0 - x[0]) - 400.0 * x[0] * (x[1] - x[0] * x[0]),
            200.0 * (x[1] - x[0] * x[0]),
        ]
    };
    
    let gd_rosenbrock = GradientDescent::new()
        .with_learning_rate(LearningRate::Adaptive {
            initial: 0.001,
            epsilon: 1e-8,
        })
        .with_momentum(0.9)
        .with_max_iterations(10000);
    
    let result = gd_rosenbrock.minimize(rosenbrock, grad_rosenbrock, &[-1.0, 1.0]).unwrap();
    
    println!("Minimizing Rosenbrock function:");
    println!("  Final point: [{:.6}, {:.6}]", result.parameters[0], result.parameters[1]);
    println!("  Final value: {:.10}", result.value);
    println!("  Iterations: {}", result.iterations);
    
    // Levenberg-Marquardt Curve Fitting
    println!("\n=== Levenberg-Marquardt Curve Fitting ===");
    
    // Fit exponential: y = a * exp(b * x)
    let x_data = vec![0.0, 1.0, 2.0, 3.0, 4.0];
    let y_data = vec![1.0, 2.72, 7.39, 20.09, 54.60]; // Approximately e^x
    
    let residual = |params: &[f64], i: usize| {
        let prediction = params[0] * (params[1] * x_data[i]).exp();
        y_data[i] - prediction
    };
    
    let lm = levenberg_marquardt::LevenbergMarquardt::new()
        .with_max_iterations(100);
    
    let result = lm.fit(residual, &[1.0, 1.0], x_data.len()).unwrap();
    
    println!("Fitting y = a * exp(b * x):");
    println!("  Fitted parameters: a = {:.4}, b = {:.4}", 
             result.parameters[0], result.parameters[1]);
    println!("  Final cost: {:.6}", result.cost);
    println!("  Iterations: {}", result.iterations);
    println!("  Converged: {}", result.converged);
    
    // Verify fit
    println!("\n  Verification:");
    for i in 0..x_data.len() {
        let pred = result.parameters[0] * (result.parameters[1] * x_data[i]).exp();
        println!("    x={:.1}: y_actual={:.2}, y_pred={:.2}, error={:.4}",
                 x_data[i], y_data[i], pred, (y_data[i] - pred).abs());
    }
    
    // Monte Carlo Integration
    println!("\n=== Monte Carlo Integration ===");
    
    // Integrate x² from 0 to 1 (analytical result: 1/3)
    let f1 = |x: &[f64]| x[0] * x[0];
    let bounds1 = vec![(0.0, 1.0)];
    
    let mc_result = monte_carlo::integrate(f1, &bounds1, 100_000).unwrap();
    
    println!("∫₀¹ x² dx:");
    println!("  Estimated value: {:.6}", mc_result.value);
    println!("  Analytical value: {:.6}", 1.0/3.0);
    println!("  Error: {:.6}", (mc_result.value - 1.0/3.0).abs());
    println!("  Monte Carlo error estimate: ±{:.6}", mc_result.error);
    println!("  Relative error: {:.2}%", mc_result.relative_error() * 100.0);
    
    // 2D integration: ∫∫ x*y dx dy over [0,1]×[0,1] (analytical: 1/4)
    let f2 = |x: &[f64]| x[0] * x[1];
    let bounds2 = vec![(0.0, 1.0), (0.0, 1.0)];
    
    let mc_result2 = monte_carlo::integrate(f2, &bounds2, 100_000).unwrap();
    
    println!("\n∫₀¹∫₀¹ x*y dx dy:");
    println!("  Estimated value: {:.6}", mc_result2.value);
    println!("  Analytical value: {:.6}", 0.25);
    println!("  Error: {:.6}", (mc_result2.value - 0.25).abs());
    
    // Estimate π using Monte Carlo
    let circle_indicator = |x: &[f64]| {
        if x[0]*x[0] + x[1]*x[1] <= 1.0 {
            1.0
        } else {
            0.0
        }
    };
    let bounds_circle = vec![(-1.0, 1.0), (-1.0, 1.0)];
    
    let pi_result = monte_carlo::integrate(circle_indicator, &bounds_circle, 1_000_000).unwrap();
    
    println!("\nEstimating π:");
    println!("  Estimated value: {:.6}", pi_result.value);
    println!("  Actual value: {:.6}", PI);
    println!("  Error: {:.6}", (pi_result.value - PI).abs());
}
