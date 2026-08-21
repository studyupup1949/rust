#![doc = include_str!("../README.md")]

mod adjustment;
mod clock;
mod simple;

pub use adjustment::{AdjustmentSettings, ConstantRate};
pub use clock::Clock;
pub use simple::AdjustingClock;

#[cfg(test)]
mod test;
