# aad - Automatic Adjoint Differentiation Library

[![crates.io](https://img.shields.io/crates/v/aad.svg)](https://crates.io/crates/aad)
[![docs.rs](https://img.shields.io/docsrs/aad)](https://docs.rs/aad)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A pure Rust automatic differentiation library using reverse-mode adjoint differentiation.

## Features

- **Supports both f32 and f64**: Generic implementation works with any floating-point type implementing
  `num_traits::Float`.
- **Reverse-mode autodiff**: Efficiently compute gradients for scalar-valued functions with many inputs.
- **Operator overloading**: Use standard mathematical operators with variables.
- **High Performance**: Optimized for minimal runtime overhead.
    - Benchmarks show competitive performance, often outperforming alternatives in gradient computation (
      see [Benchmarks](#benchmarks)).
- **Type-agnostic functions**: Write generic mathematical code using the `FloatLike` trait.
- **Derive macros**: Automatically generate differentiable functions with `#[autodiff]` macro (requires `derive`
  feature).

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
aad = { version = "0.8.0", features = ["derive"] }
```

## Usage

### Basic Example

```rust
use aad::Tape;

fn main() {
    // Initialize computation tape
    let tape = Tape::default();

    // Create variables
    let [x, y] = tape.create_variables(&[2.0_f64, 3.0_f64]);

    // Build computation graph
    let z = (x + y) * x.sin();

    // Forward pass
    println!("z = {:.2}", z.value()); // z = 4.55

    // Reverse pass
    let gradients = z.compute_gradients();
    println!("Gradients: dx = {:.2}, dy = {:.2}",
             gradients.get_gradients(&[x, y]));
    // Gradients: dx = -1.17, dy = 0.91
}
```

### Using Macros for Automatic Differentiation

Enable the `derive` feature and use `#[autodiff]` to automatically differentiate functions:

```rust
use aad::{Tape, autodiff};

#[autodiff]
fn f(x: f64, y: f64) -> f64 {
    5.0 + 2.0 * x + y / 3.0
}

fn main() {
    let tape = Tape::default();
    let [x, y] = tape.create_variables(&[2.0_f64, 3.0_f64]);

    // Compute value and gradients
    let z = f(x, y);
    let gradients = z.compute_gradients();

    println!("Result: {:.2}", z.value());    // 5.0 + 4.0 + 1.0 = 10.00
    println!("Gradients: dx = {:.2}, dy = {:.2}",
             gradients.get_gradients(&[x, y]));
    // Gradients: dx = 2.00, dy = 0.33
}
```

## Benchmarks

Run benchmarks with:

```bash
cargo bench --features benchmarks
```

[Detailed results](https://nakashima-hikaru.github.io/aad/reports/)

## License

MIT License - see [LICENSE](LICENSE)
