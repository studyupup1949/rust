pub trait LinearValueHandlerRef<Receiver, Value> {
    #[must_use]
    fn apply(receiver: &Receiver, value: Value, has_next: bool) -> Option<Value>;
}

pub trait LinearValueHandlerMut<Receiver, Value> {
    #[must_use]
    fn apply(receiver: &mut Receiver, value: Value, has_next: bool) -> Option<Value>;
}
