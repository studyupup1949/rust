use core::future::Future;

pub trait FinalAsyncValueHandlerRef<Receiver, Value> {
    #[must_use]
    fn apply<'a>(receiver: &'a Receiver, value: Value) -> impl Future<Output = ()> + 'a;
}

pub trait FinalAsyncValueHandlerMut<Receiver, Value> {
    #[must_use]
    fn apply<'a>(receiver: &'a mut Receiver, value: Value) -> impl Future<Output = ()> + 'a;
}
