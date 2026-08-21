//! A non-empty growable array type.
//!
//! The standard [`Vec`] can be empty. When your invariant requires at least one
//! element, you end up sprinkling `unwrap()` and `expect()` across call sites to
//! silence the compiler. [`NonEmptyVec`] encodes that guarantee in the type itself —
//! `first()` and `last()` return `&T`, not `Option<&T>`, and there is no `is_empty`.
//!
//! The [`nev!`](crate::nev) macro constructs instances with the same syntax as standard `vec!`.
//!
//! [`Vec`]: alloc::vec::Vec

mod nev;
pub use nev::{Nev, NonEmptyVec};

/// Creates a [`NonEmptyVec`] containing the arguments.
///
/// `nev!` allows `NonEmptyVec`s to be defined with the same syntax as array expressions.
/// There are two forms of this macro:
/// * Create a [`NonEmptyVec`] containing a given list of elements:
/// ```
/// use addon::nev;
/// let v = nev![1, 2, 3];
/// assert_eq!(v[0], 1);
/// assert_eq!(v[1], 2);
/// assert_eq!(v[2], 3);
/// ```
/// * Create a Vec from a given element and size:
/// ```
/// use addon::nev;
/// let v = nev![1; 3];
/// assert_eq!(v, [1, 1, 1]);
/// ```
///
/// `nev![]` cannot be called empty, because a `NonEmptyVec` cannot be empty.
///
/// Note that unlike array expressions this syntax supports all elements which implement `Clone` and the number of elements doesn’t have to be a constant.
/// This will use clone to duplicate an expression, so one should be careful using this with types having a nonstandard Clone implementation
#[macro_export]
macro_rules! nev {
    ($elem:expr) => {
        $crate::vec::NonEmptyVec::new($elem)
    };

    ($elem:expr; $n:expr) => {
        {
            let val = $elem;
            let mut v = $crate::vec::NonEmptyVec::new(val.clone());
            for _ in 1..$n {
                v.push(val.clone());
            }
            v
        }
    };

    ($first:expr, $($rest:expr),+ $(,)?) => {
        {
            let mut v = $crate::vec::NonEmptyVec::new($first);
            $(
                v.push($rest);
            )*
            v
        }
    };
}
