use serde::Serialize;

use crate::__internal::alloc::string::String;

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<T>` that serializes values to JSON and delegates to an inner acceptor.
#[must_use = "SerializeToJsonAcceptor must be used to emit JSON payloads"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct SerializeToJsonAcceptor<NextAccepts>
where
    NextAccepts: Accepts<serde_json::Result<String>>,
{
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
}

impl<NextAccepts> SerializeToJsonAcceptor<NextAccepts>
where
    NextAccepts: Accepts<serde_json::Result<String>>,
{
    pub fn new(next_acceptor: NextAccepts) -> Self {
        Self { next_acceptor }
    }
}

impl<Value, NextAccepts> Accepts<Value> for SerializeToJsonAcceptor<NextAccepts>
where
    Value: Serialize,
    NextAccepts: Accepts<serde_json::Result<String>>,
{
    fn accept(&self, value: Value) {
        let result = serde_json::to_string(&value);
        self.next_acceptor.accept(result)
    }
}
