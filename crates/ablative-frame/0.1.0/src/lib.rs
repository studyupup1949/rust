//! Frame application framework.
//!
//! Re-exports the five Frame crates as a single dependency:
//!
//! - [`core`] — component model, lifecycle, WASM host
//! - [`state`] — content-addressed storage, entities, branching
//! - [`conv`] — conversation patterns over liminal
//! - [`view`] — view fragment composition
//! - [`mcp`] — MCP tool surface for agent participants

pub use frame_conv as conv;
pub use frame_core as core;
pub use frame_mcp as mcp;
pub use frame_state as state;
pub use frame_view as view;
