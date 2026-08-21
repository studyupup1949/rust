use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<T>` implementation that forwards only once.
#[must_use = "OnceAcceptor must be used to enforce single acceptance"]
#[derive(Debug, NextAcceptorsInternal)]
pub struct OnceAcceptor<Value, NextAccepts>
where
    NextAccepts: Accepts<Value>,
{
    executed: AtomicBool,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, NextAccepts> OnceAcceptor<Value, NextAccepts>
where
    NextAccepts: Accepts<Value>,
{
    /// Creates a new `OnceAcceptor`.
    pub fn new(next_acceptor: NextAccepts) -> Self {
        Self {
            executed: AtomicBool::new(false),
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, NextAccepts> Accepts<Value> for OnceAcceptor<Value, NextAccepts>
where
    NextAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        if !self.executed.swap(true, Ordering::SeqCst) {
            self.next_acceptor.accept(value);
        }
    }
}
