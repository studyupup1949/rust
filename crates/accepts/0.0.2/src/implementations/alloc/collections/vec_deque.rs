use alloc::collections::VecDeque;
use core::future::Future;

use crate::{Accepts, AsyncAccepts};

impl<Value: Clone, A: Accepts<Value>> Accepts<Value> for VecDeque<A> {
    fn accept(&self, value: Value) {
        if let Some((last, rest)) = self.as_slices().1.split_last() {
            for a in rest {
                a.accept(value.clone());
            }
            last.accept(value);
        } else if let Some((last, rest)) = self.as_slices().0.split_last() {
            for a in rest {
                a.accept(value.clone());
            }
            last.accept(value);
        }
    }
}

impl<Value: Clone, A: AsyncAccepts<Value>> AsyncAccepts<Value> for VecDeque<A> {
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async {
            if let Some((last, rest)) = self.as_slices().1.split_last() {
                for a in rest {
                    a.accept_async(value.clone()).await;
                }
                last.accept_async(value).await;
            } else if let Some((last, rest)) = self.as_slices().0.split_last() {
                for a in rest {
                    a.accept_async(value.clone()).await;
                }
                last.accept_async(value).await;
            }
        }
    }
}
