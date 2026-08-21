use core::{
    convert::Infallible,
    ops::{ControlFlow, Try},
    pin::Pin,
};

use crate::cancellation::{NonCancellableToken, TrCancellationToken};

pub trait TrSyncTask: Sized {
    type MayCancelOutput: Sized;

    fn may_cancel_with<C>(
        self,
        cancel: Pin<&mut C>,
    ) -> impl Try<Output = Self::MayCancelOutput>
    where
        C: TrCancellationToken;

    fn wait(self) -> Self::MayCancelOutput {
        let r = self.may_cancel_with(NonCancellableToken::pinned());
        match Try::branch(r) {
            ControlFlow::Continue(t) => t,
            ControlFlow::Break(_) => unreachable!("[TrSyncTask::wait]"),
        }
    }
}

pub struct CompletedSyncTask<T>(T);

impl<T> CompletedSyncTask<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    pub fn may_cancel_with<C>(
        self,
        cancel: Pin<&mut C>,
    ) -> impl Try<Output = T>
    where
        C: TrCancellationToken,
    {
        let _ = cancel;
        Result::<T, Infallible>::Ok(self.0)
    }

    pub fn wait(self) -> T {
        self.0
    }
}

impl<T> From<T> for CompletedSyncTask<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> TrSyncTask for CompletedSyncTask<T> {
    type MayCancelOutput = T;

    #[inline]
    fn may_cancel_with<C>(
        self, 
        cancel: Pin<&mut C>,
    ) -> impl Try<Output = Self::MayCancelOutput>
    where
        C: TrCancellationToken,
    {
        CompletedSyncTask::may_cancel_with(self, cancel)
    }

    #[inline]
    fn wait(self) -> Self::MayCancelOutput {
        CompletedSyncTask::wait(self)
    }
}
