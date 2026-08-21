use serde::Serialize;

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<T>` implementation that serializes values with a serializer created by the provided factory.
#[must_use = "SerializeAcceptor must be used to drive serialization before forwarding"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct SerializeAcceptor<SerializerFactory, NextAccepts> {
    serializer_factory: SerializerFactory,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
}

impl<SerializerFactory, NextAccepts> SerializeAcceptor<SerializerFactory, NextAccepts> {
    pub fn new(serializer_factory: SerializerFactory, next_acceptor: NextAccepts) -> Self {
        Self {
            serializer_factory,
            next_acceptor,
        }
    }
}

impl<Value, SerializerFactory, NextAccepts, Ser> Accepts<Value>
    for SerializeAcceptor<SerializerFactory, NextAccepts>
where
    Value: Serialize,
    SerializerFactory: Fn() -> Ser,
    NextAccepts: Accepts<Result<Ser::Ok, Ser::Error>>,
    Ser: serde::Serializer,
{
    fn accept(&self, value: Value) {
        let serializer = (self.serializer_factory)();
        let result = value.serialize(serializer);
        self.next_acceptor.accept(result);
    }
}
