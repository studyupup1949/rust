//! Special functions

mod digamma;
mod gamma;
mod iwasawa;

#[allow(dead_code, unused_imports)]
pub (crate) use gamma::naive_gamma;

pub use digamma::adic_digamma;
pub use iwasawa::iwasawa_log;
