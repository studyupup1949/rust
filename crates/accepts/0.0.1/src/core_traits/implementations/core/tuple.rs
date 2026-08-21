use core::future::Future;
#[cfg(feature = "alloc")]
use core::pin::Pin;

#[cfg(feature = "alloc")]
use crate::__internal::alloc::boxed::Box;
#[cfg(feature = "alloc")]
use crate::core_traits::DynAsyncAccepts;
use crate::core_traits::{Accepts, AsyncAccepts};

macro_rules! tuple_accept_calls {
    ($self:expr, $value:ident, $last:tt) => {
        $self.$last.accept($value);
    };
    ($self:expr, $value:ident, $first:tt, $($rest:tt),+) => {
        $self.$first.accept($value.clone());
        tuple_accept_calls!($self, $value, $($rest),+);
    };
}

macro_rules! tuple_async_calls {
    ($method:ident, $self:expr, $value:ident, $last:tt) => {
        $self.$last.$method($value).await;
    };
    ($method:ident, $self:expr, $value:ident, $first:tt, $($rest:tt),+) => {
        $self.$first.$method($value.clone()).await;
        tuple_async_calls!($method, $self, $value, $($rest),+);
    };
}

impl<Value, A: Accepts<Value>> Accepts<Value> for (A,) {
    fn accept(&self, value: Value) {
        self.0.accept(value);
    }
}

impl<Value, A: AsyncAccepts<Value>> AsyncAccepts<Value> for (A,) {
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        self.0.accept_async(value)
    }
}

#[cfg(feature = "alloc")]
impl<Value, A: DynAsyncAccepts<Value>> DynAsyncAccepts<Value> for (A,) {
    fn accept_async_dyn<'a>(&'a self, value: Value) -> Pin<Box<dyn Future<Output = ()> + 'a>>
    where
        Value: 'a,
    {
        self.0.accept_async_dyn(value)
    }
}

macro_rules! tuple_impls {
    ($(($($idx:tt $name:ident),+));+ $(;)?) => {
        $(
            impl<Value, $( $name: Accepts<Value> ),+> Accepts<Value> for ( $( $name, )+ )
            where
                Value: Clone,
            {
                fn accept(&self, value: Value) {
                    tuple_accept_calls!(self, value, $($idx),+);
                }
            }

            impl<Value, $( $name: AsyncAccepts<Value> ),+> AsyncAccepts<Value> for ( $( $name, )+ )
            where
                Value: Clone,
            {
                fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
                where
                    Value: 'a,
                {
                    async move {
                        tuple_async_calls!(accept_async, self, value, $($idx),+);
                    }
                }
            }

            #[cfg(feature = "alloc")]
            impl<Value, $( $name: DynAsyncAccepts<Value> ),+> DynAsyncAccepts<Value> for ( $( $name, )+ )
            where
                Value: Clone,
            {
                fn accept_async_dyn<'a>(&'a self, value: Value) -> Pin<Box<dyn Future<Output = ()> + 'a>>
                where
                    Value: 'a,
                {
                    Box::pin(async move {
                        tuple_async_calls!(accept_async_dyn, self, value, $($idx),+);
                    })
                }
            }
        )+
    };
}

tuple_impls! {
    (0 A, 1 B);
    (0 A, 1 B, 2 C);
    (0 A, 1 B, 2 C, 3 D);
    (0 A, 1 B, 2 C, 3 D, 4 E);
    (0 A, 1 B, 2 C, 3 D, 4 E, 5 F);
    (0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G);
    (0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H);
    (0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I);
    (0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J);
    (0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K);
    (0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L);
}
