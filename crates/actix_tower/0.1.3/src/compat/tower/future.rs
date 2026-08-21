//! Future types for the Tower bridge.
//!
//! # Zero-Allocation Design
//!
//! Prior to this revision, both future types were `Pin<Box<dyn Future>>`:
//! - One heap allocation per request crossing the bridge (≈ 40–80 ns).
//! - One vtable lookup per `poll()` call, preventing LLVM inlining.
//!
//! Both types are now **concrete enum state machines** (see [`future_impl`])
//! that live entirely on the caller's stack frame. The compiler monomorphises
//! a separate, fully-inlined copy for each distinct inner future type,
//! so the poll chain flattens to a single codegen path with no indirect calls.
//!
//! # Public Re-exports
//!
//! The concrete types are re-exported here to preserve the original public
//! import path (`crate::compat::tower::future::*`).

pub use super::future_impl::{TowerMiddlewareFutureImpl, ActixServiceWrapperFutureImpl};

/// Convenience re-export for the primary outbound future.
///
/// This is the future returned by [`TowerMiddlewareService::call`](super::service::TowerMiddlewareService).
/// Parameterised over `F` — the concrete `TS::Future` type — so LLVM can
/// monomorphise and inline across the entire async call chain.
pub use super::future_impl::TowerMiddlewareFutureImpl as TowerMiddlewareFuture;

/// Convenience re-export for the primary inbound future.
///
/// This is the future returned by [`ActixServiceWrapper::call`](super::service::ActixServiceWrapper).
/// Parameterised over `CollectFut` and `CallFut`; both are concrete types
/// visible to the compiler for full optimisation.
pub use super::future_impl::ActixServiceWrapperFutureImpl as ActixServiceWrapperFuture;
