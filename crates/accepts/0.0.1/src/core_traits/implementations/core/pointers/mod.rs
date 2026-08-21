mod macros;

use macros::impl_pointer;

mod accepts;

mod async_accepts;
#[cfg(feature = "alloc")]
mod dyn_async_accepts;
