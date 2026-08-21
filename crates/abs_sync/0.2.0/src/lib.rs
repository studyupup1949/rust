#![no_std]

// to enable no hand-written poll
#![feature(async_fn_traits)]
#![feature(impl_trait_in_assoc_type)]
#![feature(unboxed_closures)]

#![feature(sync_unsafe_cell)]
#![feature(try_trait_v2)]
#![feature(type_alias_impl_trait)]

#[cfg(test)]
extern crate std;

pub mod async_lock;
pub mod async_mutex;
pub mod cancellation;
pub mod never_cancel;
pub mod ok_or;
pub mod sync_lock;
pub mod sync_mutex;
pub mod sync_tasks;

pub mod preludes {
    pub use super::async_lock::TrAsyncRwLock;
    pub use super::async_mutex::TrAsyncMutex;
    pub use super::cancellation::{TrCancellationToken, TrMayCancel};
    pub use super::ok_or::{OkOr, XtOkOr};
    pub use super::sync_lock::TrSyncRwLock;
    pub use super::sync_mutex::TrSyncMutex;
    pub use super::sync_tasks::TrSyncTask;
}
