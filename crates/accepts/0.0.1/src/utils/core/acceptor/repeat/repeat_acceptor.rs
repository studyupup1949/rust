use core::marker::PhantomData;

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

#[must_use = "RepeatAcceptor must be used to apply the repeat count when forwarding values"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct RepeatAcceptor<Value, RepeatCountFn, NextAccepts>
where
    Value: Clone,
    RepeatCountFn: Fn(&Value) -> usize,
    NextAccepts: Accepts<Value>,
{
    repeat_count_fn: RepeatCountFn,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, RepeatCountFn, NextAccepts> RepeatAcceptor<Value, RepeatCountFn, NextAccepts>
where
    Value: Clone,
    RepeatCountFn: Fn(&Value) -> usize,
    NextAccepts: Accepts<Value>,
{
    /// Creates a new `RepeatAcceptor` with a repeat count function.
    pub fn with_fn(repeat_count_fn: RepeatCountFn, next_acceptor: NextAccepts) -> Self {
        Self {
            repeat_count_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }

    /// Creates a new `RepeatAcceptor` with a fixed repeat count.
    pub fn new(
        repeat_count: usize,
        next_acceptor: NextAccepts,
    ) -> RepeatAcceptor<Value, impl Fn(&Value) -> usize, NextAccepts> {
        RepeatAcceptor {
            repeat_count_fn: move |_| repeat_count,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, RepeatCountFn, NextAccepts> Accepts<Value>
    for RepeatAcceptor<Value, RepeatCountFn, NextAccepts>
where
    Value: Clone,
    RepeatCountFn: Fn(&Value) -> usize,
    NextAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        let n: usize = (self.repeat_count_fn)(&value);
        if n == 0 {
            return;
        }
        for _ in 0..n - 1 {
            self.next_acceptor.accept(value.clone());
        }
        self.next_acceptor.accept(value);
    }
}
