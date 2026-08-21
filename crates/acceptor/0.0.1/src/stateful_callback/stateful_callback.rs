use accepts::Accepts;
use core::marker::PhantomData;

/// `Accepts<Value>` implementation that runs a closure with shared state.
///
/// The state can hold anything, including multiple "next" acceptors or
/// around-style hooks, letting the callback decide how to route each value.
#[must_use = "StatefulCallback must be used to run the stateful callback"]
#[derive(Debug, Clone)]
pub struct StatefulCallback<Value, State, CallbackFn> {
    state: State,
    callback: CallbackFn,
    _marker: PhantomData<Value>,
}

impl<Value, State, CallbackFn> StatefulCallback<Value, State, CallbackFn>
where
    CallbackFn: Fn(&State, Value),
{
    /// Creates a new `StatefulCallback` with the given state and callback.
    pub fn new(state: State, callback: CallbackFn) -> Self {
        Self {
            state,
            callback,
            _marker: PhantomData,
        }
    }
}

impl<Value, State, CallbackFn> Accepts<Value> for StatefulCallback<Value, State, CallbackFn>
where
    CallbackFn: Fn(&State, Value),
{
    fn accept(&self, value: Value) {
        (self.callback)(&self.state, value);
    }
}
