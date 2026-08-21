//! Sealed trait pattern to prevent external implementations.

/// A sealed trait that cannot be implemented outside of this crate.
pub trait Sealed {}

// Implement for internal types only
impl<T: ?Sized> Sealed for &T {}
impl<T: ?Sized> Sealed for &mut T {}