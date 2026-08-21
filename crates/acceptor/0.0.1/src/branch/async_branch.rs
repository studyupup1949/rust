use accepts::AsyncAccepts;
use core::{future::Future, marker::PhantomData};

/// An `AsyncAccepts` implementation that forwards values when the predicate
/// resolves to `true`.
#[must_use = "AsyncBranch must be used to evaluate conditional async forwarding"]
#[derive(Debug, Clone)]
pub struct AsyncBranch<Value, ConditionFn, ConditionFut, TrueAccepts, FalseAccepts> {
    condition: ConditionFn,
    true_acceptor: TrueAccepts,
    false_acceptor: FalseAccepts,
    _marker: PhantomData<(Value, ConditionFut)>,
}

impl<Value, ConditionFn, ConditionFut, TrueAccepts, FalseAccepts>
    AsyncBranch<Value, ConditionFn, ConditionFut, TrueAccepts, FalseAccepts>
where
    ConditionFn: Fn(&Value) -> ConditionFut,
    ConditionFut: Future<Output = bool>,
    TrueAccepts: AsyncAccepts<Value>,
    FalseAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `AsyncBranch`.
    pub fn new(
        condition: ConditionFn,
        true_acceptor: TrueAccepts,
        false_acceptor: FalseAccepts,
    ) -> Self {
        Self {
            condition,
            true_acceptor,
            false_acceptor,
            _marker: PhantomData,
        }
    }

    pub fn true_acceptor(&self) -> &TrueAccepts {
        &self.true_acceptor
    }

    pub fn true_acceptor_mut(&mut self) -> &mut TrueAccepts {
        &mut self.true_acceptor
    }

    pub fn false_acceptor(&self) -> &FalseAccepts {
        &self.false_acceptor
    }

    pub fn false_acceptor_mut(&mut self) -> &mut FalseAccepts {
        &mut self.false_acceptor
    }
}

impl<Value, ConditionFn, ConditionFut, TrueAccepts, FalseAccepts> AsyncAccepts<Value>
    for AsyncBranch<Value, ConditionFn, ConditionFut, TrueAccepts, FalseAccepts>
where
    ConditionFn: Fn(&Value) -> ConditionFut,
    ConditionFut: Future<Output = bool>,
    TrueAccepts: AsyncAccepts<Value>,
    FalseAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async {
            if (self.condition)(&value).await {
                self.true_acceptor.accept_async(value).await;
            } else {
                self.false_acceptor.accept_async(value).await;
            }
        }
    }
}
