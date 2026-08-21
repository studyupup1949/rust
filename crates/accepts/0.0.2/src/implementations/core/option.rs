use core::future::Future;

use crate::{Accepts, AsyncAccepts};

impl<Value, A: Accepts<Value>> Accepts<Value> for Option<A> {
    fn accept(&self, value: Value) {
        if let Some(inner) = self {
            inner.accept(value);
        }
    }
}

impl<Value, A: AsyncAccepts<Value>> AsyncAccepts<Value> for Option<A> {
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async move {
            if let Some(inner) = self {
                inner.accept_async(value).await;
            }
        }
    }
}
