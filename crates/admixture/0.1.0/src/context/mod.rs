//! Test context management for integration tests.
//!
//! This module provides the core abstractions for managing test contexts
//! that orchestrate multiple services (databases, caches, APIs, etc.) with
//! proper lifecycle management.
//!
//! # Overview
//!
//! A test context manages the lifecycle of multiple services:
//! 1. **Setup**: Services are configured but not started
//! 2. **Running**: All services are started and healthy
//! 3. **Stopped**: Services have been gracefully shut down
//!
//! # Architecture
//!
//! The context system uses a typestate pattern with user-defined structs:
//!
//! - **Setup struct**: Contains `ServiceSetup` fields (not yet started)
//! - **Running struct**: Contains `ServiceRunning` fields (started and healthy)
//! - **StoppableContext wrapper**: Manages lifecycle and provides cleanup guarantees
//!
//! # Example (Manual - before macros)
//!
//! ```rust,ignore
//! use admixture::context::{TestContext, ContextSetup, ContextRunning};
//! use std::time::Duration;
//!
//! // Define setup and running structs
//! struct MyContextSetup {
//!     postgres: PostgresServiceSetup,
//! }
//!
//! struct MyContextRunning {
//!     postgres: PostgresServiceRunning,
//! }
//!
//! // Implement ContextSetup trait
//! impl ContextSetup for MyContextSetup {
//!     type Running = MyContextRunning;
//!     
//!     async fn start_all(self, config: &ContextConfig) -> eyre::Result<Self::Running> {
//!         let postgres = self.postgres.start().await?;
//!         wait_until_healthy_with_config(&postgres, config).await?;
//!         
//!         Ok(MyContextRunning { postgres })
//!     }
//! }
//!
//! impl ContextRunning for MyContextRunning {
//!     async fn stop_all(&mut self) -> eyre::Result<()> {
//!         self.postgres.stop().await?;
//!         Ok(())
//!     }
//! }
//!
//! // Use in tests
//! #[tokio::test]
//! async fn test_my_feature() -> eyre::Result<()> {
//!     let ctx = TestContext::builder(MyContextSetup { postgres: /* ... */ })
//!         .with_startup_timeout(Duration::from_secs(60))
//!         .build()
//!         .await?;
//!     
//!     // Access services via Deref
//!     let client = ctx.postgres.client().await?;
//!     
//!     // Use client...
//!     
//!     ctx.stop().await?;
//!     Ok(())
//! }
//! ```

pub mod builder;
pub mod config;
pub mod error;
pub mod health;
pub mod setup;

use std::ops::Deref;

pub use builder::ContextBuilder;
pub use config::ContextConfig;
pub use error::ContextError;
pub use health::{wait_until_healthy, wait_until_healthy_with_config};
pub use setup::{ContextRunning, ContextSetup};

use tracing::{error, info};

/// A basic test context wrapper without automatic cleanup.
///
/// This is a generic wrapper that doesn't provide automatic cleanup on drop.
/// For most use cases, prefer `StoppableContext` which implements Drop.
///
/// # Type Parameter
///
/// * `T` - The running context struct (contains `ServiceRunning` fields)
pub struct TestContext<T> {
    inner: T,
}

impl<T> TestContext<T> {
    /// Create a builder for configuring and starting a test context.
    ///
    /// Note: The builder returns a `StoppableContext` which provides
    /// automatic cleanup on drop.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = TestContext::builder(setup)
    ///     .with_startup_timeout(Duration::from_secs(60))
    ///     .build()
    ///     .await?;
    /// ```
    pub fn builder(setup: T) -> ContextBuilder<T> {
        ContextBuilder::new(setup)
    }
}

impl<T> Deref for TestContext<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A test context with automatic cleanup on drop.
///
/// `StoppableContext<T>` wraps a running context and ensures all services
/// are properly stopped when the context is dropped, even during panics.
///
/// # Type Parameter
///
/// * `T` - The running context struct (must implement `ContextRunning`)
///
/// # Cleanup Guarantees
///
/// - **Automatic cleanup**: Services are stopped automatically in Drop using `block_on`
///   - Works correctly within tokio runtimes (the expected use case)
///   - Errors are logged but don't cause panics (best-effort cleanup)
///   - Handles panics gracefully (Drop is called during unwinding)
/// - **Manual stop** (recommended): Call `context.stop().await?` for explicit error handling
///   - Allows you to handle errors explicitly
///   - Consumes the context, preventing Drop from running
///   - Preferred when you want to ensure cleanup succeeded
///
/// # Example
///
/// ```rust,ignore
/// // Automatic cleanup (Drop handles it)
/// {
///     let ctx = TestContext::builder(setup).build().await?;
///     let client = ctx.postgres.client().await?;
///     // ... use client ...
/// }  // <- Drop automatically stops services here
///
/// // Manual cleanup (recommended for explicit error handling)
/// let ctx = TestContext::builder(setup).build().await?;
/// let client = ctx.postgres.client().await?;
/// // ... use client ...
/// ctx.stop().await?;  // Explicit cleanup with error handling
/// ```
///
/// # Implementation Note
///
/// Drop uses `Handle::try_current()` and `block_on()` to run async cleanup.
/// This works correctly in tokio-based tests (the framework's design).
/// If no runtime is available (rare), an error is logged.
pub struct StoppableContext<T: ContextRunning + Send + 'static> {
    inner: Option<T>,
}

impl<T: ContextRunning + Send + 'static> StoppableContext<T> {
    /// Manually stop the test context and all its services.
    ///
    /// This method allows you to stop the context explicitly with error handling.
    /// **This is optional** - Drop will automatically clean up if you don't call this.
    ///
    /// # When to Use
    ///
    /// - When you want explicit error handling during cleanup
    /// - When you want to verify cleanup succeeded in tests
    /// - When cleanup timing is important for your test logic
    ///
    /// If you don't call this, Drop will handle cleanup automatically using `block_on`.
    ///
    /// # Error Handling
    ///
    /// This method uses best-effort cleanup:
    /// - Attempts to stop all services
    /// - Logs errors but continues stopping other services
    /// - Returns `Ok(())` even if some services failed to stop
    ///
    /// This ensures that all cleanup is attempted and resources are released,
    /// even if individual services encounter errors during shutdown.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = TestContext::builder(setup).build().await?;
    /// // ... use context ...
    /// ctx.stop().await?;  // Explicit cleanup with error handling
    /// ```
    pub async fn stop(mut self) -> Result<(), T::Error> {
        info!("Stopping test context");

        // Take the inner value - if it's already None, we've already stopped
        let Some(mut inner) = self.inner.take() else {
            info!("Test context already stopped");
            return Ok(());
        };

        match inner.stop_all().await {
            Ok(()) => {
                info!("Test context stopped successfully");
                Ok(())
            }
            Err(e) => {
                // Log the error but still return Ok (best-effort cleanup)
                error!(error = %e, "Error during context shutdown (continuing with best-effort cleanup)");
                Ok(())
            }
        }
    }
}

impl<T: ContextRunning + Send + 'static> Deref for StoppableContext<T> {
    type Target = T;

    /// Transparently access the inner running context.
    ///
    /// This allows you to access services directly:
    /// ```rust,ignore
    /// let client = ctx.postgres.client().await?;
    /// //           ^^^^^^^^^^^^^ - Direct field access via Deref
    /// ```
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect(
            "StoppableContext inner value was taken. \
             This is a bug - the context should not be used after .stop() is called."
        )
    }
}

impl<T: ContextRunning + Send + 'static> Drop for StoppableContext<T> {
    /// Automatically cleanup resources when dropped.
    ///
    /// This implementation spawns an async task to stop all services when the
    /// context is dropped. This provides best-effort cleanup even if `.stop().await`
    /// was not called explicitly.
    ///
    /// # Implementation Details
    ///
    /// - Uses `Option::take()` to move the inner context out of `&mut self`
    /// - Spawns a detached tokio task to run async `stop_all()`
    /// - If no tokio runtime is available, logs an error (rare case)
    ///
    /// # Caveats
    ///
    /// - Cleanup runs asynchronously after Drop returns (non-blocking)
    /// - You cannot observe or handle cleanup errors in Drop
    /// - Resources may not be immediately released
    ///
    /// For these reasons, calling `.stop().await` explicitly is still **strongly
    /// recommended** when you need guarantees about:
    /// - Cleanup completion (synchronous/blocking)
    /// - Error handling during cleanup
    /// - Resource release timing
    fn drop(&mut self) {
        // Take the inner value - if it's None, stop() was already called
        let Some(mut inner) = self.inner.take() else {
            // Context was already explicitly stopped - no cleanup needed
            return;
        };

        // Try to spawn cleanup task in the tokio runtime
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // Spawn a detached task to run the async cleanup
                handle.spawn(async move {
                    match inner.stop_all().await {
                        Ok(()) => {
                            info!("Test context stopped successfully in Drop");
                        }
                        Err(e) => {
                            error!(
                                error = %e,
                                "Error during context cleanup in Drop (best-effort cleanup)"
                            );
                        }
                    }
                });
                info!("Test context cleanup task spawned in Drop");
            }
            Err(_) => {
                // No tokio runtime - can't spawn cleanup task
                error!(
                    "Cannot clean up StoppableContext: no tokio runtime available. \
                     Resources may leak. Ensure StoppableContext is used within a tokio runtime \
                     and call .stop().await explicitly for guaranteed cleanup."
                );
            }
        }
    }
}
