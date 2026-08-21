mod db;
pub mod display;
mod parts;
mod result;

pub use db::AocDatabase;
pub use parts::Parts;
pub use result::{AocResult, AocYear};

pub(crate) use std::fmt::Display;
