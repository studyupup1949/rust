use core::future::Future;

use crate::{Accepts, AsyncAccepts};

impl<Value: Clone, A: Accepts<Value>> Accepts<Value> for [A] {
    fn accept(&self, value: Value) {
        if let Some((last, rest)) = self.split_last() {
            for a in rest {
                a.accept(value.clone());
            }
            last.accept(value);
        }
    }
}

impl<Value: Clone, A: AsyncAccepts<Value>> AsyncAccepts<Value> for [A] {
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
