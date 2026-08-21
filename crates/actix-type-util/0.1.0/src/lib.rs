//! # Actix type util
//! Crate with some useful types for working with [actix-web](https://crates.io/crates/actix-web)
//!
//! ## License
//!
//! MIT or Apache-2.0, at your option

mod empty;
mod redirect;
mod set_cookie;

pub use empty::Empty;
pub use redirect::Redirect;
pub use set_cookie::SetCookie;

