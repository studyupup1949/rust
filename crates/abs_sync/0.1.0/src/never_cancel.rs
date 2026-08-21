use core::{
    future::Future,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{ControlFlow, Try},
    pin::Pin,
    task::{Context, Poll},
};

use pin_utils::pin_mut;
use crate::cancellation::{NonCancellableToken, TrIntoFutureMayCancel};

/// An convenient future to implement `IntoFuture` for `TrIntoFutureMayCancel`
/// that will not actually being cancelled, and that the output will differs
/// (but deducible) from the cancellable version.
pub struct FutureForTaskNeverCancel<'a, T>
where
    T: TrIntoFutureMayCancel<'a, MayCancelOutput: Try>,
{
    task_: MaybeUninit<T>,
    _use_: PhantomData<&'a mut T>,
}

impl<'a, T> FutureForTaskNeverCancel<'a, T>
where
    T: TrIntoFutureMayCancel<'a, MayCancelOutput: Try>,
{
    pub fn new(task: T) -> Self {
        FutureForTaskNeverCancel {
            task_: MaybeUninit::new(task),
            _use_: PhantomData,
        }
    }

    async fn run_without_cancel(
        self: Pin<&mut Self>,
    ) -> <T::MayCancelOutput as Try>::Output {
        let task = unsafe { self.task_.assume_init_read() };
        let cancel = NonCancellableToken::pinned();
        let r = task.may_cancel_with(cancel).await;
        match Try::branch(r) {
            ControlFlow::Continue(x) => x,
            _ => unreachable!("[FutureForTaskNeverCancel::run_without_cancel]"),
        }
    }
}

impl<'a, T> Future for FutureForTaskNeverCancel<'a, T>
where
    T: TrIntoFutureMayCancel<'a, MayCancelOutput: Try>,
{
    type Output = <T::MayCancelOutput as Try>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let f = self.run_without_cancel();
        pin_mut!(f);
        f.poll(cx)
    }
}
