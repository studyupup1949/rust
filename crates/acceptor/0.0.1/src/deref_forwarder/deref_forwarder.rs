use core::{
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use accepts::{Accepts, AsyncAccepts};

#[must_use = "DerefForwarder must be used to forward values through the dereferenced acceptor"]
#[derive(Debug, Clone)]
pub struct DerefForwarder<Value, Inner, InnerAccepts> {
    inner: Inner,
    _marker: PhantomData<(Value, InnerAccepts)>,
}

impl<Value, Inner, InnerAccepts> DerefForwarder<Value, Inner, InnerAccepts>
where
    Inner: Deref<Target = InnerAccepts>,
{
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}
impl<Value, Inner, InnerAccepts> Deref for DerefForwarder<Value, Inner, InnerAccepts>
where
    Inner: Deref<Target = InnerAccepts>,
{
    type Target = InnerAccepts;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
impl<Value, Inner, InnerAccepts> DerefMut for DerefForwarder<Value, Inner, InnerAccepts>
where
    Inner: Deref<Target = InnerAccepts> + DerefMut<Target = InnerAccepts>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

impl<Value, Inner, InnerAccepts> Accepts<Value> for DerefForwarder<Value, Inner, InnerAccepts>
where
    Inner: Deref<Target = InnerAccepts>,
    InnerAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        self.deref().accept(value)
    }
}
impl<Value, Inner, InnerAccepts> AsyncAccepts<Value> for DerefForwarder<Value, Inner, InnerAccepts>
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
