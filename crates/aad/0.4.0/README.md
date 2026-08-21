# AAD: Adjoint Automatic Differentiation

[![CI](https://github.com/nakashima-hikaru/aad/actions/workflows/ci.yml/badge.svg)](https://github.com/nakashima-hikaru/aad/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/aad.svg)](https://crates.io/crates/aad)
[![Docs.rs](https://docs.rs/aad/badge.svg)](https://docs.rs/aad/latest/aad/)

A Rust library for reverse-mode automatic differentiation (AD), enabling efficient gradient computation for
scalar-valued functions.

## Features

- **Intuitive API**: Write equations as naturally as primitive `f64` operations.
    - Operator overloading (`+`, `*`, `sin`, `ln`, etc.) for seamless expression building.
    - Inspired by [`rustograd`](https://github.com/msakuta/rustograd) and [
      `RustQuant_autodiff`](https://github.com/avhz/RustQuant/tree/main/crates/RustQuant_autodiff).
- **High Performance**: Optimized for minimal runtime overhead.
    - Benchmarks show competitive performance, often outperforming alternatives in gradient computation (
      see [Benchmarks](#benchmarks)).
- **Zero Dependencies**: Core library has no external dependencies.
    - [`RustQuant_autodiff`](https://github.com/avhz/RustQuant/tree/main/crates/RustQuant_autodiff) includes extra
      dependencies, which may require additional system setup when installing on Linux.
    - (Optional `criterion` and `RustQuant_autodiff` for benchmarking only.)
- **Extensive Math Support**:
    - Trigonometric (`sin`, `cos`, `tanh`), exponential (`exp`, `powf`), logarithmic (`ln`, `log10`), and more.
    - Full list in [supported operations](#supported-operations).

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
aad = "0.3.0"
```

## Quick Start

```rust
use aad::Tape;
use aad::ScalarLike;

fn main() {
    // Initialize a computation tape
    let tape = Tape::default();

    // Create variables
    let x = tape.create_variable(2.0);
    let y = tape.create_variable(3.0);

    // Create type-agnostic mathematical functions that work with both Variable and f64:
    fn f<S: ScalarLike>(x: S, y: S) -> S {
        (x + y) * x.sin()
    }
    
    let z = f(x, y);

    // Forward pass: compute value
    println!("z = {:.2}", z.value()); // Output: z = 4.55

    // Reverse pass: compute gradients
    let grads = z.compute_gradients();
    println!("Gradients: dx = {:.2}, dy = {:.2}",
             grads.get_gradient(&x),
             grads.get_gradient(&y)
    ); // Output: dx = -2.83, dy = 0.14
}
```

## Supported Operations

### Basic Arithmetic

- `+`, `-`, `*`, `/`, negation
- Assignment operators (`+=`, `*=`, etc.)

### Mathematical Functions

- **Exponential**: `exp`, `powf`, `sqrt`, `hypot`
- **Logarithmic**: `ln`, `log`, `log2`, `log10`
- **Trigonometric**: `sin`, `cos`, `tan`, `asin`, `acos`, `sinh`, `cosh`
- **Other**: `abs`, `recip`, `cbrt`

See [`math.rs`](./src/overload/math.rs) for full details.

## Benchmarks

[Detailed results](https://nakashima-hikaru.github.io/aad/reports/)

## Design Notes

- **Tape-based**: All operations are recorded to a `Tape` for efficient reverse-mode traversal.
- **Lightweight**: Variables are `Copy`-enabled structs with minimal memory footprint.

## Contributing

Contributions are welcome! Open an issue or PR for feature requests, bug fixes, or documentation improvements.

## License

MIT License. See [LICENSE](LICENSE) for details.
