use core::future::Future;

pub trait LinearAsyncValueHandlerRef<Receiver, Value> {
    #[must_use]
    fn apply<'a>(
        receiver: &'a Receiver,
        value: Value,
        has_next: bool,
    ) -> impl Future<Output = Option<Value>> + 'a
    where
        Option<Value>: 'a;
}

pub trait LinearAsyncValueHandlerMut<Receiver, Value> {
    #[must_use]
    fn apply<'a>(
        receiver: &'a mut Receiver,
        value: Value,
        has_next: bool,
    ) -> impl Future<Output = Option<Value>> + 'a
    where
        Option<Value>: 'a;
}
