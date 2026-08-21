//! # Advanced Algorithms Library
//!
//! A comprehensive Rust library providing high-performance implementations of advanced algorithms
//! across multiple domains including:
//!
//! - **Numerical Analysis & Linear Algebra**: FFT, QR Decomposition, Newton-Raphson, SVD
//! - **Number Theory & Cryptography**: Miller-Rabin Primality Test, Extended Euclidean Algorithm, Mersenne Twister
//! - **Optimization & Statistics**: Gradient Descent, Levenberg-Marquardt, Monte Carlo Integration
//! - **Graph Algorithms**: Dijkstra's, A*, Bellman-Ford, Floyd-Warshall
//!
//! ## Features
//!
//! - **High Performance**: Optimized implementations with multi-threading support via Rayon
//! - **Well Documented**: Comprehensive documentation with examples for each algorithm
//! - **Type Safe**: Leverages Rust's type system for correctness
//! - **Numerically Stable**: Implementations prioritize numerical stability
//!
//! ## Quick Start
//!
//! ```rust
//! use advanced_algorithms::numerical::fft;
//! use advanced_algorithms::optimization::gradient_descent;
//! use advanced_algorithms::number_theory::miller_rabin;
//!
//! // Fast Fourier Transform
//! let signal = vec![1.0, 2.0, 3.0, 4.0];
//! let spectrum = fft::fft(&signal);
//!
//! // Check if a number is prime
//! let is_prime = miller_rabin::is_prime(17, 10);
//! ```

pub mod numerical;
pub mod number_theory;
pub mod optimization;
pub mod graph;

// Re-export commonly used types
pub use num_complex::Complex64;

/// Common error type for the library
#[derive(Debug, Clone)]
pub enum AlgorithmError {
    /// Input dimensions are incompatible
    DimensionMismatch(String),
    /// Numerical computation failed to converge
    ConvergenceFailure(String),
    /// Invalid input parameters
    InvalidInput(String),
    /// Algorithm encountered a numerical instability
    NumericalInstability(String),
}

impl std::fmt::Display for AlgorithmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlgorithmError::DimensionMismatch(msg) => write!(f, "Dimension mismatch: {}", msg),
            AlgorithmError::ConvergenceFailure(msg) => write!(f, "Convergence failure: {}", msg),
            AlgorithmError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AlgorithmError::NumericalInstability(msg) => write!(f, "Numerical instability: {}", msg),
        }
    }
}

impl std::error::Error for AlgorithmError {}

/// Result type for algorithm operations
pub type Result<T> = std::result::Result<T, AlgorithmError>;
