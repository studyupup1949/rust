use core::{
    ops::{Deref, DerefMut, Try},
    pin::Pin,
};

use crate::cancellation::TrIntoFutureMayCancel;

/// Reader-Writer lock for asynchronous task pattern.
pub trait TrAsyncRwLock {
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

    fn read_async<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl TrIntoFutureMayCancel<'g,
            MayCancelOutput: Try<Output = Self::ReaderGuard<'g>>>
    where
        'a: 'g;

    fn write_async<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl TrIntoFutureMayCancel<'g,
            MayCancelOutput: Try<Output = Self::WriterGuard<'g>>>
    where
        'a: 'g;

    fn upgradable_read_async<'g>(
        self: Pin<&'g mut Self>,
    ) -> impl TrIntoFutureMayCancel<'g,
            MayCancelOutput: Try<Output = Self::UpgradableGuard<'g>>>
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
    fn downgrade(self) -> <Self::Acquire as TrAcquire<'a, T>>::ReaderGuard<'g>;

    fn upgrade(self) -> impl TrUpgrade<'a, 'g, T, Acquire = Self::Acquire>;
}

pub trait TrWriterGuard<'a, 'g, T>
where
    'a: 'g,
    Self: 'g + TrReaderGuard<'a, 'g, T> + DerefMut<Target = T>,
    T: 'a + ?Sized,
{
    fn downgrade(self) -> <Self::Acquire as TrAcquire<'a, T>>::ReaderGuard<'g>;

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
    ) -> impl Try<Output = <Self::Acquire as TrAcquire<'a, T>>::WriterGuard<'u>>
    where
        'g: 'u;

    fn upgrade_async<'u>(
        self: Pin<&'u mut Self>,
    ) -> impl TrIntoFutureMayCancel<'u, MayCancelOutput: Try<Output =
            <Self::Acquire as TrAcquire<'a, T>>::WriterGuard<'u>>>
    where
        'g: 'u;

    fn into_guard(
        self,
    ) -> <Self::Acquire as TrAcquire<'a, T>>::UpgradableGuard<'g>;
}

#[cfg(test)]
mod demo_ {
    use std::{
        borrow::BorrowMut,
        ops::{ControlFlow, Deref, DerefMut},
    };

    use pin_utils::pin_mut;
    use crate::{
        cancellation::{NonCancellableToken, TrIntoFutureMayCancel},
        x_deps::pin_utils,
    };

    use super::*;

    #[allow(dead_code)]
    async fn generic_rwlock_smoke_<B, L, T>(rwlock: B)
    where
        B: BorrowMut<L>,
        L: TrAsyncRwLock<Target = T>,
    {
        let acq = rwlock.borrow().acquire();
        pin_mut!(acq);
        let read_async = acq.as_mut().read_async();
        // let write_async = acq.write_async(); // illegal
        let ControlFlow::Continue(read_guard) = read_async
            .may_cancel_with(NonCancellableToken::pinned())
            .await
            .branch()
        else {
            panic!()
        };
        let _ = read_guard.deref();
        // let write_async = acq.write_async(); // illegal
        drop(read_guard);
        let ControlFlow::Continue(upgradable) = acq
            .as_mut()
            .upgradable_read_async()
            .may_cancel_with(NonCancellableToken::pinned())
            .await
            .branch()
        else {
            panic!()
        };
        let _ = upgradable.deref();
        let upgrade = upgradable.upgrade();
        pin_mut!(upgrade);
        let ControlFlow::Continue(mut write_guard) = upgrade
            .upgrade_async()
            .may_cancel_with(NonCancellableToken::pinned())
            .await
            .branch()
        else {
            panic!()
        };
        let _ = write_guard.deref_mut();
        let upgradable = write_guard.downgrade_to_upgradable();
        drop(upgradable)
    }
}
