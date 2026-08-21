use core::{
    ops::{DerefMut, Try},
    pin::Pin,
};

use crate::cancellation::TrMayCancel;

/// Mutex for asynchronous task pattern.
pub trait TrAsyncMutex {
    type Target: ?Sized;

    fn acquire(&self) -> impl TrAcquire<'_, Self::Target>;
}

pub trait TrAcquire<'a, T>
where
    Self: 'a,
    T: 'a + ?Sized,
{
    type Guard<'g>: TrMutexGuard<'a, 'g, T> where 'a: 'g;

    fn try_lock<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl Try<Output = Self::Guard<'g>>
    where
        'a: 'g;

    fn lock_async<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl TrMayCancel<'g, 
        MayCancelOutput: Try<Output = Self::Guard<'g>>>
    where
        'a: 'g;
}

pub trait TrMutexGuard<'a, 'g, T>
where
    'a: 'g,
    Self: 'g + Sized + DerefMut<Target = T>,
    T: 'a + ?Sized,
{}
