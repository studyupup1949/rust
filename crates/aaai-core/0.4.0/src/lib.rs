//! # aaai-core v0.4.0
//!
//! Core engine for **aaai** (audit for asset integrity).
//!
//! # Module map
//!
//! ```text
//! aaai-core
//!   ├── config    — AuditDefinition and its YAML I/O
//!   ├── diff      — parallel folder walker, DiffEntry (binary + stats), ignore patterns
//!   ├── audit     — match DiffEntries → AuditResult; large-file warnings
//!   ├── report    — Markdown / JSON report generation (with optional masking)
//!   ├── history   — append-only audit run log (~/.aaai/history.jsonl)
//!   ├── masking   — regex-based secret masking engine
//!   ├── project   — .aaai.yaml project-level config
//!   ├── templates — built-in rule templates
//!   └── profile   — named before/after/definition presets
//! ```

// SPDX-License-Identifier: Apache-2.0

pub mod audit;
pub mod config;
pub mod diff;
pub mod history;
pub mod masking;
pub mod profile;
pub mod project;
pub mod report;
pub mod templates;

pub use audit::engine::AuditEngine;
pub use audit::result::{AuditResult, AuditStatus, AuditSummary, FileAuditResult};
pub use config::definition::{AuditDefinition, AuditEntry, AuditStrategy};
pub use diff::engine::DiffEngine;
pub use diff::entry::{DiffEntry, DiffStats, DiffType, LARGE_FILE_THRESHOLD};
pub use diff::ignore::IgnoreRules;
pub use masking::engine::MaskingEngine;
pub use report::generator::ReportGenerator;
