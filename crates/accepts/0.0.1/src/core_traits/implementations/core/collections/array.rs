use core::future::Future;

use crate::core_traits::{Accepts, AsyncAccepts};

impl<Value: Clone, A: Accepts<Value>, const N: usize> Accepts<Value> for [A; N] {
    fn accept(&self, value: Value) {
        if let Some((last, rest)) = self.split_last() {
            for a in rest {
                a.accept(value.clone());
            }
            last.accept(value);
        }
    }
}

impl<Value: Clone, A: AsyncAccepts<Value>, const N: usize> AsyncAccepts<Value> for [A; N] {
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

#[cfg(feature = "alloc")]
mod alloc {
    use core::{future::Future, pin::Pin};

    use crate::__internal::alloc::boxed::Box;
    use crate::core_traits::DynAsyncAccepts;

    impl<Value: Clone, A: DynAsyncAccepts<Value>, const N: usize> DynAsyncAccepts<Value> for [A; N] {
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
}
