//! # The Addon Prelude
//! The addon library comes with a variety of tools. However, if you had to manually import every single thing that you used, it would be very verbose.
//! But importing a lot of things that a program never uses isn’t good either. A balance needs to be struck.
//! The prelude is the list of things that you should import. It’s kept as small as possible, and is focused on useful stuff.
//!
//! The prelude is meant to be imported using the wildcard `*` import, since it is kept light.
//! ```
//! use addon::prelude::*;
//! ```

pub use crate::ord::Bounded;

#[cfg(feature = "alloc")]
pub use crate::nev;
#[cfg(feature = "alloc")]
pub use crate::vec::{Nev, NonEmptyVec};
