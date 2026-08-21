use core::marker::PhantomData;

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<I>` implementation that forwards each item to the next acceptor.
#[must_use = "IteratorAcceptor must be used to forward iterator items individually"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct IteratorAcceptor<Iter, NextAccepts>
where
    Iter: IntoIterator,
    NextAccepts: Accepts<Iter::Item>,
{
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Iter>,
}

impl<Iter, NextAccepts> IteratorAcceptor<Iter, NextAccepts>
where
    Iter: IntoIterator,
    NextAccepts: Accepts<Iter::Item>,
{
    /// Creates a new `IteratorAcceptor`.
    pub fn new(next: NextAccepts) -> Self {
        Self {
            next_acceptor: next,
            _marker: PhantomData,
        }
    }
}

impl<Iter, NextAccepts> Accepts<Iter> for IteratorAcceptor<Iter, NextAccepts>
where
    Iter: IntoIterator,
    NextAccepts: Accepts<Iter::Item>,
{
    fn accept(&self, iter: Iter) {
        for item in iter {
            self.next_acceptor.accept(item);
        }
    }
}
