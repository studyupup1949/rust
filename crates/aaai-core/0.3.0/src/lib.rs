//! # aaai-core
//!
//! Core engine for **aaai** (audit for asset integrity).
//!
//! # Module map
//!
//! ```text
//! aaai-core
//!   ├── config    — AuditDefinition and its YAML I/O
//!   ├── diff      — folder walker, DiffEntry, ignore patterns
//!   ├── audit     — match DiffEntries → AuditResult
//!   ├── report    — Markdown / JSON report generation
//!   ├── history   — append-only audit run log
//!   ├── templates — built-in rule templates
//!   └── profile   — named before/after/definition presets
//! ```

// SPDX-License-Identifier: Apache-2.0

pub mod audit;
pub mod config;
pub mod diff;
pub mod history;
pub mod profile;
pub mod report;
pub mod templates;

pub use audit::engine::AuditEngine;
pub use audit::result::{AuditResult, AuditStatus, AuditSummary, FileAuditResult};
pub use config::definition::{AuditDefinition, AuditEntry, AuditStrategy};
pub use diff::engine::DiffEngine;
pub use diff::entry::{DiffEntry, DiffType};
pub use diff::ignore::IgnoreRules;
pub use report::generator::ReportGenerator;
