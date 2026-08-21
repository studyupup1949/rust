use crate::__internal::alloc::boxed::Box;
use core::{future::Future, pin::Pin};

/// Asynchronous acceptor that returns a trait-object future.
///
/// This trait provides an asynchronous acceptor that returns a boxed trait
/// object future.  It is especially useful when the consumer needs to store
/// heterogeneous acceptors behind trait objects.
///
/// ```rust
/// use accepts::core_traits::{AsyncAccepts, DynAsyncAccepts};
/// use core::{future::Future, pin::Pin};
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
/// impl DynAsyncAccepts<String> for TaskRunner {
///     fn accept_async_dyn<'a>(&'a self, task: String) -> Pin<Box<dyn Future<Output = ()> + 'a>>
///     where
///         String: 'a,
///     {
///         Box::pin(self.accept_async(task))
///     }
/// }
/// ```
pub trait DynAsyncAccepts<Value> {
    fn accept_async_dyn<'a>(&'a self, value: Value) -> Pin<Box<dyn Future<Output = ()> + 'a>>
    where
        Value: 'a;
}
