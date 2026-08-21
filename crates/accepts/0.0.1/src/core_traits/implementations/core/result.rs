use core::future::Future;

use crate::core_traits::{Accepts, AsyncAccepts};

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

#[cfg(feature = "alloc")]
mod alloc {
    use core::{
        future::{Future, ready},
        pin::Pin,
    };

    use crate::__internal::alloc::boxed::Box;
    use crate::core_traits::DynAsyncAccepts;

    impl<Value, A: DynAsyncAccepts<Value>, E> DynAsyncAccepts<Value> for Result<A, E> {
        fn accept_async_dyn<'a>(&'a self, value: Value) -> Pin<Box<dyn Future<Output = ()> + 'a>>
        where
            Value: 'a,
        {
            if let Ok(inner) = self {
                inner.accept_async_dyn(value)
            } else {
                Box::pin(ready(()))
            }
        }
    }
}
