pub trait FinalValueErrorHandlerRef<Receiver, Value, Error> {
    #[must_use]
    fn apply(receiver: &Receiver, value: Value) -> Result<(), Error>;
}

pub trait FinalValueErrorHandlerMut<Receiver, Value, Error> {
    #[must_use]
    fn apply(receiver: &mut Receiver, value: Value) -> Result<(), Error>;
}
