use core::{
    future::{Future, IntoFuture}, marker::PhantomData, mem::MaybeUninit, ops::{AsyncFnOnce, Try}
};

use crate::cancellation::{NonCancellableToken, TrMayCancel};

/// An convenient future to implement `IntoFuture` for `TrIntoFutureMayCancel`
/// that will not actually being cancelled, and that the output will differs
/// (but deducible) from the cancellable version.
pub struct NeverCancelAsync<'a, T>
where
    T: TrMayCancel<'a, MayCancelOutput: Try>,
{
    task_: MaybeUninit<T>,
    _use_: PhantomData<&'a ()>,
}

impl<'a, T> NeverCancelAsync<'a, T>
where
    T: TrMayCancel<'a, MayCancelOutput: Try>,
{
    pub fn new(task: T) -> Self {
        NeverCancelAsync {
            task_: MaybeUninit::new(task),
            _use_: PhantomData,
        }
    }

    async fn run_without_cancel(self) -> T::MayCancelOutput {
        let task = unsafe { self.task_.assume_init_read() };
        let cancel = NonCancellableToken::pinned();
        task.may_cancel_with(cancel).await
    }
}

impl<'a, T> AsyncFnOnce<()> for NeverCancelAsync<'a, T>
where
    T: TrMayCancel<'a, MayCancelOutput: Try>,
{
    type CallOnceFuture = impl Future<Output = Self::Output>;
    type Output = T::MayCancelOutput;

    extern "rust-call" fn async_call_once(
        self,
        _: (),
    ) -> Self::CallOnceFuture {
        self.run_without_cancel()
    }
}

impl<'a, T> IntoFuture for NeverCancelAsync<'a, T>
where
    T: TrMayCancel<'a, MayCancelOutput: Try>,
{
    type IntoFuture = <Self as AsyncFnOnce<()>>::CallOnceFuture;
    type Output = <Self::IntoFuture as Future>::Output;

    fn into_future(self) -> Self::IntoFuture {
        self()
    }
}
