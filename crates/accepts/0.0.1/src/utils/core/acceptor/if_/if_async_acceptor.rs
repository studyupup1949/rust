use core::{future::Future, marker::PhantomData};

use crate::{core_traits::AsyncAccepts, macros::internal::codegen::auto_impl_dyn_internal};

/// An `AsyncAccepts` implementation that forwards values when the predicate
/// resolves to `true`.
#[must_use = "IfAsyncAcceptor must be used to evaluate conditional async forwarding"]
#[derive(Debug, Clone)]
pub struct IfAsyncAcceptor<Value, ConditionFn, ConditionFut, ThenAccepts, ElseAccepts>
where
    ConditionFn: Fn(&Value) -> ConditionFut,
    ConditionFut: Future<Output = bool>,
    ThenAccepts: AsyncAccepts<Value>,
    ElseAccepts: AsyncAccepts<Value>,
{
    condition: ConditionFn,
    then_acceptor: ThenAccepts,
    else_acceptor: ElseAccepts,
    _marker: PhantomData<(Value, ConditionFut)>,
}

impl<Value, ConditionFn, ConditionFut, ThenAccepts, ElseAccepts>
    IfAsyncAcceptor<Value, ConditionFn, ConditionFut, ThenAccepts, ElseAccepts>
where
    ConditionFn: Fn(&Value) -> ConditionFut,
    ConditionFut: Future<Output = bool>,
    ThenAccepts: AsyncAccepts<Value>,
    ElseAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `IfAsyncAcceptor.
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

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Value, ConditionFn, ConditionFut, ThenAccepts, ElseAccepts> AsyncAccepts<Value>
    for IfAsyncAcceptor<Value, ConditionFn, ConditionFut, ThenAccepts, ElseAccepts>
where
    ConditionFn: Fn(&Value) -> ConditionFut,
    ConditionFut: Future<Output = bool>,
    ThenAccepts: AsyncAccepts<Value>,
    ElseAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async {
            if (self.condition)(&value).await {
                self.then_acceptor.accept_async(value).await;
            } else {
                self.else_acceptor.accept_async(value).await;
            }
        }
    }
}

#[cfg(feature = "alloc")]
use crate::core_traits::{DynAsyncAccepts, NextAcceptors};
#[cfg(feature = "alloc")]
impl<Value, ConditionFn, ConditionFut, ThenAccepts, ElseAccepts> NextAcceptors
    for IfAsyncAcceptor<Value, ConditionFn, ConditionFut, ThenAccepts, ElseAccepts>
where
    ConditionFn: Fn(&Value) -> ConditionFut,
    ConditionFut: Future<Output = bool>,
    ThenAccepts: AsyncAccepts<Value> + DynAsyncAccepts<Value>,
    ElseAccepts: AsyncAccepts<Value> + DynAsyncAccepts<Value>,
{
    type Acceptor<'a>
        = dyn DynAsyncAccepts<Value> + 'a
    where
        Self: 'a;

    type Iter<'a>
        = core::array::IntoIter<&'a Self::Acceptor<'a>, 2>
    where
        Self: 'a;
    fn next_acceptors(&self) -> Self::Iter<'_> {
        let then: &dyn DynAsyncAccepts<Value> = &self.then_acceptor;
        let else_: &dyn DynAsyncAccepts<Value> = &self.else_acceptor;
        [then, else_].into_iter()
    }
}
