#[cfg(feature = "__internal_macros_flag")]
pub(crate) mod internal;

#[cfg(feature = "macros")]
mod public;
#[cfg(feature = "macros")]
pub use public::*;
