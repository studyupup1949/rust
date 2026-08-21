use core::{
    cell::SyncUnsafeCell,
    future::{self, Future, IntoFuture},
    pin::Pin,
};

pub trait TrCancellationToken: Clone {
    fn is_cancelled(&self) -> bool;

    fn can_be_cancelled(&self) -> bool;

    fn cancellation(self: Pin<&mut Self>) -> impl IntoFuture<Output = ()>;
}

pub trait TrIntoFutureMayCancel<'a>
where
    Self: 'a + Sized,
{
    type MayCancelOutput;

    fn may_cancel_with<C>(
        self,
        cancel: Pin<&'a mut C>,
    ) -> impl Future<Output = Self::MayCancelOutput>
    where
        C: TrCancellationToken;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CancelledToken;

impl CancelledToken {
    pub const fn new() -> Self {
        CancelledToken
    }

    pub fn pinned() -> Pin<&'static mut Self> {
        static mut SHARED: SyncUnsafeCell<CancelledToken> =
            SyncUnsafeCell::new(CancelledToken::new());
        unsafe {
            Pin::new_unchecked(SHARED.get_mut())
        }
    }

    pub fn cancellation(self: Pin<&mut Self>) -> future::Ready<()> {
        future::ready(())
    }
}

impl TrCancellationToken for CancelledToken {
    fn is_cancelled(&self) -> bool {
        true
    }

    fn can_be_cancelled(&self) -> bool {
        false
    }

    fn cancellation(self: Pin<&mut Self>) -> impl IntoFuture<Output = ()> {
        CancelledToken::cancellation(self)
    }
}

/// A cancellation token that will never be cancelled, usually used
/// as a dummy for `TrCancellationToken`.
#[derive(Debug, Default, Clone, Copy)]
pub struct NonCancellableToken;

impl NonCancellableToken {
    pub const fn new() -> Self {
        NonCancellableToken
    }

    pub fn pinned() -> Pin<&'static mut Self> {
        static mut SHARED: SyncUnsafeCell<NonCancellableToken> =
            SyncUnsafeCell::new(NonCancellableToken::new());
        unsafe {
            Pin::new_unchecked(SHARED.get_mut())
        }
    }

    pub fn cancellation(self: Pin<&mut Self>) -> future::Pending<()> {
        future::pending()
    }
}

impl TrCancellationToken for NonCancellableToken {
    fn is_cancelled(&self) -> bool {
        false
    }

    fn can_be_cancelled(&self) -> bool {
        false
    }

    fn cancellation(self: Pin<&mut Self>) -> impl IntoFuture<Output = ()> {
        NonCancellableToken::cancellation(self)
    }
}
