#![doc = include_str!("../README.md")]

/// Project specific errors
pub mod errors;
/// Parsing functionality
pub mod parser;

/// A collection of common requirements
pub mod prelude {
    #[doc(hidden)]
    pub use crate::parser::{parse_acf, Acf};

    #[doc(hidden)]
    pub use crate::errors::AcfError;
}
