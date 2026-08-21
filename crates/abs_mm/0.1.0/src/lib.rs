#![no_std]

// We always pull in `std` during tests, because it's just easier
// to write tests when you can assume you're on a capable platform
#[cfg(any(test, feature = "support-std"))]
extern crate std;

pub mod as_pinned;
pub mod mem_alloc;
pub mod res_man;

#[cfg(any(test, feature = "support-std"))]
mod std_global_;
