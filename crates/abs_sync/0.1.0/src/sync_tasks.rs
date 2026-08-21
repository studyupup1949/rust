use core::{
    convert::Infallible,
    ops::{ControlFlow, Try},
    pin::Pin,
};

use crate::cancellation::{NonCancellableToken, TrCancellationToken};

pub trait TrSyncTask: Sized {
    type Output: Sized;

    fn may_cancel_with<C>(
        self,
        cancel: Pin<&mut C>,
    ) -> impl Try<Output = Self::Output>
    where
        C: TrCancellationToken;

    fn wait(self) -> Self::Output {
        let r = self.may_cancel_with(NonCancellableToken::pinned());
        match Try::branch(r) {
            ControlFlow::Continue(t) => t,
            ControlFlow::Break(_) => unreachable!("[TrSyncTask::wait]"),
        }
    }
}

pub struct CompletedSyncTask<T>(T);

impl<T> CompletedSyncTask<T> {
    #[inline]
    pub const fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> From<T> for CompletedSyncTask<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> TrSyncTask for CompletedSyncTask<T> {
    type Output = T;

    fn may_cancel_with<C>(
        self, 
        cancel: Pin<&mut C>,
    ) -> impl Try<Output = Self::Output>
    where
        C: TrCancellationToken,
    {
        let _ = cancel;
        Result::<T, Infallible>::Ok(self.0)
    }
}
