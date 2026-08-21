use core::{future::Future, pin::Pin};

use crate::core_traits::DynAsyncAccepts;

super::impl_pointer!(DynAsyncAccepts::accept_async_dyn<'a>(&'a self, value: Value) -> Pin<crate::__internal::alloc::boxed::Box<dyn Future<Output = ()> + 'a>> where Value: 'a);
