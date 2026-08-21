#![no_std]

use cfg_if::cfg_if;

cfg_if! (
    if #[cfg(feature = "full")] {
        extern crate alloc;
    }
);

pub mod vectors;
pub mod matricies;
pub mod prelude;