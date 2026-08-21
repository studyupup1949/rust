use accepts::Accepts;
use core::marker::PhantomData;

#[must_use = "Branch must be used to evaluate conditional forwarding"]
#[derive(Debug, Clone)]
pub struct Branch<Value, ConditionFn, TrueAccepts, FalseAccepts> {
    condition: ConditionFn,
    true_acceptor: TrueAccepts,
    false_acceptor: FalseAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, ConditionFn, TrueAccepts, FalseAccepts>
    Branch<Value, ConditionFn, TrueAccepts, FalseAccepts>
where
    ConditionFn: Fn(&Value) -> bool,
    TrueAccepts: Accepts<Value>,
    FalseAccepts: Accepts<Value>,
{
    /// Creates a new `Branch`.
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

impl<Value, ConditionFn, TrueAccepts, FalseAccepts> Accepts<Value>
    for Branch<Value, ConditionFn, TrueAccepts, FalseAccepts>
where
    ConditionFn: Fn(&Value) -> bool,
    TrueAccepts: Accepts<Value>,
    FalseAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        if (self.condition)(&value) {
            self.true_acceptor.accept(value);
        } else {
            self.false_acceptor.accept(value);
        }
    }
}
