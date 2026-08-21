use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "no_std")]  {
        pub use crate::vectors::{VectorSlice, MutVectorSlice};
    }
}

cfg_if! {
    if #[cfg(feature = "full")] {
        pub use crate::vectors::{Vector, MutVector};
    }
}