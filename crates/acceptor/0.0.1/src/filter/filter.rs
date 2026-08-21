use accepts::Accepts;
use core::marker::PhantomData;

/// `Accepts<Value>` implementation that forwards values when the predicate returns `true`.
#[must_use = "Filter must be used to apply the filter predicate"]
#[derive(Debug, Clone)]
pub struct Filter<Value, Predicate, NextAccepts> {
    predicate: Predicate,
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, Predicate, NextAccepts> Filter<Value, Predicate, NextAccepts>
where
    Predicate: Fn(&Value) -> bool,
    NextAccepts: Accepts<Value>,
{
    /// Creates a new `Filter`.
    pub fn new(predicate: Predicate, next_acceptor: NextAccepts) -> Self {
        Self {
            predicate,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, Predicate, NextAccepts> Accepts<Value> for Filter<Value, Predicate, NextAccepts>
where
    Predicate: Fn(&Value) -> bool,
    NextAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        if (self.predicate)(&value) {
            self.next_acceptor.accept(value);
        }
    }
}
