# AAD

This crate provides tools for implementing adjoint(a.k.a. reverse-mode) automatic differentiation in Rust. It
enables gradient computation for scalar values through a flexible and extensible API.

- **User-Friendly Design**: Equations can be manipulated as seamlessly as primitive floating-point types.
  - This design draws heavy inspiration from the [`rustograd`](https://github.com/msakuta/rustograd) and [
    `RustQuant_autodiff`](https://github.com/avhz/RustQuant/tree/main/crates/RustQuant_autodiff) library.
- **High Performance**: The library is designed to be both efficient and scalable, with minimal overhead.
  - Benchmarks show it is as fast or faster compared to `RustQuant_autodiff`.

## Quick Start

Here's an example of how to use the library:

```rust
use aad::Tape;

fn main() {
    let tape = Tape::default();

    let x = tape.var(2.0);
    let y = tape.var(3.0);

    let z = (x + y) * x.sin();

    println!("{}", z.value());

    let grads = z.backward();
    println!("Gradients are: {:?}", grads.get(&[x, y]));
}
```

## License

This project is licensed under the [MIT License](LICENSE).

