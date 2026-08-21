use core::{future::Future, pin::Pin};

use crate::__internal::alloc::boxed::Box;
use crate::core_traits::{Accepts, AsyncAccepts, DynAsyncAccepts};

impl<Value: Clone, A: Accepts<Value>> Accepts<Value> for Vec<A> {
    fn accept(&self, value: Value) {
        if let Some((last, rest)) = self.split_last() {
            for a in rest {
                a.accept(value.clone());
            }
            last.accept(value);
        }
    }
}

impl<Value: Clone, A: AsyncAccepts<Value>> AsyncAccepts<Value> for Vec<A> {
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async {
            if let Some((last, rest)) = self.split_last() {
                for a in rest {
                    a.accept_async(value.clone()).await;
                }
                last.accept_async(value).await;
            }
        }
    }
}

impl<Value: Clone, A: DynAsyncAccepts<Value>> DynAsyncAccepts<Value> for Vec<A> {
    fn accept_async_dyn<'a>(&'a self, value: Value) -> Pin<Box<dyn Future<Output = ()> + 'a>>
    where
        Value: 'a,
    {
        Box::pin(async {
            if let Some((last, rest)) = self.split_last() {
                for a in rest {
                    a.accept_async_dyn(value.clone()).await;
                }
                last.accept_async_dyn(value).await;
            }
        })
    }
}
