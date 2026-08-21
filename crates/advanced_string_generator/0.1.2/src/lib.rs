mod regex_generator;
pub use regex_generator::RegexGenerator;

#[cfg(feature = "wasm")]
mod wasm;
#[cfg(feature = "wasm")]
pub use wasm::*;
