# accepts

> ⚠️ **Pre-release**: version 0.0.1 is still experimental. APIs, features, and crate layout may change without backward compatibility guarantees.

`accepts` is a Rust toolkit for composing synchronous and asynchronous "acceptor" pipelines. Each acceptor receives a value and either performs side effects or forwards the value to the next acceptor, allowing you to represent complex flows as small, reusable building blocks. The library supports `no_std` targets and lets you opt into additional capabilities through feature flags.

Typical use cases include:

- Cleaning and validating event streams before they reach analytics storage.
- Routing asynchronous jobs into specialized executors without coupling producers to consumers.

The acceptor pattern shines in these situations because each adapter isolates a single responsibility, so you can swap, reorder, or reuse steps instead of rewriting hand-rolled control flow for every pipeline.

## Quickstart

### Add the dependency

```toml
[dependencies]
accepts = { version = "0.0.1", features = ["utils"] }
```

* Disable default features (`default-features = false`) to use the crate in `no_std` environments. Enable only the feature flags you need for utilities or extensions.

### Understand the `Accepts` trait

Every synchronous pipeline step implements the `Accepts` trait. Its full definition in the crate is intentionally tiny:

```rust
pub trait Accepts<Value> {
    fn accept(&self, value: Value);
}
```

The single `accept` method is all you need to implement to plug a custom stage into the chain. Async adapters follow the same idea through the `AsyncAccepts` counterpart, which returns a future instead of completing immediately.

`accept` intentionally returns `()` because each stage either forwards the value to the next acceptor or consumes it via side effects, so there is nothing useful to hand back and chaining stays simple.

### Build a minimal pipeline

```rust
use accepts::core_traits::Accepts;
use accepts::utils::core::acceptor::{CallbackAcceptor, FilterAcceptor, MapAcceptor};

fn main() {
    let sink = CallbackAcceptor::new(|value: i32| println!("final value: {value}"));
    let filter = FilterAcceptor::new(|value: &i32| value % 2 == 0, sink);
    let pipeline = MapAcceptor::new(|value: i32| value * 2, filter);

    for value in [1, 2, 3, 4] {
        pipeline.accept(value);
    }
}
```

This pipeline doubles each input, filters for even numbers, and prints the remaining values. The same pattern extends to asynchronous acceptors (`AsyncAccepts`) and additional adapters.

## Crate layout

- **`accepts`**: The primary crate. It exposes the core traits (`Accepts`, `AsyncAccepts`, `DynAsyncAccepts`) along with utility adapters and extension modules.
- **`accepts-macros`**: Houses procedural macros that generate acceptor implementations. These are re-exported through the `macros` feature of the main crate.
- **`accepts-codegen`**: Internal helper crate used by the procedural macros for code generation.
- **`accepts-test`**: Internal test-support crate used for validating the bundled adapters (not published).

## Key capabilities

### Core traits

- `Accepts<T>`: Abstraction for synchronous acceptors that process values immediately.
- `AsyncAccepts<T>`: Asynchronous acceptors that return a future which resolves when processing completes.
- `DynAsyncAccepts<T>`: Helper trait for working with asynchronous acceptors as trait objects in `alloc` environments.
- `NextAcceptors*` family: Utility traits that expose the "next" acceptor within adapters to cut down on boilerplate.

### Built-in acceptors

Enabling the `utils` feature unlocks a suite of ready-to-use adapters.

- **core** (no platform requirements): Mapping, inspecting, filtering, routing, repeat execution, one-shot forwarding, and more, with synchronous and asynchronous variants.
- **std** (requires the standard library): State-heavy adapters such as rate limiters, deduplicators, circuit breakers, and delegation helpers for containers like `Mutex`, `RwLock`, and `Vec`.

### Extension modules (`ext-*` features)

- `ext-log`: Integrates with the `log` crate to record accepted values.
- `ext-tracing`: Adds instrumentation hooks for `tracing` spans and events.
- `ext-serde` / `ext-serde_json`: Enables serialization-aware acceptors and JSON transformations.
- `ext-tokio`: Bridges asynchronous pipelines with Tokio components, such as `mpsc` senders and scheduling utilities.
- `ext-reqwest`: Builds request-sending pipelines on top of the `reqwest` HTTP client.

### Macros and code generation

Enable the `macros` feature to access procedural macros under `accepts::macros::codegen`.

- `auto_impl_async`: Derives `AsyncAccepts` implementations for your types.
- `auto_impl_dyn`: Generates `DynAsyncAccepts` implementations that return `Pin<Box<_>>` values (requires `alloc`).

The `NextAcceptors` derive macro and the `generate_linear_acceptor` helper macro are currently kept internal while we iterate on
their design. Both APIs are expected to change, and once the functionality stabilizes we may publish these macros (or successors
that fill the same role) as part of the public surface.

Behind the scenes, the macros rely on the `accepts-codegen` crate, including utilities such as `generate_linear_acceptor` for building pipeline structs safely and ergonomically.

## Feature flags

| Feature | Default | Description |
| --- | --- | --- |
| `std` | ✅ (default) | Enables adapters that rely on the standard library. Disable it for `no_std` builds. |
| `alloc` | ⛔ | Turns on heap-allocating utilities and makes `DynAsyncAccepts` available. |
| `utils` | ⛔ | Adds the collection of core and standard-library adapters. |
| `ext-*` | ⛔ | Opt-in integrations with crates like `log`, `tracing`, and `serde`. |
| `macros` | ⛔ | Re-exports the procedural macros for automatic impl generation. |
| `codegen` | ⛔ | Exposes the code generation helpers as part of the public API. |
| `all` | ⛔ | Convenience flag that enables `std`, `utils`, `ext-*`, `macros`, and `codegen`. |

## License

This project is dual-licensed under MIT and Apache-2.0. You may use it under either license.


## Contributing

Bug reports, feature requests, and pull requests are always welcome.

> I'm still new to GitHub and not very confident in English.
> For now, I'm doing my best with the help of ChatGPT,
> so I might misunderstand something. Thanks for your understanding!
