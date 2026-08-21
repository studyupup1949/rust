use accepts::AsyncAccepts;
use core::{future::Future, marker::PhantomData};

/// `AsyncAccepts<Value>` implementation that runs a closure with shared state.
///
/// The state can hold anything, including multiple "next" acceptors or
/// around-style hooks, letting the callback decide how to route each value.
#[must_use = "AsyncStatefulCallback must be used to run the async stateful callback"]
#[derive(Debug, Clone)]
pub struct AsyncStatefulCallback<Value, State, CallbackFn, CallbackFut> {
    state: State,
    callback: CallbackFn,
    _marker: PhantomData<(Value, CallbackFut)>,
}

impl<Value, State, CallbackFn, CallbackFut>
    AsyncStatefulCallback<Value, State, CallbackFn, CallbackFut>
where
    CallbackFn: Fn(&State, Value) -> CallbackFut,
    CallbackFut: Future<Output = ()>,
{
    /// Creates a new `AsyncStatefulCallback` with the given state and callback.
    pub fn new(state: State, callback: CallbackFn) -> Self {
        Self {
            state,
            callback,
            _marker: PhantomData,
        }
    }
}

impl<Value, State, CallbackFn, CallbackFut> AsyncAccepts<Value>
    for AsyncStatefulCallback<Value, State, CallbackFn, CallbackFut>
where
    CallbackFn: Fn(&State, Value) -> CallbackFut,
    CallbackFut: Future<Output = ()>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        (self.callback)(&self.state, value)
    }
}
