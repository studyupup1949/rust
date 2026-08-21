use core::future::Future;

pub trait LinearAsyncValueErrorHandlerRef<Receiver, Value, Error> {
    #[must_use]
    fn apply<'a>(
        receiver: &'a Receiver,
        value: Value,
        has_next: bool,
    ) -> impl Future<Output = Result<Option<Value>, Error>> + 'a
    where
        Option<Value>: 'a,
        Error: 'a;
}

pub trait LinearAsyncValueErrorHandlerMut<Receiver, Value, Error> {
    #[must_use]
    fn apply<'a>(
        receiver: &'a mut Receiver,
        value: Value,
        has_next: bool,
    ) -> impl Future<Output = Result<Option<Value>, Error>> + 'a
    where
        Option<Value>: 'a,
        Error: 'a;
}
