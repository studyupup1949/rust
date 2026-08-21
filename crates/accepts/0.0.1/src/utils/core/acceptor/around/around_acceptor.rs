use core::marker::PhantomData;

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<Value>` that runs hooks with shared context before and after delegating.
#[must_use = "AroundAcceptor must be used to run the before/after hooks around accepted values"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct AroundAcceptor<Value, Context, BeforeFn, AfterFn, NextAccepts>
where
    BeforeFn: Fn(&Context),
    AfterFn: Fn(&Context),
    NextAccepts: Accepts<Value>,
{
    context: Context,
    before_fn: BeforeFn,
    after_fn: AfterFn,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, Context, BeforeFn, AfterFn, NextAccepts>
    AroundAcceptor<Value, Context, BeforeFn, AfterFn, NextAccepts>
where
    BeforeFn: Fn(&Context),
    AfterFn: Fn(&Context),
    NextAccepts: Accepts<Value>,
{
    pub fn new(
        context: Context,
        before_fn: BeforeFn,
        after_fn: AfterFn,
        next_acceptor: NextAccepts,
    ) -> Self {
        Self {
            context,
            before_fn,
            after_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, Context, BeforeFn, AfterFn, NextAccepts> Accepts<Value>
    for AroundAcceptor<Value, Context, BeforeFn, AfterFn, NextAccepts>
where
    BeforeFn: Fn(&Context),
    AfterFn: Fn(&Context),
    NextAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        (self.before_fn)(&self.context);
        self.next_acceptor.accept(value);
        (self.after_fn)(&self.context);
    }
}
