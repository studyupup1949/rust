#![cfg_attr(not(test), no_std)]
#![allow(clippy::manual_async_fn, clippy::module_inception)]
#![doc = "alloc-powered acceptors built on `accepts`"]

extern crate alloc;

mod btree_kv_router;
pub use btree_kv_router::*;

#[cfg(test)]
pub(crate) mod support;
