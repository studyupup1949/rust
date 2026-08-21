//! Environment variable sources.
//!
//! Implements plugin architecture for creating and managing sources.

pub mod config;
mod traits;
mod variable;
mod registry;

#[cfg(feature = "file")]
mod file;
#[cfg(feature = "file")]
mod file_manager;

#[cfg(feature = "shell")]
mod shell;

mod memory;

pub use config::{SourceRefreshOptions, FileSourceConfig, ShellSourceConfig, RemoteSourceConfig, MemorySourceConfig};
pub use traits::*;
pub use variable::*;
pub use registry::*;

#[cfg(feature = "file")]
pub use file::FileSource;
#[cfg(feature = "file")]
pub use file_manager::FileSourceManager;

#[cfg(feature = "shell")]
pub use shell::ShellSource;

pub use memory::MemorySource;

pub use traits::SourceSnapshot;
