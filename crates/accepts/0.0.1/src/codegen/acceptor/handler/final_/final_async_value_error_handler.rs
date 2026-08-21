use core::future::Future;

pub trait FinalAsyncValueErrorHandlerRef<Receiver, Value, Error> {
    #[must_use]
    fn apply<'a>(
        receiver: &'a Receiver,
        value: Value,
    ) -> impl Future<Output = Result<(), Error>> + 'a
    where
        Error: 'a;
}

pub trait FinalAsyncValueErrorHandlerMut<Receiver, Value, Error> {
    #[must_use]
    fn apply<'a>(
        receiver: &'a mut Receiver,
        value: Value,
    ) -> impl Future<Output = Result<(), Error>> + 'a
    where
        Error: 'a;
}
