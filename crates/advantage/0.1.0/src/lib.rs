#![cfg_attr(feature = "ffi", feature(try_trait))]

extern crate nalgebra;
extern crate num;
#[doc(hidden)]
pub extern crate paste;
extern crate rayon;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate static_assertions;

pub use nalgebra::{DMatrix, DVector};
pub use num::traits::Float;

pub mod drivers;

mod macros;
pub use macros::*;

mod acontext;
pub use acontext::*;

mod afloat;
pub use afloat::*;

#[cfg(feature = "ffi")]
pub mod ffi;

mod function;
pub use function::*;

mod operation;
pub use operation::*;

mod scalar;
pub use scalar::*;

mod tape;
pub use tape::*;

/// Default imports that all projects using this crate should have in scope
pub mod prelude {
    pub use super::Function as _;
    pub use super::Tape as _;

    pub use super::adv_fn;
    pub use super::adv_fn_obj;
    pub use super::adv_struct;
}
