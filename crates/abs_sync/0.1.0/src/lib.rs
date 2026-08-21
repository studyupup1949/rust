#![no_std]
#![feature(sync_unsafe_cell)]
#![feature(try_trait_v2)]
#![feature(type_alias_impl_trait)]

#[cfg(test)]
extern crate std;

pub mod async_lock;
pub mod cancellation;
pub mod never_cancel;
pub mod ok_or;
pub mod sync_lock;
pub mod sync_tasks;

pub mod preludes {
    pub use super::async_lock::TrAsyncRwLock;
    pub use super::sync_lock::TrSyncRwLock;
    pub use super::sync_tasks::TrSyncTask;
    pub use super::cancellation::{TrCancellationToken, TrIntoFutureMayCancel};
    pub use super::ok_or::XtOkOr;
}

pub mod x_deps {
    pub use atomex;
    pub use pin_utils;
}
