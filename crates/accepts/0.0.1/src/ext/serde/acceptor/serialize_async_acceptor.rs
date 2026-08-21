use serde::Serialize;

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `AsyncAccepts<T>` implementation that serializes values with a serializer created by the provided factory.
#[must_use = "SerializeAsyncAcceptor must be used to drive async serialization before forwarding"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct SerializeAsyncAcceptor<SerializerFactory, NextAccepts> {
    serializer_factory: SerializerFactory,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
}

impl<SerializerFactory, NextAccepts> SerializeAsyncAcceptor<SerializerFactory, NextAccepts> {
    pub fn new(serializer_factory: SerializerFactory, next_acceptor: NextAccepts) -> Self {
        Self {
            serializer_factory,
            next_acceptor,
        }
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Value, SerializerFactory, NextAccepts, Ser> AsyncAccepts<Value>
    for SerializeAsyncAcceptor<SerializerFactory, NextAccepts>
where
    Value: Serialize,
    SerializerFactory: Fn() -> Ser,
    NextAccepts: AsyncAccepts<Result<Ser::Ok, Ser::Error>>,
    Ser: serde::Serializer,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl core::future::Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async move {
            let serializer = (self.serializer_factory)();
            let result = value.serialize(serializer);
            self.next_acceptor.accept_async(result).await;
        }
    }
}
