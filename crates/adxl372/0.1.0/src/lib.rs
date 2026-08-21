#![no_std]

mod error;

pub mod config;
pub mod device;
pub mod fifo;
pub mod interface;
mod log;
pub mod params;
pub mod registers;
pub mod self_test;

pub use crate::device::Adxl372;
pub use crate::error::{Error, Result};
