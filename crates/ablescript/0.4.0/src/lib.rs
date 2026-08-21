//! The AbleScript language reference implementation. See
//! <https://git.ablecorp.us/AbleScript/able-script> for more
//! information.

#![forbid(unsafe_code, clippy::unwrap_used)]

pub mod ast;
pub mod error;
pub mod interpret;
pub mod parser;
pub mod variables;

mod base_55;
mod brian;
mod consts;
mod lexer;
