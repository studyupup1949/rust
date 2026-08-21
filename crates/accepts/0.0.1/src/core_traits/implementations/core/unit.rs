use core::future::{Future, ready};

use crate::core_traits::{Accepts, AsyncAccepts};

impl<Value> Accepts<Value> for () {
    fn accept(&self, _: Value) {}
}

impl<Value> AsyncAccepts<Value> for () {
    fn accept_async<'a>(&'a self, _: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        ready(())
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

    impl<Value> DynAsyncAccepts<Value> for () {
        fn accept_async_dyn<'a>(&'a self, _: Value) -> Pin<Box<dyn Future<Output = ()> + 'a>>
        where
            Value: 'a,
        {
            Box::pin(ready(()))
        }
    }
}
