use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};

use accepts::Accepts;

/// `Accepts<T>` implementation that forwards only once.
#[must_use = "Once must be used to enforce single acceptance"]
#[derive(Debug)]
pub struct Once<Value, NextAccepts> {
    executed: AtomicBool,
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, NextAccepts> Once<Value, NextAccepts>
where
    NextAccepts: Accepts<Value>,
{
    /// Creates a new `Once`.
    pub fn new(next_acceptor: NextAccepts) -> Self {
        Self {
            executed: AtomicBool::new(false),
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, NextAccepts> Accepts<Value> for Once<Value, NextAccepts>
where
    NextAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        if !self.executed.swap(true, Ordering::SeqCst) {
            self.next_acceptor.accept(value);
        }
    }
}
