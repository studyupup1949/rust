# Advanced Algorithms Library

A comprehensive Rust library providing high-performance implementations of advanced algorithms across multiple domains: numerical analysis, linear algebra, number theory, cryptography, optimization, statistics, and graph theory.

[![Crates.io](https://img.shields.io/crates/v/advanced-algorithms.svg)](https://crates.io/crates/advanced-algorithms)
[![Documentation](https://docs.rs/advanced-algorithms/badge.svg)](https://docs.rs/advanced-algorithms)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## 🚀 Features

- **High Performance**: Optimized implementations with multi-threading support via Rayon
- **Numerically Stable**: Algorithms prioritize numerical stability and accuracy
- **Well Documented**: Comprehensive documentation with examples for each algorithm
- **Type Safe**: Leverages Rust's type system for correctness and safety
- **Production Ready**: Thoroughly tested with extensive test suites

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
advanced-algorithms = "0.1.0"
```

## 🧮 Algorithms Included

### Numerical Analysis & Linear Algebra

#### Fast Fourier Transform (FFT)

The "most important numerical algorithm of our lifetime." Essential for signal processing, audio compression, and image processing.

```rust
use advanced_algorithms::numerical::fft;

let signal = vec![1.0, 2.0, 3.0, 4.0, 3.0, 2.0, 1.0, 0.0];
let spectrum = fft::fft(&signal);

// For large datasets, use parallel version
let large_signal: Vec<f64> = (0..8192).map(|i| (i as f64).sin()).collect();
let spectrum_parallel = fft::fft_parallel(&large_signal);

// Compute power spectrum
let power = fft::power_spectrum(&spectrum);
```

#### QR Decomposition

Robust method for solving linear least squares problems using Householder transformations. More numerically stable than Gaussian elimination.

```rust
use advanced_algorithms::numerical::qr_decomposition::QRDecomposition;

let matrix = vec![
    vec![12.0, -51.0, 4.0],
    vec![6.0, 167.0, -68.0],
    vec![-4.0, 24.0, -41.0],
];

let qr = QRDecomposition::decompose(&matrix).unwrap();
let q = qr.q();  // Orthogonal matrix
let r = qr.r();  // Upper triangular matrix

// Solve linear system Ax = b
let b = vec![1.0, 2.0, 3.0];
let x = qr.solve(&b).unwrap();
```

#### Newton-Raphson Method

Iterative root-finding algorithm with quadratic convergence. Used in calculators and physics engines.

```rust
use advanced_algorithms::numerical::newton_raphson;

// Find square root of 2 by solving x² - 2 = 0
let f = |x: f64| x * x - 2.0;
let df = |x: f64| 2.0 * x;

let root = newton_raphson::find_root(f, df, 1.0, 1e-10, 100).unwrap();
assert!((root - 2.0_f64.sqrt()).abs() < 1e-10);

// Advanced solver with history
use advanced_algorithms::numerical::newton_raphson::{NewtonRaphsonSolver, NewtonRaphsonConfig};

let solver = NewtonRaphsonSolver::new(f, df)
    .with_config(NewtonRaphsonConfig {
        tolerance: 1e-12,
        max_iterations: 50,
        verbose: true,
    });

let result = solver.solve(1.0).unwrap();
println!("Root: {}, iterations: {}", result.root, result.iterations);
```

#### Singular Value Decomposition (SVD)

Matrix factorization for dimensionality reduction, PCA, and pseudoinverse computation.

```rust
use advanced_algorithms::numerical::svd::SVD;

let matrix = vec![
    vec![1.0, 2.0],
    vec![3.0, 4.0],
    vec![5.0, 6.0],
];

let svd = SVD::decompose(&matrix, 1e-10, 1000).unwrap();
let singular_values = svd.singular_values();
let rank = svd.rank(1e-8);
let condition_number = svd.condition_number();
```

### Number Theory & Cryptography

#### Miller-Rabin Primality Test

Probabilistic primality testing for RSA key generation and cryptographic applications.

```rust
use advanced_algorithms::number_theory::miller_rabin;

// Probabilistic test
assert!(miller_rabin::is_prime(1_000_000_007, 20));

// Deterministic test for 64-bit integers
assert!(miller_rabin::is_prime_deterministic(982_451_653));

// Generate random prime
let prime = miller_rabin::generate_prime(32, 20);
```

#### Extended Euclidean Algorithm

Computes GCD and modular multiplicative inverses for cryptography.

```rust
use advanced_algorithms::number_theory::extended_euclidean;

// Find GCD and Bézout coefficients
let result = extended_euclidean::extended_gcd(240, 46);
assert_eq!(result.gcd, 2);
assert_eq!(240 * result.x + 46 * result.y, 2);

// Modular inverse for RSA
let inv = extended_euclidean::mod_inverse(3, 11).unwrap();
assert_eq!((3 * inv) % 11, 1);

// Solve linear congruence: 3x ≡ 6 (mod 9)
let solutions = extended_euclidean::solve_linear_congruence(3, 6, 9);
```

#### Mersenne Twister (MT19937)

High-quality pseudo-random number generator with period 2^19937 - 1.

```rust
use advanced_algorithms::number_theory::mersenne_twister::MersenneTwister;

let mut rng = MersenneTwister::new(12345);

let random_int = rng.next_u32();
let random_float = rng.next_f64();  // [0, 1)
let random_range = rng.next_range(100);  // [0, 100)
let gaussian = rng.next_gaussian();  // N(0, 1)
```

### Optimization & Statistics

#### Gradient Descent

Fundamental optimizer for machine learning and neural network training.

```rust
use advanced_algorithms::optimization::gradient_descent::{GradientDescent, LearningRate};

// Minimize Rosenbrock function
let f = |x: &[f64]| {
    let a = 1.0 - x[0];
    let b = x[1] - x[0] * x[0];
    a * a + 100.0 * b * b
};

let grad_f = |x: &[f64]| vec![
    -2.0 * (1.0 - x[0]) - 400.0 * x[0] * (x[1] - x[0] * x[0]),
    200.0 * (x[1] - x[0] * x[0]),
];

let gd = GradientDescent::new()
    .with_learning_rate(LearningRate::Adaptive {
        initial: 0.01,
        epsilon: 1e-8,
    })
    .with_momentum(0.9)
    .with_max_iterations(10000);

let result = gd.minimize(f, grad_f, &[0.0, 0.0]).unwrap();
```

#### Levenberg-Marquardt Algorithm

Specialized optimizer for curve fitting and non-linear least squares.

```rust
use advanced_algorithms::optimization::levenberg_marquardt::LevenbergMarquardt;

// Fit exponential: y = a*exp(b*x)
let x_data = vec![0.0, 1.0, 2.0, 3.0, 4.0];
let y_data = vec![1.0, 2.7, 7.4, 20.1, 54.6];

let residual = |params: &[f64], i: usize| {
    let prediction = params[0] * (params[1] * x_data[i]).exp();
    y_data[i] - prediction
};

let lm = LevenbergMarquardt::new()
    .with_max_iterations(200)
    .with_tolerance(1e-8);

let result = lm.fit(residual, &[1.0, 1.0], x_data.len()).unwrap();
println!("Fitted parameters: {:?}", result.parameters);
```

#### Monte Carlo Integration

Numerical integration using random sampling with parallel processing.

```rust
use advanced_algorithms::optimization::monte_carlo;

// Integrate x² from 0 to 1 (should be 1/3)
let f = |x: &[f64]| x[0] * x[0];
let bounds = vec![(0.0, 1.0)];

let result = monte_carlo::integrate(f, &bounds, 100000).unwrap();
println!("Integral: {} ± {}", result.value, result.error);

// Multidimensional integration
let f_2d = |x: &[f64]| x[0] * x[1];
let bounds_2d = vec![(0.0, 1.0), (0.0, 1.0)];
let result_2d = monte_carlo::integrate(f_2d, &bounds_2d, 100000).unwrap();
```

### Graph Algorithms

#### Dijkstra's Algorithm

Shortest path for graphs with non-negative weights.

```rust
use advanced_algorithms::graph::{Graph, dijkstra};

let mut graph = Graph::new(5);
graph.add_edge(0, 1, 4.0);
graph.add_edge(0, 2, 1.0);
graph.add_edge(2, 1, 2.0);
graph.add_edge(1, 3, 1.0);
graph.add_edge(2, 3, 5.0);

let result = dijkstra::shortest_path(&graph, 0);
let path = result.path_to(3).unwrap();
println!("Shortest path: {:?}, distance: {}", path, result.distance[3]);
```

#### A\* Search Algorithm

Heuristic-based pathfinding for optimal paths.

```rust
use advanced_algorithms::graph::{Graph, astar};

let mut graph = Graph::new(10);
// ... add edges ...

// Manhattan distance heuristic for grid
let heuristic = |node: usize| {
    // Estimate remaining distance to goal
    (goal_x - node_x).abs() + (goal_y - node_y).abs()
};

let result = astar::find_path(&graph, start, goal, heuristic).unwrap();
println!("Path: {:?}, cost: {}", result.1, result.0);
```

#### Bellman-Ford Algorithm

Shortest paths with negative edge weights and cycle detection.

```rust
use advanced_algorithms::graph::{Graph, bellman_ford};

let mut graph = Graph::new(4);
graph.add_edge(0, 1, 1.0);
graph.add_edge(1, 2, -3.0);  // Negative weight OK
graph.add_edge(2, 3, 1.0);

let result = bellman_ford::shortest_path(&graph, 0).unwrap();

// Detect negative cycles
if bellman_ford::has_negative_cycle(&graph) {
    let cycle = bellman_ford::find_negative_cycle(&graph).unwrap();
    println!("Negative cycle found: {:?}", cycle);
}
```

#### Floyd-Warshall Algorithm

All-pairs shortest paths.

```rust
use advanced_algorithms::graph::{Graph, floyd_warshall};

let mut graph = Graph::new(4);
// ... add edges ...

let result = floyd_warshall::all_pairs_shortest_path(&graph).unwrap();

// Get distance between any two nodes
let dist = result.distance(0, 3);
let path = result.path(0, 3).unwrap();

// Find graph properties
let diameter = floyd_warshall::diameter(&graph);
let centers = floyd_warshall::find_center(&graph);
```

## 🔧 Performance Tips

### Multi-threading

Many algorithms support parallel processing:

```rust
// FFT automatically uses parallel processing for large inputs
let spectrum = fft::fft_parallel(&large_signal);

// Monte Carlo integration parallelizes automatically
let result = monte_carlo::integrate(f, &bounds, 1_000_000).unwrap();
```

### Optimization Levels

For maximum performance, compile with optimizations:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

## 📚 Examples

See the `examples/` directory for complete working examples:

```bash
cargo run --example fft_demo
cargo run --example graph_pathfinding
cargo run --example curve_fitting
cargo run --example prime_generation
```

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Run benchmarks:

```bash
cargo bench
```

## 📖 Documentation

Full API documentation is available at [docs.rs](https://docs.rs/advanced-algorithms).

Generate local documentation:

```bash
cargo doc --open
```

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

This project is dual-licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

You may choose either license for your use.

## 🙏 Acknowledgments

This library implements classic algorithms from computer science and numerical analysis. Special thanks to:

- Donald Knuth for _The Art of Computer Programming_
- William H. Press et al. for _Numerical Recipes_
- Cormen, Leiserson, Rivest, and Stein for _Introduction to Algorithms_

## 📞 Support

- 📧 Email: adarsh.dubey64@gmail.com
- 🐛 Issues: [GitHub Issues](https://github.com/Casperrules/advanced-algorithms/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/Casperrules/advanced-algorithms/discussions)

## 🗺️ Roadmap

- [ ] Additional optimization algorithms (L-BFGS, Conjugate Gradient)
- [ ] More linear algebra operations (Cholesky decomposition, LU decomposition)
- [ ] Statistical distributions and hypothesis tests
- [ ] Parallel graph algorithms
- [ ] GPU acceleration support
