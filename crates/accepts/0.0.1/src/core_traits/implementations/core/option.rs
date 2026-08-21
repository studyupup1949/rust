use core::future::Future;

use crate::core_traits::{Accepts, AsyncAccepts};

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

#[cfg(feature = "alloc")]
mod alloc {
    use core::future::{Future, ready};

    use crate::__internal::alloc::boxed::Box;
    use crate::core_traits::DynAsyncAccepts;

    impl<Value, A: DynAsyncAccepts<Value>> DynAsyncAccepts<Value> for Option<A> {
        fn accept_async_dyn<'a>(
            &'a self,
            value: Value,
        ) -> core::pin::Pin<Box<dyn Future<Output = ()> + 'a>>
        where
            Value: 'a,
        {
            if let Some(inner) = self {
                inner.accept_async_dyn(value)
            } else {
                Box::pin(ready(()))
            }
        }
    }
}
