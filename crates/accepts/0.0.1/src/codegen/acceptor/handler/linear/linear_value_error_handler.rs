pub trait LinearValueErrorHandlerRef<Receiver, Value, Error> {
    #[must_use]
    fn apply(receiver: &Receiver, value: Value, has_next: bool) -> Result<Option<Value>, Error>;
}

pub trait LinearValueErrorHandlerMut<Receiver, Value, Error> {
    #[must_use]
    fn apply(receiver: &mut Receiver, value: Value, has_next: bool)
    -> Result<Option<Value>, Error>;
}
