#![no_std]

#[cfg(test)]
extern crate alloc;

pub use cipher;

mod nh;

mod construction;
pub use self::construction::Cipher;

#[cfg(test)]
mod tests;
