use core::{
    ops::{Deref, DerefMut, Try},
    pin::Pin,
};
use crate::sync_tasks::TrSyncTask;

pub trait TrSyncRwLock {
    type Target: ?Sized;

    fn acquire(&self) -> impl TrAcquire<'_, Self::Target>;
}

pub trait TrAcquire<'a, T>
where
    Self: 'a,
    T: 'a + ?Sized,
{
    type ReaderGuard<'g>: TrReaderGuard<'a, 'g, T> where 'a: 'g;

    type WriterGuard<'g>: TrWriterGuard<'a, 'g, T> where 'a: 'g;

    type UpgradableGuard<'g>: TrUpgradableReaderGuard<'a, 'g, T> where 'a: 'g;

    fn try_read<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl Try<Output = Self::ReaderGuard<'g>>
    where
        'a: 'g;

    fn try_write<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl Try<Output = Self::WriterGuard<'g>>
    where
        'a: 'g;

    fn try_upgradable_read<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl Try<Output = Self::UpgradableGuard<'g>>
    where
        'a: 'g;

    fn read<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl TrSyncTask<Output = Self::ReaderGuard<'g>>
    where
        'a: 'g;

    fn write<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl TrSyncTask<Output = Self::WriterGuard<'g>>
    where
        'a: 'g;

    fn upgradable_read<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl TrSyncTask<Output = Self::UpgradableGuard<'g>>
    where
        'a: 'g;
}

pub trait TrReaderGuard<'a, 'g, T>
where
    'a: 'g,
    Self: 'g + Sized + Deref<Target = T>,
    T: 'a + ?Sized,
{
    type Acquire: TrAcquire<'a, T>;

    fn as_reader_guard(&self) -> &Self {
        self
    }
}

pub trait TrUpgradableReaderGuard<'a, 'g, T>
where
    'a: 'g,
    Self: 'g + TrReaderGuard<'a, 'g, T>,
    T: 'a + ?Sized,
{
    fn downgrade(
        self,
    ) -> <Self::Acquire as TrAcquire<'a, T>>::ReaderGuard<'g>;

    fn try_upgrade(
        self,
    ) -> Result<<Self::Acquire as TrAcquire<'a, T>>::WriterGuard<'g> , Self>;

    fn upgrade(
        self,
    ) -> impl TrUpgrade<'a, 'g, T, Acquire = Self::Acquire>;
}

pub trait TrWriterGuard<'a, 'g, T>
where
    'a: 'g,
    Self: 'g + TrReaderGuard<'a, 'g, T> + DerefMut<Target = T>,
    T: 'a + ?Sized,
{
    fn downgrade_to_reader(
        self,
    ) -> <Self::Acquire as TrAcquire<'a, T>>::ReaderGuard<'g>;

    fn downgrade_to_upgradable(
        self,
    ) -> <Self::Acquire as TrAcquire<'a, T>>::UpgradableGuard<'g>;
}

pub trait TrUpgrade<'a, 'g, T>
where
    'a: 'g,
    T: 'a + ?Sized,
{
    type Acquire: TrAcquire<'a, T>;

    fn try_upgrade<'u>(
        self: Pin<&'u mut Self>,
    ) -> impl Try<Output =
            <Self::Acquire as TrAcquire<'a, T>>::WriterGuard<'u>>
    where
        'g: 'u;

    fn upgrade<'u>(
        self: Pin<&'u mut Self>,
    ) -> impl TrSyncTask<Output =
            <Self::Acquire as TrAcquire<'a, T>>::WriterGuard<'u>>
    where
        'g: 'u;

    fn into_guard(
        self,
    ) -> <Self::Acquire as TrAcquire<'a, T>>::UpgradableGuard<'g>;
}

pub trait TrSyncMutex {
    type Target: ?Sized;

    type MutexGuard<'a>: TrMutexGuard<'a, Self::Target> where Self: 'a;

    fn is_acquired(&self) -> bool;

    fn try_acquire(&self) -> Option<Self::MutexGuard<'_>>;

    fn acquire(&self) -> impl TrSyncTask<Output = Self::MutexGuard<'_>>;
}

pub trait TrMutexGuard<'a, T>
where
    Self: Sized + DerefMut<Target = T>,
    T: 'a + ?Sized,
{
    type Mutex: 'a + ?Sized + TrSyncMutex<Target = T>;
}
