use core::marker::PhantomData;

use crate::core_traits::{Accepts, NextAcceptors};

#[must_use = "IfAcceptor must be used to evaluate conditional forwarding"]
#[derive(Debug, Clone)]
pub struct IfAcceptor<Value, ConditionFn, ThenAccepts, ElseAccepts>
where
    ConditionFn: Fn(&Value) -> bool,
    ThenAccepts: Accepts<Value>,
    ElseAccepts: Accepts<Value>,
{
    condition: ConditionFn,
    then_acceptor: ThenAccepts,
    else_acceptor: ElseAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, ConditionFn, ThenAccepts, ElseAccepts>
    IfAcceptor<Value, ConditionFn, ThenAccepts, ElseAccepts>
where
    ConditionFn: Fn(&Value) -> bool,
    ThenAccepts: Accepts<Value>,
    ElseAccepts: Accepts<Value>,
{
    /// Creates a new `IfAcceptor`.
    pub fn new(
        condition: ConditionFn,
        then_acceptor: ThenAccepts,
        else_acceptor: ElseAccepts,
    ) -> Self {
        Self {
            condition,
            then_acceptor,
            else_acceptor,
            _marker: PhantomData,
        }
    }

    pub fn then_acceptor(&self) -> &ThenAccepts {
        &self.then_acceptor
    }

    pub fn then_acceptor_mut(&mut self) -> &mut ThenAccepts {
        &mut self.then_acceptor
    }

    pub fn else_acceptor(&self) -> &ElseAccepts {
        &self.else_acceptor
    }

    pub fn else_acceptor_mut(&mut self) -> &mut ElseAccepts {
        &mut self.else_acceptor
    }
}

impl<Value, ConditionFn, ThenAccepts, ElseAccepts> Accepts<Value>
    for IfAcceptor<Value, ConditionFn, ThenAccepts, ElseAccepts>
where
    ConditionFn: Fn(&Value) -> bool,
    ThenAccepts: Accepts<Value>,
    ElseAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        if (self.condition)(&value) {
            self.then_acceptor.accept(value);
        } else {
            self.else_acceptor.accept(value);
        }
    }
}

impl<Value, ConditionFn, ThenAccepts, ElseAccepts> NextAcceptors
    for IfAcceptor<Value, ConditionFn, ThenAccepts, ElseAccepts>
where
    ConditionFn: Fn(&Value) -> bool,
    ThenAccepts: Accepts<Value>,
    ElseAccepts: Accepts<Value>,
{
    type Acceptor<'a>
        = dyn Accepts<Value> + 'a
    where
        Self: 'a;

    type Iter<'a>
        = core::array::IntoIter<&'a Self::Acceptor<'a>, 2>
    where
        Self: 'a;
    fn next_acceptors(&self) -> Self::Iter<'_> {
        let then: &dyn Accepts<Value> = &self.then_acceptor;
        let else_: &dyn Accepts<Value> = &self.else_acceptor;
        [then, else_].into_iter()
    }
}
