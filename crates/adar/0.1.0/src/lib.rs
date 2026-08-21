#![doc = include_str!("../README.md")]

pub mod state_machine;
pub mod tuples;
pub use adar_macros as macros;
pub mod enums;

pub mod prelude {
    pub use crate::enums::*;
    pub use crate::macros::*;
    pub use crate::state_machine::*;
    pub use crate::tuples::*;
}
