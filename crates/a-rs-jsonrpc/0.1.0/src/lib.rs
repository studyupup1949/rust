//! # A-RS-JSONRPC
//!
//! `a-rs-jsonrpc` is a battery-included, high-performance JSON-RPC library for Rust.
//! It provides a seamless experience for both Server and Client implementations,
//! supporting JSON-RPC 1.0 and 2.0 specifications with a heavy focus on developer
//! productivity through procedural macros.
//!
//! ## Core Features
//!
//! - **Full-Duplex Capability**: Comprehensive tools for building both JSON-RPC servers and clients.
//! - **Protocol Flexibility**: Full support for both JSON-RPC 1.0 and 2.0 versions.
//! - **Runtime Agnostic**: Built on `async/await` and `reqwest`, compatible with various async runtimes like Tokio.
//! - **Macro-Driven**: Drastically reduce boilerplate using `#[rpc_method]` and `#[jsonrpc_service_fn_...]`.
//!
//! ---
//!
//! ## Quick Start
//!
//! ### 1. Server-Side Service Definition
//!
//! Define RPC methods easily by choosing between **Array-based** (positional) or **Object-based** (named) parameter modes.
//!
//! ```rust
//! use a_rs_jsonrpc::{RpcError, jsonrpc_service_fn_array, jsonrpc_service_fn_obj};
//!
//! /// Array Mode: Expects params as a list, e.g., `{"params": [10, 20]}`
//! #[jsonrpc_service_fn_array(method = "addArray", version = "v2")]
//! async fn add_array(a: i32, b: i32) -> Result<i32, RpcError> {
//!     Ok(a + b)
//! }
//!
//! /// Object Mode: Expects params as a named map, e.g., `{"params": {"lhs": 10, "rhs": 20}}`
//! #[jsonrpc_service_fn_obj(method = "addObj", version = "v2")]
//! async fn add_obj(lhs: i32, rhs: i32) -> Result<i32, RpcError> {
//!     Ok(lhs + rhs)
//! }
//! ```
//!
//! ### 2. Client-Side Usage
//!
//! The library offers high-level traits for direct calls and attribute macros for defining typed interfaces.
//!
//! #### Defining an Interface with `rpc_method`
//! ```rust
//! use a_rs_jsonrpc::macros::rpc_method;
//!
//! #[rpc_method(
//!     url = "http://localhost:3000/",
//!     method = "addArray",
//!     version = "v2",
//!     mode = "array"
//! )]
//! async fn add(a: i32, b: i32) -> Result<JsonRpcResponse<i32>, RpcError> {}
//! ```
//!
//! #### Using Trait Extensions
//! ```rust
//! // Call using a tuple (serialized as a JSON array)
//! let resp = (10, 20).send_v2_request(URL, APP_JSON, "addArray").await?;
//!
//! // Call using a struct (serialized as a JSON object)
//! #[derive(Serialize)]
//! struct MyParams { lhs: i32, rhs: i32 }
//! let resp = MyParams { lhs: 10, rhs: 20 }.send_v2_request_obj(URL, APP_JSON, "addObj").await?;
//! ```
//!
//! ---
//!
//! ## Parameter Modes Comparison
//!
//!
//!
//! | Mode | Attribute | Parameter Format | Best Use Case |
//! | :--- | :--- | :--- | :--- |
//! | **Array** | `mode = "array"` | `[val1, val2]` | Simple APIs or legacy 1.0 compatibility. |
//! | **Object** | `mode = "obj"` | `{"key": val}` | Complex APIs where parameter names add clarity. |
//!
//! ---
//!
//! ## Error Handling
//!
//! All network, serialization, and protocol errors are unified into the [`RpcError`] enum.
//! Server-side business logic errors are returned via the [`response::JsonRpcError`] structure
//! within the response.
//!
//! ## Debugging and Tracing
//!
//! The crate integrates with the `tracing` ecosystem. To inspect raw JSON-RPC requests and responses,
//! initialize a subscriber in your entry point:
//! ```rust
//! tracing_subscriber::fmt()
//!     .with_max_level(tracing::Level::DEBUG)
//!     .init();
//! ```
//!

pub mod client;
pub mod error;
pub mod id;
pub mod request;
pub mod response;
pub mod service;

pub use a_rs_jsonrpc_macros::rpc_method;
pub use async_trait;
pub use client::JsonRpcClient;
pub use client::JsonRpcClientCall;
pub use error::RpcError;
pub use id::Id as JsonRpcId;
pub use linkme;
pub use request::JsonRpcRequest;
pub use response::JsonRpcResponse;
pub use serde;
pub use serde_json;
pub use service::JsonRpcServiceFn;
pub use service::RPC_SERVICES;
pub use service::RpcServiceEntry;
pub use service::dispatch as dispatch_rpc_request;
pub use service::init as init_rpc_service;
pub use service::jsonrpc_service_fn_array;
pub use service::jsonrpc_service_fn_obj;
