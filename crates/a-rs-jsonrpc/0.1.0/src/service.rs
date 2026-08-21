//! # JSON-RPC Server Module
//!
//! This module provides the infrastructure for building a JSON-RPC server. It utilizes
//! a distributed registration system that allows methods to be defined anywhere
//! in the codebase and automatically collected into a global routing table.
//!
//! ## Key Components
//!
//! - **`linkme` Integration**: Uses distributed slices to collect RPC methods at compile-time.
//! - **Automatic Dispatch**: Routes incoming JSON-RPC requests to the correct handler based on the `method` field.
//! - **Async Handlers**: Supports `async` functions as RPC services via `BoxFuture`.
//!
//! ## Workflow
//! 1. Define a function and annotate it with `#[jsonrpc_service_fn_...]`.
//! 2. The macro registers the function into the [`RPC_SERVICES`] slice.
//! 3. Call [`init()`] at application startup to build the [`ROUTE_TABLE`].
//! 4. Use [`dispatch()`] to process raw request bytes.

use crate::RpcError;
pub use a_rs_jsonrpc_macros::{jsonrpc_service_fn_array, jsonrpc_service_fn_obj};
use futures::future::BoxFuture;
use linkme::distributed_slice;
use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock};

/// A trait for types that can handle JSON-RPC requests.
///
/// This trait is primarily used by the procedural macros to wrap user-defined
/// functions into a standardized interface.
#[async_trait::async_trait]
pub trait JsonRpcServiceFn {
    /// The return type of the RPC method, which must be serializable.
    type Result;

    /// Processes a raw byte request and returns a structured JSON-RPC response.
    async fn handle(
        req: &[u8],
    ) -> Result<crate::response::JsonRpcResponse<Self::Result>, crate::error::RpcError>
    where
        Self::Result: serde::Serialize;
}

/// Internal envelope used to peek at the `method` field of a JSON-RPC request
/// without deserializing the entire payload.
#[derive(Deserialize)]
struct MethodEnvelope<'a> {
    #[serde(borrow)]
    method: &'a str,
}

/// A registration entry for an RPC method.
///
/// Each entry maps a unique method name (used in the JSON `method` field)
/// to a type-erased handler function.
#[derive(Debug)]
pub struct RpcServiceEntry {
    /// The string name of the RPC method.
    pub method: &'static str,
    /// The handler function pointer that returns a boxed future.
    pub handler: RpcHandlerFn,
}

/// A distributed slice containing all registered RPC services.
///
/// This slice is populated at compile-time by the `#[jsonrpc_service_fn_...]` macros
/// using the `linkme` crate.

#[distributed_slice]
pub static RPC_SERVICES: [RpcServiceEntry];

/// A type alias for the internal handler function signature.
///
/// It takes raw request bytes and returns a [`BoxFuture`] resolving to a
/// JSON-serialized response string.
pub type RpcHandlerFn =
    fn(req: &[u8]) -> BoxFuture<'static, Result<String, crate::error::RpcError>>;

/// A global, lazily-initialized routing table.
///
/// On first access, it collects all entries from [`RPC_SERVICES`] into a [`HashMap`].
/// It will panic if duplicate method names are detected.
static ROUTE_TABLE: LazyLock<HashMap<&'static str, RpcHandlerFn>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for entry in RPC_SERVICES {
        if m.insert(entry.method, entry.handler).is_some() {
            panic!("Duplicate method registered: {}", entry.method);
        }
    }
    m
});

/// Dispatches a raw JSON-RPC request to the appropriate registered handler.
///
/// This is the main entry point for integrating the library with a web server
/// (e.g., Axum or Actix). It extracts the method name and executes the mapped future.
///
/// # Errors
/// Returns [`RpcError::MethodNotFound`] if the method name is not in the routing table.
pub async fn dispatch(body: &[u8]) -> Result<String, RpcError> {
    let MethodEnvelope { method } = serde_json::from_slice(body)?;
    if let Some(handler) = ROUTE_TABLE.get(method) {
        return handler(body).await;
    }
    Err(RpcError::MethodNotFound)
}

/// Initializes the RPC service and logs all registered methods.
///
/// It is recommended to call this during application startup to ensure the
/// [`ROUTE_TABLE`] is valid and to verify registered services.
pub fn init() {
    tracing::info!("RPC Service initialized with {} methods", ROUTE_TABLE.len());
    for method in ROUTE_TABLE.keys() {
        tracing::info!("  - {}", method);
    }
}
