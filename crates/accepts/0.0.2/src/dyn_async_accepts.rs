use crate::alloc::boxed::Box;
use core::{future::Future, pin::Pin};

/// Asynchronous acceptor that returns a boxed trait-object future.
///
/// This trait is automatically implemented for every [`AsyncAccepts`](crate::AsyncAccepts) type when
/// the `alloc` feature is enabled. It is useful when the consumer needs to
/// store heterogeneous async acceptors behind trait objects.
///
/// ```rust
/// use accepts::{AsyncAccepts, DynAsyncAccepts};
/// use core::future::Future;
///
/// struct TaskRunner;
///
/// impl AsyncAccepts<String> for TaskRunner {
///     fn accept_async<'a>(&'a self, task: String) -> impl Future<Output = ()> + 'a
///     where
///         String: 'a,
///     {
///         async move {
///             println!("{}", task);
///         }
///     }
/// }
///
/// // DynAsyncAccepts is provided automatically when `alloc` is enabled.
/// let runner = TaskRunner;
/// let dyn_runner: &dyn DynAsyncAccepts<String> = &runner;
/// # let _ = dyn_runner;
/// ```
pub trait DynAsyncAccepts<Value> {
    #[must_use = "the returned future must be awaited to complete processing"]
    fn accept_async_dyn<'a>(&'a self, value: Value) -> Pin<Box<dyn Future<Output = ()> + 'a>>
    where
        Value: 'a;
}

impl<T, Value> DynAsyncAccepts<Value> for T
where
    T: crate::AsyncAccepts<Value> + ?Sized,
{
    fn accept_async_dyn<'a>(&'a self, value: Value) -> Pin<Box<dyn Future<Output = ()> + 'a>>
    where
        Value: 'a,
    {
        Box::pin(self.accept_async(value))
    }
}
