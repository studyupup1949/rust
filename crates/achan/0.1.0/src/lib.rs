#![no_std]

extern crate alloc;

mod value;

#[cfg(feature = "wasm-component")]
mod bindings;
#[cfg(feature = "wasm-component")]
mod component;

pub use value::Value;
