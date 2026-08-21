use core::future::Future;

/// Receives a value and returns a future that completes once processing finishes.
///
/// This mirrors [`Accepts`](super::Accepts) for asynchronous scenarios where the
/// acceptor must yield until some work is done. The returned future is tied to
/// the lifetime of `self`, making it straightforward to borrow state inside the
/// async body.
///
/// ```rust
/// use accepts::AsyncAccepts;
/// use core::future::Future;
///
/// # use core::task::{Context, Poll};
/// # use std::boxed::Box;
/// #
/// # fn block_on<F: Future<Output = T>, T>(future: F) -> T {
/// #     let waker = core::task::Waker::noop();
/// #     let mut context = Context::from_waker(&waker);
/// #     let mut future = Box::pin(future);
/// #     match future.as_mut().poll(&mut context) {
/// #         Poll::Ready(output) => output,
/// #         Poll::Pending => panic!("Future must complete without suspension in this example"),
/// #     }
/// # }
/// #
/// struct AsyncPrinter;
///
/// impl AsyncAccepts<String> for AsyncPrinter {
///     fn accept_async<'a>(&'a self, value: String) -> impl Future<Output = ()> + 'a
///     where
///         String: 'a,
///     {
///         async move {
///             println!("{}", value);
///         }
///     }
/// }
///
/// # block_on(async {
/// let printer = AsyncPrinter;
/// printer
///     .accept_async(String::from("Hello, async world!"))
///     .await;
/// # });
/// ```
pub trait AsyncAccepts<Value> {
    #[must_use = "the returned future must be awaited to complete processing"]
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a;
}
