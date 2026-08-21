use std::fmt;
use std::future::Future;

/// Async runtime bridge for connecting async TDengine client with sync ADBC traits.
///
/// TDengine's `taos-client` is async, but ADBC traits are synchronous. This enum
/// wraps a Tokio runtime, detecting existing runtime via `Handle::try_current()`
/// or creating new multi-threaded runtime when needed.
pub enum Runtime {
    Handle(tokio::runtime::Handle),
    TokioRuntime(tokio::runtime::Runtime),
}

impl Runtime {
    /// Creates a new runtime instance.
    ///
    /// Reuses existing Tokio runtime if available, otherwise creates a new
    /// multi-threaded runtime with all features enabled.
    pub fn new() -> std::io::Result<Self> {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            Ok(Self::Handle(handle))
        } else {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;

            Ok(Self::TokioRuntime(rt))
        }
    }

    /// Blocks on a future, bridging async calls to sync context.
    pub fn block_on<F: Future>(&self, fut: F) -> F::Output {
        match self {
            Runtime::Handle(handle) => tokio::task::block_in_place(|| handle.block_on(fut)),
            Runtime::TokioRuntime(runtime) => runtime.block_on(fut),
        }
    }
}

impl Clone for Runtime {
    fn clone(&self) -> Self {
        match self {
            Runtime::Handle(handle) => Runtime::Handle(handle.clone()),
            Runtime::TokioRuntime(_) => {
                // Cloning a TokioRuntime handle requires getting the current runtime
                match tokio::runtime::Handle::try_current() {
                    Ok(handle) => Runtime::Handle(handle),
                    Err(_) => panic!("Cannot clone TokioRuntime without active runtime"),
                }
            }
        }
    }
}

impl fmt::Debug for Runtime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Runtime::Handle(_) => f.write_str("Runtime::Handle(...)"),
            Runtime::TokioRuntime(_) => f.write_str("Runtime::TokioRuntime(...)"),
        }
    }
}

/// Blocks on a future using the provided runtime.
///
/// This helper function is used for running async operations in a sync context,
/// particularly useful when working with connection pools.
pub fn block_on_on_runtime<F: std::future::Future>(
    fut: F,
    rt: &Runtime,
) -> F::Output {
    rt.block_on(fut)
}

// Rust guideline compliant 2025-01-02