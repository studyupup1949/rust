use core::future::Future;

use crate::core_traits::AsyncAccepts;

super::impl_pointer!(AsyncAccepts::accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a where Value: 'a);
