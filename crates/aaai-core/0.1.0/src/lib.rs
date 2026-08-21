//! # aaai-core
//!
//! Core engine for **aaai** (audit for asset integrity).
//!
//! This crate is GUI- and CLI-independent. It owns all business logic:
//! folder diffing, audit definition I/O, audit judgement, and report
//! generation.  Both `aaai-cli` and `aaai-gui` depend on this crate and
//! share the same judgement results — the spec's CLI/GUI consistency
//! requirement is satisfied structurally.
//!
//! # Module map
//!
//! ```text
//! aaai-core
//!   ├── config   — AuditDefinition and its YAML I/O
//!   ├── diff     — folder walker and DiffEntry production
//!   ├── audit    — match DiffEntries against AuditDefinition → AuditResult
//!   └── report   — Markdown / JSON report generation
//! ```

// SPDX-License-Identifier: Apache-2.0

pub mod audit;
pub mod config;
pub mod diff;
pub mod report;

// Convenience re-exports so downstream crates can write
// `use aaai_core::{AuditEngine, DiffEngine, …}` without navigating
// the module tree.
pub use audit::engine::AuditEngine;
pub use audit::result::{AuditResult, AuditStatus, FileAuditResult};
pub use config::definition::{AuditDefinition, AuditEntry, AuditStrategy};
pub use diff::engine::DiffEngine;
pub use diff::entry::{DiffEntry, DiffType};
pub use report::generator::ReportGenerator;
