use core::future::Future;

use crate::{Accepts, AsyncAccepts};

impl<Value, A: Accepts<Value>, E> Accepts<Value> for Result<A, E> {
    fn accept(&self, value: Value) {
        if let Ok(inner) = self {
            inner.accept(value);
        }
    }
}

impl<Value, A: AsyncAccepts<Value>, E> AsyncAccepts<Value> for Result<A, E> {
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async move {
            if let Ok(inner) = self {
                inner.accept_async(value).await;
            }
        }
    }
}
