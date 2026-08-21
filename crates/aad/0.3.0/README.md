# AAD

[![CI](https://github.com/nakashima-hikaru/aad/actions/workflows/ci.yml/badge.svg)](https://github.com/nakashima-hikaru/aad/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/aad.svg)](https://crates.io/crates/aad)
[![Docs.rs](https://img.shields.io/docsrs/aad)](https://docs.rs/aad)

This crate provides tools for implementing adjoint(a.k.a. reverse-mode) automatic differentiation in Rust. It
enables gradient computation for scalar values through a flexible and extensible API.

- **User-Friendly Design**: Equations can be manipulated as seamlessly as primitive floating-point types.
    - This design draws heavy inspiration from the [`rustograd`](https://github.com/msakuta/rustograd) and [
      `RustQuant_autodiff`](https://github.com/avhz/RustQuant/tree/main/crates/RustQuant_autodiff) library.
- **High Performance**: The library is designed to be both efficient and scalable, with minimal overhead.
    - Benchmarks show it is as fast or faster compared to [
      `RustQuant_autodiff`](https://github.com/avhz/RustQuant/tree/main/crates/RustQuant_autodiff).
    - Note: [
      `RustQuant_autodiff`](https://github.com/avhz/RustQuant/tree/main/crates/RustQuant_autodiff) includes extra dependencies, which may require additional system setup when installing
      on Linux.
- **No Dependencies**: The library is self-contained and does not rely on any external dependencies.

## Quick Start

Here's an example of how to use the library:

```rust
use aad::Tape;

fn main() {
    let tape = Tape::default();

    let x = tape.crate_variable(2.0);
    let y = tape.crate_variable(3.0);

    let z = (x + y) * x.sin();

    println!("{}", z.value());

    let grads = z.compute_gradients();
    println!("Gradients are: {:?}", grads.get_gradients(&[x, y]));
}
```

## Benchmarks

Benchmark results can be viewed [here](https://nakashima-hikaru.github.io/aad/reports/).

## License

This project is licensed under the [MIT License](LICENSE).

