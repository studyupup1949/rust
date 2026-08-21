#![doc = include_str!("../README.md")]

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "async-trait")]
mod task_actor;

#[cfg(feature = "async-trait")]
pub use task_actor::*;

mod task_actor_macros;

pub use task_actor_macros::*;

mod auto_detach_task;

pub use auto_detach_task::*;

#[cfg(feature = "thread_pool")]
mod thread_pool;

#[cfg(feature = "thread_pool")]
pub use thread_pool::*;

#[cfg(all(test, feature = "thread_pool"))]
mod task_actor_macro_tests;
