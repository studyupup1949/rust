use core::marker::PhantomData;

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<Value>` implementation that inspects the value before passing it on.
#[must_use = "InspectAcceptor must be used to run the inspection before forwarding"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct InspectAcceptor<Value, InspectFn, NextAccepts>
where
    InspectFn: Fn(&Value),
    NextAccepts: Accepts<Value>,
{
    inspect_fn: InspectFn,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, InspectFn, NextAccepts> InspectAcceptor<Value, InspectFn, NextAccepts>
where
    InspectFn: Fn(&Value),
    NextAccepts: Accepts<Value>,
{
    /// Creates a new `InspectAcceptor`.
    pub fn new(inspect_fn: InspectFn, next_acceptor: NextAccepts) -> Self {
        Self {
            inspect_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, InspectFn, NextAccepts> Accepts<Value>
    for InspectAcceptor<Value, InspectFn, NextAccepts>
where
    InspectFn: Fn(&Value),
    NextAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        (self.inspect_fn)(&value);
        self.next_acceptor.accept(value);
    }
}
