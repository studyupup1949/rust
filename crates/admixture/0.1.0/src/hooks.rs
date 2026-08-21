//! Lifecycle hooks for test contexts.
//!
//! Hooks allow running setup/teardown logic at specific points in the test lifecycle.

use std::{future::Future, pin::Pin};

/// Type alias for hook functions with Higher-Rank Trait Bounds.
///
/// Hooks receive a reference to the running context of concrete type C
/// and return a pinned future.
pub type HookFn<C> = for<'a> fn(
    &'a C,
) -> Pin<
    Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send>>> + Send + 'a>,
>;

/// Container for lifecycle hooks with concrete context type.
///
/// All hooks are optional. Hooks are executed at specific points in the test lifecycle:
/// - `before_all`: After context starts, before any tests run
/// - `after_all`: After all tests complete, before context stops (best-effort)
/// - `before_each`: Before each individual test
/// - `after_each`: After each individual test (always runs, even if test fails)
pub struct Hooks<C> {
    pub before_all: Option<HookFn<C>>,
    pub after_all: Option<HookFn<C>>,
    pub before_each: Option<HookFn<C>>,
    pub after_each: Option<HookFn<C>>,
    pub _phantom: std::marker::PhantomData<fn(&C)>,
}

// Manual implementation because derive would require C: Copy, but we only store fn pointers
impl<C> Clone for Hooks<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> Copy for Hooks<C> {}
