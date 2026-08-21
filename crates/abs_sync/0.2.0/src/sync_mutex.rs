use core::{
    ops::{DerefMut, Try},
    pin::Pin,
};

use crate::sync_tasks::TrSyncTask;

pub trait TrSyncMutex {
    type Target: ?Sized;

    fn acquire(&self) -> impl TrAcquire<'_, Self::Target>;
}

pub trait TrAcquire<'a, T>
where
    Self: 'a,
    T: 'a + ?Sized,
{
    type MutexGuard<'g>: TrMutexGuard<'a, 'g, T> where 'a: 'g;

    fn try_lock<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl Try<Output = Self::MutexGuard<'g>>
    where
        'a: 'g;

    fn lock<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl TrSyncTask<MayCancelOutput = Self::MutexGuard<'g>>
    where
        'a: 'g;
}

pub trait TrMutexGuard<'a, 'g, T>
where
    'a: 'g,
    Self: 'g + DerefMut<Target = T>,
    T: 'a + ?Sized,
{}
