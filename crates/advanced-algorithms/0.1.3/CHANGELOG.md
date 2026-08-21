# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2025-12-31

### Changed
- Updated README documentation

## [0.1.2] - 2025-12-31

### Changed
- Updated repository and homepage URLs
- Minor documentation improvements

## [0.1.1] - 2025-12-31

### Changed

- Excluded PUBLISHING.md and PROJECT_SUMMARY.md from published package
- All clippy warnings fixed for production-ready code

## [0.1.0] - 2025-12-31

### Added

#### Numerical Analysis & Linear Algebra

- **Fast Fourier Transform (FFT)**: Cooley-Tukey algorithm with parallel processing support

  - `fft()`: Standard FFT for real-valued input
  - `fft_complex()`: FFT for complex-valued input
  - `ifft()`: Inverse FFT
  - `fft_parallel()`: Multi-threaded FFT for large datasets
  - `power_spectrum()`: Compute power spectrum from FFT output
  - `magnitude_spectrum()`: Compute magnitude spectrum

- **QR Decomposition**: Householder transformations for numerically stable decomposition

  - `QRDecomposition::decompose()`: Compute Q and R matrices
  - `solve()`: Solve linear systems using QR
  - Support for overdetermined systems

- **Newton-Raphson Method**: Iterative root finding

  - `find_root()`: Basic root finding
  - `NewtonRaphsonSolver`: Advanced solver with configuration
  - `find_roots_in_interval()`: Find multiple roots
  - `numerical_gradient()`: Numerical gradient computation

- **Singular Value Decomposition (SVD)**: Matrix factorization
  - `SVD::decompose()`: Compute U, Σ, V^T
  - `rank()`: Compute matrix rank
  - `condition_number()`: Compute condition number

#### Number Theory & Cryptography

- **Miller-Rabin Primality Test**: Probabilistic and deterministic primality testing

  - `is_prime()`: Probabilistic test
  - `is_prime_deterministic()`: Deterministic for 64-bit integers
  - `generate_prime()`: Generate random primes

- **Extended Euclidean Algorithm**: GCD and modular arithmetic

  - `extended_gcd()`: Compute GCD and Bézout coefficients
  - `mod_inverse()`: Modular multiplicative inverse
  - `solve_linear_congruence()`: Solve ax ≡ b (mod m)
  - `gcd_multiple()`: GCD of multiple numbers
  - `lcm()`, `lcm_multiple()`: Least common multiple

- **Mersenne Twister (MT19937)**: High-quality PRNG
  - `MersenneTwister::new()`: Initialize with seed
  - `next_u32()`, `next_u64()`: Generate random integers
  - `next_f64()`: Generate random floats
  - `next_gaussian()`: Box-Muller normal distribution

#### Optimization & Statistics

- **Gradient Descent**: First-order optimization

  - Multiple learning rate strategies (constant, decreasing, exponential, adaptive)
  - Momentum support
  - `StochasticGD`: Mini-batch gradient descent

- **Levenberg-Marquardt Algorithm**: Non-linear least squares

  - Curve fitting
  - Parameter estimation
  - Automatic Jacobian computation

- **Monte Carlo Integration**: Numerical integration via sampling
  - Parallel processing for large sample sizes
  - Multi-dimensional integration
  - Importance sampling support

#### Graph Algorithms

- **Dijkstra's Algorithm**: Single-source shortest paths

  - `shortest_path()`: Compute all shortest paths from source
  - `shortest_path_to_target()`: Early termination for specific target
  - Path reconstruction

- **A\* Search Algorithm**: Heuristic pathfinding

  - `find_path()`: A\* with custom heuristic
  - `find_path_bounded()`: A\* with cost limit
  - Grid utilities with Manhattan and Euclidean distance

- **Bellman-Ford Algorithm**: Shortest paths with negative weights

  - `shortest_path()`: Single-source shortest paths
  - `has_negative_cycle()`: Cycle detection
  - `find_negative_cycle()`: Locate negative cycles

- **Floyd-Warshall Algorithm**: All-pairs shortest paths
  - `all_pairs_shortest_path()`: Compute all distances
  - `transitive_closure()`: Graph reachability
  - `diameter()`: Find graph diameter
  - `find_center()`: Find graph center

#### Infrastructure

- Comprehensive error handling with `AlgorithmError` enum
- Extensive test coverage for all algorithms
- Detailed documentation with examples
- Multi-threading support via Rayon
- Type-safe APIs leveraging Rust's type system

### Documentation

- Complete README.md with usage examples
- PUBLISHING.md guide for crates.io publication
- Inline documentation for all public APIs
- Examples in doc comments

### Performance

- Optimized release profile configuration
- Parallel implementations for computationally intensive algorithms
- Numerically stable implementations

[Unreleased]: https://github.com/yourusername/advanced-algorithms/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/advanced-algorithms/releases/tag/v0.1.0
