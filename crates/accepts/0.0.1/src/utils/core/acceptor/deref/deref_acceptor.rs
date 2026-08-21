use core::{
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::core_traits::{Accepts, AsyncAccepts, NextAcceptors, NextAcceptorsMut};

#[must_use = "DerefAcceptor must be used to forward values through the dereferenced acceptor"]
#[derive(Debug, Clone)]
pub struct DerefAcceptor<Value, Inner, InnerAccepts>
where
    Inner: Deref<Target = InnerAccepts>,
{
    inner: Inner,
    _marker: PhantomData<(Value, InnerAccepts)>,
}
impl<Value, Inner, InnerAccepts> Deref for DerefAcceptor<Value, Inner, InnerAccepts>
where
    Inner: Deref<Target = InnerAccepts>,
{
    type Target = InnerAccepts;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
impl<Value, Inner, InnerAccepts> DerefMut for DerefAcceptor<Value, Inner, InnerAccepts>
where
    Inner: Deref<Target = InnerAccepts> + DerefMut<Target = InnerAccepts>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

impl<Value, Inner, InnerAccepts> Accepts<Value> for DerefAcceptor<Value, Inner, InnerAccepts>
where
    Inner: Deref<Target = InnerAccepts>,
    InnerAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        self.deref().accept(value)
    }
}
impl<Value, Inner, InnerAccepts> AsyncAccepts<Value> for DerefAcceptor<Value, Inner, InnerAccepts>
where
    Inner: Deref<Target = InnerAccepts>,
    InnerAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        self.deref().accept_async(value)
    }
}

#[cfg(feature = "alloc")]
use crate::__internal::alloc::boxed::Box;
#[cfg(feature = "alloc")]
use crate::core_traits::DynAsyncAccepts;
#[cfg(feature = "alloc")]
impl<Value, Inner, InnerAccepts> DynAsyncAccepts<Value>
    for DerefAcceptor<Value, Inner, InnerAccepts>
where
    Inner: Deref<Target = InnerAccepts>,
    InnerAccepts: DynAsyncAccepts<Value>,
{
    fn accept_async_dyn<'a>(
        &'a self,
        value: Value,
    ) -> core::pin::Pin<Box<dyn Future<Output = ()> + 'a>>
    where
        Value: 'a,
    {
        self.deref().accept_async_dyn(value)
    }
}

impl<Value, Inner, InnerAccepts> NextAcceptors for DerefAcceptor<Value, Inner, InnerAccepts>
where
    Inner: Deref<Target = InnerAccepts>,
    for<'a> InnerAccepts: NextAcceptors + 'a,
{
    type Acceptor<'a>
        = InnerAccepts::Acceptor<'a>
    where
        Self: 'a;
    type Iter<'a>
        = InnerAccepts::Iter<'a>
    where
        Self: 'a;
    fn next_acceptors(&self) -> Self::Iter<'_> {
        self.inner.next_acceptors()
    }
}

impl<Value, Inner, InnerAccepts> NextAcceptorsMut for DerefAcceptor<Value, Inner, InnerAccepts>
where
    Inner: Deref<Target = InnerAccepts> + DerefMut<Target = InnerAccepts>,
    for<'a> InnerAccepts: NextAcceptorsMut + 'a,
{
    type Acceptor<'a>
        = InnerAccepts::Acceptor<'a>
    where
        Self: 'a;
    type Iter<'a>
        = InnerAccepts::Iter<'a>
    where
        Self: 'a;
    fn next_acceptors_mut(&mut self) -> Self::Iter<'_> {
        self.inner.next_acceptors_mut()
    }
}
