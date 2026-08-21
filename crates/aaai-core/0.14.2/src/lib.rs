//! # aaai-core
//!
//! Core engine for **aaai** (audit for asset integrity).
//!
//! This crate provides all domain logic: folder diffing, audit evaluation,
//! report generation, secret masking, audit history, profiles, and project config.
//! It is consumed by [`aaai-cli`] (the CLI binary) and [`aaai-gui`] (the desktop GUI).
//!
//! ## Module map
//!
//! ```text
//! aaai-core
//!   ├── config      — AuditDefinition YAML, entry fields, lockfile, I/O
//!   ├── diff        — parallel folder diff (rayon), DiffEntry, IgnoreRules (.aaaiignore)
//!   ├── audit       — AuditEngine, AuditResult, AuditStatus, AuditWarning
//!   ├── report      — Markdown / JSON / HTML / SARIF v2.1.0 output
//!   ├── masking     — regex-based secret redaction (9 built-in patterns)
//!   ├── history     — append-only audit run log (~/.aaai/history.jsonl)
//!   ├── profile     — named before/after/definition combos + user prefs (theme)
//!   ├── project     — .aaai.yaml auto-discovery and project-level defaults
//!   └── templates   — 8 built-in rule templates (version_bump, port_change, …)
//! ```
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use aaai_core::{DiffEngine, AuditEngine, AuditDefinition};
//! use std::path::Path;
//!
//! let diffs = DiffEngine::compare(Path::new("./before"), Path::new("./after")).unwrap();
//! let definition = AuditDefinition::new_empty();
//! let result = AuditEngine::evaluate(&diffs, &definition);
//! println!("PASSED: {}", result.summary.is_passing());
//! ```
//!
//! ## Exit code contract (used by aaai-cli)
//!
//! | Code | Meaning |
//! |---|---|
//! | 0 | PASSED — all entries OK or Ignored |
//! | 1 | FAILED — one or more audit failures |
//! | 2 | PENDING — unresolved entries |
//! | 3 | ERROR — file-level errors |
//! | 4 | CONFIG_ERROR — definition parse error |


pub mod audit;
pub mod config;
pub mod diff;
pub mod history;
pub mod masking;
pub mod profile;
pub mod project;
pub mod report;
pub mod templates;

pub use audit::engine::{AuditEngine, AuditOptions};
pub use audit::result::{AuditResult, AuditStatus, AuditSummary, FileAuditResult};
pub use config::definition::{AuditDefinition, AuditEntry, AuditStrategy};
pub use diff::engine::DiffEngine;
pub use diff::entry::{DiffEntry, DiffStats, DiffType, LARGE_FILE_THRESHOLD};
pub use diff::ignore::IgnoreRules;
pub use diff::progress::{DiffProgress, ProgressSink, ChannelProgress, NullProgress};
pub use masking::engine::MaskingEngine;
pub use report::generator::ReportGenerator;
