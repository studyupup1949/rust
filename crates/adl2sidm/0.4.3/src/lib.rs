//! # ⚠️ `adl2sidm` has been renamed to [`adl2rsdm`](https://crates.io/crates/adl2rsdm)
//!
//! This crate is **deprecated** and receives no further updates. It re-exports
//! the `adl2rsdm` library API so existing `use adl2sidm::…` code keeps
//! compiling. The command-line tool is now `adl2rsdm`:
//!
//! ```text
//! cargo install adl2rsdm
//! ```
pub use adl2rsdm::*;
