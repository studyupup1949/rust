use core::hash::Hash;
use std::{cell::RefCell, collections::HashSet};

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<Value>` implementation that forwards each unique value only once.
#[must_use = "DeduplicateAcceptor must be used to suppress duplicate values"]
#[derive(Debug, NextAcceptorsInternal)]
pub struct DeduplicateAcceptor<Value, NextAccepts>
where
    Value: Clone + Eq + Hash,
    NextAccepts: Accepts<Value>,
{
    seen: RefCell<HashSet<Value>>,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
}

impl<Value, NextAccepts> DeduplicateAcceptor<Value, NextAccepts>
where
    Value: Clone + Eq + Hash,
    NextAccepts: Accepts<Value>,
{
    /// Creates a new `DeduplicateAcceptor`.
    pub fn new(next_acceptor: NextAccepts) -> Self {
        Self::with_seen(RefCell::new(HashSet::new()), next_acceptor)
    }

    pub fn with_seen(seen: RefCell<HashSet<Value>>, next_acceptor: NextAccepts) -> Self {
        Self {
            seen,
            next_acceptor,
        }
    }
}

impl<Value, NextAccepts> Accepts<Value> for DeduplicateAcceptor<Value, NextAccepts>
where
    Value: Clone + Eq + Hash,
    NextAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        let mut seen = self.seen.borrow_mut();
        if seen.insert(value.clone()) {
            drop(seen);
            self.next_acceptor.accept(value);
        }
    }
}
