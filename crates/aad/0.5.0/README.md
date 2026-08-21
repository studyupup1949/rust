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
- **Type-agnostic functions**: Write generic mathematical code using the `ScalarLike` trait.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
aad = "0.5.0"
```

For benchmarks:

```toml
[features]
benchmarks = ["aad/benchmarks"]
```

## Usage

### Basic Example

```rust
use aad::{Tape, ScalarLike};

fn main() {
    // Initialize computation tape
    let tape = Tape::default();

    // Create variables
    let x = tape.create_variable(2.0);
    let y = tape.create_variable(3.0);

    // Build computation graph
    let z = (x + y) * x.sin();

    // Forward pass
    println!("z = {:.2}", z.value()); // z = 4.55

    // Reverse pass
    let gradients = z.compute_gradients();
    println!("Gradients: dx = {:.2}, dy = {:.2}",
             gradients.get_gradient(&x),
             gradients.get_gradient(&y));
    // Gradients: dx = -1.17, dy = 0.91
}
```

### Generic Mathematical Functions

```rust
use aad::{ScalarLike, Tape};

fn f<T, S: ScalarLike<T>>(x: S, y: S) -> S {
    (x + y) * x.sin()
}

fn main() {
    // Works with f32, f64, Variable<f32> and Variable<f64>:
    let tape = Tape::default();
    let x = tape.create_variable(2.0_f32);
    let y = tape.create_variable(3.0_f32);
    let z = f(x, y);
}
```

## Benchmarks

Run benchmarks with:

```bash
cargo bench --features benchmarks
```

## License

MIT License - see [LICENSE](LICENSE) file
