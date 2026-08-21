//! [`Permission`] trait and function implementations.
use actix_web::*;
use std::future::Future;

/// The interface used for request validation.
pub trait Permission<Args>: Clone + 'static {
    /// Future type that resolves to a `actix_web::Result<bool>`.
    type Future: Future<Output = actix_web::Result<bool>>;

    /// Request validation happens inside `call` function.
    ///
    /// # Properties
    ///
    /// * `req` - http request.
    /// * `args` - arguments.
    fn call(&self, req: HttpRequest, args: Args) -> Self::Future;
}

macro_rules! factory_tuple ({ $($param:ident)* } => {
    impl<Func, Fut, $($param,)*> Permission<($($param,)*)> for Func
    where
        Func: Fn(HttpRequest, $($param),*) -> Fut + Clone + 'static,
        Fut: Future<Output = actix_web::Result<bool>>,
    {
        type Future = Fut;


        #[inline]
        #[allow(non_snake_case)]
        fn call(&self, req: HttpRequest, ($($param,)*): ($($param,)*)) -> Self::Future {
            (self)(req, $($param,)*)
        }
    }
});

factory_tuple! {}
factory_tuple! { A }
factory_tuple! { A B }
factory_tuple! { A B C }
factory_tuple! { A B C D }
factory_tuple! { A B C D E }
factory_tuple! { A B C D E F }
factory_tuple! { A B C D E F G }
factory_tuple! { A B C D E F G H }
factory_tuple! { A B C D E F G H I }
factory_tuple! { A B C D E F G H I J }
factory_tuple! { A B C D E F G H I J K }
factory_tuple! { A B C D E F G H I J K L }
