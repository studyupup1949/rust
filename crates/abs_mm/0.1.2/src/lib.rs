#![no_std]

// We always pull in `std` during tests, because it's just easier
// to write tests when you can assume you're on a capable platform
#[cfg(any(test))]
extern crate std;

#[cfg(any(test, feature = "core_alloc"))]
mod core_alloc_;

pub mod as_pinned;

pub mod mem_alloc;
pub mod res_man;
