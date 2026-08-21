use core::marker::PhantomData;

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<Value>` implementation that forwards values when the predicate returns `true`.
#[must_use = "FilterAcceptor must be used to apply the filter predicate"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct FilterAcceptor<Value, Predicate, NextAccepts>
where
    Predicate: Fn(&Value) -> bool,
    NextAccepts: Accepts<Value>,
{
    predicate: Predicate,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, Predicate, NextAccepts> FilterAcceptor<Value, Predicate, NextAccepts>
where
    Predicate: Fn(&Value) -> bool,
    NextAccepts: Accepts<Value>,
{
    /// Creates a new `FilterAcceptor`.
    pub fn new(predicate: Predicate, next_acceptor: NextAccepts) -> Self {
        Self {
            predicate,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, Predicate, NextAccepts> Accepts<Value> for FilterAcceptor<Value, Predicate, NextAccepts>
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
