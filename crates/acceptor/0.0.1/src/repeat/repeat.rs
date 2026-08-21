use accepts::Accepts;
use core::marker::PhantomData;

#[must_use = "Repeat must be used to apply the repeat count when forwarding values"]
#[derive(Debug, Clone)]
pub struct Repeat<Value, RepeatCountFn, NextAccepts> {
    repeat_count_fn: RepeatCountFn,
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, RepeatCountFn, NextAccepts> Repeat<Value, RepeatCountFn, NextAccepts>
where
    Value: Clone,
    RepeatCountFn: for<'a> Fn(&'a Value) -> usize,
    NextAccepts: Accepts<Value>,
{
    /// Creates a new `Repeat` with a repeat count function.
    pub fn with_fn(repeat_count_fn: RepeatCountFn, next_acceptor: NextAccepts) -> Self {
        Self {
            repeat_count_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, NextAccepts> Repeat<Value, fn(&Value) -> usize, NextAccepts>
where
    Value: Clone,
    NextAccepts: Accepts<Value>,
{
    /// Creates a new `Repeat` with a fixed repeat count.
    pub fn new(
        repeat_count: usize,
        next_acceptor: NextAccepts,
    ) -> Repeat<Value, impl for<'a> Fn(&'a Value) -> usize, NextAccepts> {
        let repeat_count_fn = move |_: &Value| repeat_count;
        Repeat::with_fn(repeat_count_fn, next_acceptor)
    }
}

impl<Value, RepeatCountFn, NextAccepts> Accepts<Value> for Repeat<Value, RepeatCountFn, NextAccepts>
where
    Value: Clone,
    RepeatCountFn: for<'a> Fn(&'a Value) -> usize,
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
