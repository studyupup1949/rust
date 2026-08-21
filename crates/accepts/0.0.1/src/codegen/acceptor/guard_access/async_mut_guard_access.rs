use core::{
    future::Future,
    ops::{Deref, DerefMut},
};

use super::AcquireError;

/// Provides mutable access to an inner value through a guard.
pub trait AsyncMutGuardAccess<Source, Inner> {
    type Guard<'a>: Deref<Target = Inner> + DerefMut
    where
        Self: 'a,
        Source: 'a;

    type Error;

    /// Attempts to acquire a mutable guard.
    ///
    /// On failure an [`AcquireError`] is returned. When the returned
    /// [`AcquireError::guard`] is [`Some`], the caller may continue processing
    /// with the provided guard. If it is [`None`], the guard was not acquired
    /// and the accompanying [`AcquireError::error`] should be handled.
    #[must_use]
    fn acquire<'a>(
        source: &'a Source,
    ) -> impl Future<Output = Result<Self::Guard<'a>, AcquireError<Self::Error, Self::Guard<'a>>>> + 'a
    where
        Self: 'a;
}
