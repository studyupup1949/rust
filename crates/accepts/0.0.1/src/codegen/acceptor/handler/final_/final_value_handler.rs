pub trait FinalValueHandlerRef<Receiver, Value> {
    fn apply(receiver: &Receiver, value: Value);
}

pub trait FinalValueHandlerMut<Receiver, Value> {
    fn apply(receiver: &mut Receiver, value: Value);
}
