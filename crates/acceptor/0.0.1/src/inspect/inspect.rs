use accepts::Accepts;
use core::marker::PhantomData;

/// `Accepts<Value>` implementation that inspects the value before passing it on.
#[must_use = "Inspect must be used to run the inspection before forwarding"]
#[derive(Debug, Clone)]
pub struct Inspect<Value, InspectFn, NextAccepts> {
    inspect_fn: InspectFn,
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, InspectFn, NextAccepts> Inspect<Value, InspectFn, NextAccepts>
where
    InspectFn: Fn(&Value),
    NextAccepts: Accepts<Value>,
{
    /// Creates a new `Inspect`.
    pub fn new(inspect_fn: InspectFn, next_acceptor: NextAccepts) -> Self {
        Self {
            inspect_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, InspectFn, NextAccepts> Accepts<Value> for Inspect<Value, InspectFn, NextAccepts>
where
    InspectFn: Fn(&Value),
    NextAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        (self.inspect_fn)(&value);
        self.next_acceptor.accept(value);
    }
}
