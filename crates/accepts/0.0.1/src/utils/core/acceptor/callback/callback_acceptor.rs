use core::marker::PhantomData;

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<T>` implementation that calls a closure.
#[must_use = "CallbackAcceptor must be used to run the callback chain"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct CallbackAcceptor<Value, CallbackFn, NextAccepts = ()>
where
    CallbackFn: Fn(Value, Option<&NextAccepts>),
    NextAccepts: Accepts<Value>,
{
    callback: CallbackFn,
    #[next_acceptor(option_once, ref, mut)]
    next_acceptor: Option<NextAccepts>,
    _marker: PhantomData<Value>,
}
impl<Value, CallbackFn, NextAccepts> CallbackAcceptor<Value, CallbackFn, NextAccepts>
where
    CallbackFn: Fn(Value, Option<&NextAccepts>),
    NextAccepts: Accepts<Value>,
{
    pub fn with_next(callback: CallbackFn, next_acceptor: Option<NextAccepts>) -> Self {
        Self {
            callback,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value> CallbackAcceptor<Value, fn(Value, Option<&()>), ()>
where
    (): Accepts<Value>,
{
    /// Creates a `FnAcceptor` from a simple closure.
    pub fn new<G>(callback: G) -> CallbackAcceptor<Value, impl Fn(Value, Option<&()>), ()>
    where
        G: Fn(Value),
    {
        CallbackAcceptor::with_next(move |value, _| callback(value), None)
    }
}

impl<Value, CallbackFn, NextAccepts> Accepts<Value>
    for CallbackAcceptor<Value, CallbackFn, NextAccepts>
where
    CallbackFn: Fn(Value, Option<&NextAccepts>),
    NextAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        (self.callback)(value, self.next_acceptor.as_ref())
    }
}
