# achan [![crates.io](https://img.shields.io/crates/v/achan.svg)](https://crates.io/crates/achan) [![docs.rs](https://img.shields.io/docsrs/achan)](https://docs.rs/achan)

This crate provides a simple & convenient representation for any value:

```rust
enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>)
}
```

### `no_std`
This crate is compatible with `no_std` environments, requiring only the `alloc` crate. 

### `serde`
Support for serialising/deserialising using the [`serde`](https://github.com/serde-rs/serde) framework can be enabled via the `serde` feature.

### WASM Component
This crate defines and implements a [WIT API](https://github.com/WebAssembly/component-model) when the `wasm-component` feature is enabled, allowing it to be embedded in WASM applications.

---
