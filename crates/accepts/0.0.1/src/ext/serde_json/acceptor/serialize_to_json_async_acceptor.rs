use core::future::Future;

use serde::Serialize;

use crate::__internal::alloc::string::String;

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `Accepts<T>` that serializes values to JSON and delegates to an inner acceptor.
#[must_use = "SerializeToJsonAsyncAcceptor must be used to emit async JSON payloads"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct SerializeToJsonAsyncAcceptor<NextAccepts>
where
    NextAccepts: AsyncAccepts<serde_json::Result<String>>,
{
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
}

impl<NextAccepts> SerializeToJsonAsyncAcceptor<NextAccepts>
where
    NextAccepts: AsyncAccepts<serde_json::Result<String>>,
{
    pub fn new(next_acceptor: NextAccepts) -> Self {
        Self { next_acceptor }
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Value, NextAccepts> AsyncAccepts<Value> for SerializeToJsonAsyncAcceptor<NextAccepts>
where
    Value: Serialize,
    NextAccepts: AsyncAccepts<serde_json::Result<String>>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        let result = serde_json::to_string(&value);
        self.next_acceptor.accept_async(result)
    }
}
