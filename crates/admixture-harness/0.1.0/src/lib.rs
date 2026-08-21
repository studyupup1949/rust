//! Test harness for integration tests with admixture contexts.
//!
//! Provides the `#[admixture_test]` macro for writing integration tests
//! with automatic context lifecycle management and lifecycle hooks.
//!
//! # Example
//!
//! ```ignore
//! use admixture::context;
//! use admixture_harness::prelude::*;
//!
//! context! {
//!     MyTestContext {
//!         postgres: SqlxPostgresServiceSetup,
//!     }
//! }
//!
//! #[admixture_test(context = MyTestContext)]
//! async fn test_database_operations(ctx: &MyTestContextRunning) -> Result<(), TestError> {
//!     let client = ctx.postgres().client().await?;
//!     // Test logic here
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod runner;

use std::future::Future;
use std::pin::Pin;

// Re-export the macro
pub use admixture_harness_macros::admixture_test;

// Re-export runner components for external use
pub use runner::{ContextGroupResult, TestFailure, TestResults, collect_tests, run_all_tests_impl, run_context_group};

/// Generic context lifecycle manager trait.
///
/// Implemented by the macro for each context type to manage setup/teardown.
/// Uses concrete types instead of type erasure for better type safety.
#[allow(clippy::type_complexity)]
pub trait ContextManager<C>: Send + Sync {
    /// Start the context and return the running context.
    fn start(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<C, Box<dyn std::error::Error + Send>>> + Send>>;

    /// Stop the context.
    fn stop(
        &self,
        ctx: C,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send>>> + Send>>;
}

/// Type alias for hook functions with Higher-Rank Trait Bounds.
///
/// Hooks receive a reference to the running context of concrete type C
/// and return a pinned future.
pub use admixture::hooks::HookFn;

/// Container for lifecycle hooks with concrete context type.
///
/// All hooks are optional. Hooks are executed at specific points in the test lifecycle:
/// - `before_all`: After context starts, before any tests run
/// - `after_all`: After all tests complete, before context stops (best-effort)
/// - `before_each`: Before each individual test
/// - `after_each`: After each individual test (always runs, even if test fails)
pub use admixture::hooks::Hooks;

/// Type alias for test functions with Higher-Rank Trait Bounds.
///
/// Test functions receive a reference to the running context of concrete type C
/// and return a pinned future with the test result.
pub type TestFn<C> = for<'a> fn(
    &'a C,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send>>> + Send + 'a>>;

/// Type alias for group runner functions.
///
/// This is the type-erased boundary between test collection (inventory) and
/// typed test execution. Each context type gets its own runner implementation.
pub type GroupRunnerFn = fn(
    &'static [&'static TestDescriptor],
) -> Pin<Box<dyn Future<Output = ContextGroupResult> + Send>>;

/// Descriptor for a registered integration test.
///
/// This struct is submitted to the `inventory` collection by the `#[admixture_test]` macro.
#[derive(Copy, Clone)]
pub struct TestDescriptor {
    /// Name of the test function.
    pub name: &'static str,

    /// Module path where the test is defined.
    pub module_path: &'static str,

    /// Source file where the test is defined.
    pub file: &'static str,

    /// Line number where the test is defined.
    pub line: u32,

    /// Context type identifier (for grouping tests).
    pub context_type: &'static str,

    /// Type-erased entry point to run all tests for this context type.
    ///
    /// This function collects test functions from the per-context registry
    /// and dispatches to the generic runner with concrete types.
    pub run_group: GroupRunnerFn,
}

// Collect all registered integration tests.
inventory::collect!(TestDescriptor);

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::admixture_test;
    pub use crate::error::TestError;

    /// Initialize tracing with indicatif support for pretty progress bars.
    ///
    /// Call this once at the beginning of your test suite to enable
    /// visual progress bars and structured logging.
    ///
    /// It's safe to call this multiple times - it will only initialize once.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use admixture_harness::prelude::*;
    ///
    /// #[cfg(test)]
    /// fn init_test_tracing() {
    ///     init_tracing();
    /// }
    /// ```
    pub fn init_tracing() {
        use std::sync::Once;
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            // Just use a simple fmt subscriber for now
            // Tests can override this with tracing-indicatif if they want
            let _ = tracing_subscriber::fmt()
                .with_target(false)
                .with_test_writer()
                .try_init();
        });
    }
}
