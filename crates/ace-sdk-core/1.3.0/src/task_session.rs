//! F-080 TaskSession helper — append-only per-pin anchor store.
//!
//! Provides a concurrency-safe per-task anchor that ties a pattern search to
//! a later learn-trace so the server can credit reward correctly.
//!
//! # Storage layout (per-pin)
//! Each `pin_search` call writes its OWN file:
//! `~/.ace-cache/sessions/<safe_org>__<safe_project>/<session_id>__<pin_uuid>.json`
//!
//! `anchor_trace` globs `<session_id>__*.json`, unions all retrieval_log_ids,
//! picks the earliest pin's retrieval_id, stamps the trace, then reaps all
//! matched pin files.
//!
//! # Anchor shape (byte-identical across all 5 language SDKs)
//! ```json
//! {
//!   "session_id": "<uuid4>",
//!   "org_id": "<string>",
//!   "project_id": "<string>",
//!   "retrieval_id": "<string|null>",
//!   "retrieval_log_ids": [<i64>, ...],
//!   "created_at_ms": <i64>,
//!   "expires_at_ms": <i64>
//! }
//! ```
//!
//! # Invariants
//! 1. Fresh per-TASK uuid4 `session_id` (not the IDE/conversation session).
//! 2. `pin_search` + `anchor_trace` share byte-identical ids.
//! 3. `pin_search` persists early and unconditionally (one NEW file per call).
//! 4. Abandoned anchors expire via GC — SDK never auto-credits.
//! 5. `anchor_trace` is idempotent: a second call globs an empty set → only
//!    stamps session_id, no error.
//!
//! # Contract boundary (multi-process plugin model)
//!
//! The plugin consumer is MULTI-PROCESS: the pattern search runs in a
//! `SubagentStart` hook (process A) and the anchor/learn in a `SubagentStop`
//! hook (process B) — separate OS processes, no shared memory, B may start
//! minutes later. An in-memory handle from `begin_task_session()` cannot
//! survive A→B. Process B only has `(org, project, trace.session_id)`.
//!
//! Use [`load_task_session`] in process B to bind to the existing anchor by
//! `session_id`. The module-level [`anchor_trace`] is a one-liner convenience
//! that does the same.
//!
//! Domain-shift mid-task re-pins use `loadTaskSession(org,proj,sessionId).pinSearch(…)`
//! which appends a NEW per-pin file → accumulation works with no special-casing.
//!
//! ## Important caveats
//!
//! - **No Stop→session_id correlation**: this helper assumes
//!   `trace.session_id` is already correct. It does NOT solve fan-out or
//!   degenerate stop events — that stays the consumer's responsibility.
//! - **Reap is keyed by session_id prefix**: process B reaps every
//!   `<session_id>__*.json` that process A (and any domain-shift re-pins) wrote.
//! - **Double-anchor is idempotent**: if `anchor_trace` is called twice, the
//!   second call globs an empty set and is a no-op (session_id still stamped).
//! - **Long-runner caveat**: if a task runs past the TTL (default 24 h), the
//!   anchor files may be GC'd before `anchor_trace` is called.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::types::{ExecutionTrace, SearchResponse15};

/// Monotonic counter that ensures each `new_uuid4()` call produces a distinct
/// seed even when two calls land in the same nanosecond tick on the same thread.
static UUID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Default anchor TTL: 24 hours in milliseconds.
///
/// Override per-call via [`TaskSessionOptions::ttl_ms`].
pub const DEFAULT_ANCHOR_TTL_MS: i64 = 86_400_000;

/// Sanitise an org_id or project_id for safe filesystem embedding.
///
/// Replaces filesystem-unsafe characters with `_`.  The character set matches
/// the Go, TypeScript, and Kotlin SDKs exactly so that directory names are
/// byte-identical across all language implementations:
///
/// | char | reason |
/// |------|--------|
/// | `/`  | Unix path separator |
/// | `\`  | Windows path separator |
/// | `:`  | Windows drive letter / NTFS stream separator |
/// | `*`  | glob wildcard |
/// | `?`  | glob wildcard |
/// | `"`  | Windows forbidden, shell quoting hazard |
/// | `<`  | shell redirection / Windows forbidden |
/// | `>`  | shell redirection / Windows forbidden |
/// | `\|` | shell pipe / Windows forbidden |
/// | `\0` | null byte — always unsafe |
///
/// The two identifiers are joined with a DOUBLE underscore `__` by the caller
/// (same convention as the `.db` files).
fn sanitise(s: &str) -> String {
    s.chars()
        .map(|c| {
            if matches!(
                c,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0'
            ) {
                '_'
            } else {
                c
            }
        })
        .collect()
}

/// Anchor persisted as a per-pin JSON file.
///
/// Each `pin_search` call writes ONE instance of this; `anchor_trace` unions
/// all instances for the same session_id.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskAnchor {
    pub session_id: String,
    pub org_id: String,
    pub project_id: String,
    pub retrieval_id: Option<String>,
    pub retrieval_log_ids: Vec<i64>,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
}

/// Options for [`begin_task_session`] and [`load_task_session`].
#[derive(Debug, Clone, Default)]
pub struct TaskSessionOptions {
    /// Override the sessions directory (used in tests to avoid writing to `~`).
    pub sessions_dir: Option<PathBuf>,

    /// Caller-provided session_id.
    ///
    /// When set, `begin_task_session` uses this value verbatim instead of
    /// generating a fresh uuid4. The same id is embedded in the anchor file so
    /// process B can locate the anchor using only `(org, project, session_id)`.
    ///
    /// Typically derived from the IDE/CC subagent identity token that is stable
    /// across the Start→Stop hook pair. Leave `None` to keep the default
    /// behaviour (fresh uuid4 per task).
    pub session_id: Option<String>,

    /// Anchor TTL override in milliseconds.
    ///
    /// When `None`, the default [`DEFAULT_ANCHOR_TTL_MS`] (24 h = 86_400_000 ms)
    /// is used. For tasks known to run longer than 24 h, pass a larger value to
    /// prevent the anchor from being GC'd before `anchor_trace` is called.
    pub ttl_ms: Option<i64>,
}

/// Per-task F-080 session anchor helper.
///
/// Construct with [`begin_task_session`] (process A / new task) or
/// [`load_task_session`] (process B / re-entry by existing `session_id`).
/// One `TaskSession` per sub-agent / task — never reuse across tasks.
pub struct TaskSession {
    /// Session identifier — either a fresh uuid4 (default) or the
    /// caller-provided value from [`TaskSessionOptions::session_id`].
    pub session_id: String,
    org_id: String,
    project_id: String,
    project_dir: PathBuf,
    /// Effective TTL in ms used when writing the anchor's `expires_at_ms`.
    ttl_ms: i64,
}

/// Construct a new `TaskSession` for a fresh task (process A entry point).
///
/// If [`TaskSessionOptions::session_id`] is set, that value is used verbatim
/// as the `session_id`; otherwise a fresh uuid4 is generated.
///
/// Runs GC (sweeps expired per-pin anchor files) before returning.
pub fn begin_task_session(
    org_id: &str,
    project_id: &str,
    opts: Option<TaskSessionOptions>,
) -> TaskSession {
    let opts = opts.unwrap_or_default();
    let session_id = opts.session_id.unwrap_or_else(new_uuid4);
    let ttl_ms = opts.ttl_ms.unwrap_or(DEFAULT_ANCHOR_TTL_MS);

    let sessions_root = opts.sessions_dir.unwrap_or_else(default_sessions_dir);
    let project_dir = sessions_root.join(format!("{}__{}", sanitise(org_id), sanitise(project_id)));

    // Best-effort mkdir — ignore errors (GC will also skip gracefully)
    std::fs::create_dir_all(&project_dir).ok();

    let ts = TaskSession {
        session_id,
        org_id: org_id.to_string(),
        project_id: project_id.to_string(),
        project_dir,
        ttl_ms,
    };

    // GC: sweep expired per-pin anchors — best-effort, ignore errors
    ts.gc();

    ts
}

/// Bind to an existing task session by `session_id` (process B entry point).
///
/// Constructs a `TaskSession` bound to the given `session_id` without
/// generating a new uuid. The per-pin anchor files need not exist at the time
/// of construction — binding is by id only.
///
/// This is the entry point for the `SubagentStop` hook process (process B),
/// which receives `(org, project, trace.session_id)` from the stop event but
/// has no in-memory handle to the session created in process A.
///
/// Domain-shift re-pins also use this: call `pin_search` on the returned
/// `TaskSession` to append a NEW per-pin file; accumulation happens at
/// `anchor_trace` time.
///
/// Runs GC before returning, exactly as `begin_task_session` does.
pub fn load_task_session(
    org_id: &str,
    project_id: &str,
    session_id: &str,
    opts: Option<TaskSessionOptions>,
) -> TaskSession {
    let opts = opts.unwrap_or_default();
    let ttl_ms = opts.ttl_ms.unwrap_or(DEFAULT_ANCHOR_TTL_MS);

    let sessions_root = opts.sessions_dir.unwrap_or_else(default_sessions_dir);
    let project_dir = sessions_root.join(format!("{}__{}", sanitise(org_id), sanitise(project_id)));

    // Best-effort mkdir
    std::fs::create_dir_all(&project_dir).ok();

    let ts = TaskSession {
        session_id: session_id.to_string(),
        org_id: org_id.to_string(),
        project_id: project_id.to_string(),
        project_dir,
        ttl_ms,
    };

    // GC: sweep expired per-pin anchors — best-effort, ignore errors
    ts.gc();

    ts
}

/// The F-080 view returned by [`read_f080`] (non-reaping).
///
/// Contains the same fields that [`anchor_trace`] would stamp onto a trace,
/// but computed without consuming / reaping any pin files.
///
/// * `retrieval_id`   – The earliest surviving pin's non-null `retrieval_id`,
///   or `None` when no surviving pins carry one.
/// * `applied_log_ids` – Union of all surviving pins' `retrieval_log_ids`,
///   deduplicated and sorted. Empty when no pins are present.
#[derive(Debug, Clone, PartialEq)]
pub struct F080View {
    pub retrieval_id: Option<String>,
    pub applied_log_ids: Vec<i64>,
}

/// Stateless module-level convenience: compute the F-080 view for the given
/// `(org_id, project_id, session_id)` **without reaping** pin files.
///
/// Returns `None` when `session_id` is empty; otherwise loads the session and
/// delegates to [`TaskSession::read_f080`].
///
/// # Arguments
/// * `org_id`     – Organization identifier.
/// * `project_id` – Project identifier.
/// * `session_id` – Task session identifier.
/// * `opts`       – Optional [`TaskSessionOptions`] (TTL override, custom dir, …).
pub fn read_f080(
    org_id: &str,
    project_id: &str,
    session_id: &str,
    opts: Option<TaskSessionOptions>,
) -> F080View {
    let ts = load_task_session(org_id, project_id, session_id, opts);
    ts.read_f080()
}

/// Stateless module-level convenience: derive the session from
/// `trace.session_id`, load that session, stamp + reap, and return the trace.
///
/// Equivalent to:
/// ```ignore
/// load_task_session(org_id, project_id, &trace.session_id.unwrap())
///     .anchor_trace(trace)
/// ```
///
/// If `trace.session_id` is `None` the trace is returned unchanged (no-op).
///
/// # Arguments
/// * `org_id`     – Organization identifier.
/// * `project_id` – Project identifier.
/// * `trace`      – Execution trace to stamp. Consumed and returned mutated.
/// * `opts`       – Optional [`TaskSessionOptions`] (TTL override, custom dir, …).
pub fn anchor_trace(
    org_id: &str,
    project_id: &str,
    trace: crate::types::ExecutionTrace,
    opts: Option<TaskSessionOptions>,
) -> crate::types::ExecutionTrace {
    let sid = match &trace.session_id {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return trace, // no session_id → no-op
    };

    let ts = load_task_session(org_id, project_id, &sid, opts);
    ts.anchor_trace(trace)
}

impl TaskSession {
    /// Extract `retrieval_id` + per-pattern `retrieval_log_id` values from a
    /// search response and atomically write a NEW per-pin anchor file.
    ///
    /// Each call generates a fresh `pin_uuid` and writes
    /// `<session_id>__<pin_uuid>.json` — never reads or modifies existing files.
    /// Persist EARLY/unconditionally — even if `retrieval_id` is absent.
    pub fn pin_search(&self, search_result: &SearchResponse15) {
        let retrieval_id = search_result.retrieval_id.clone();

        // Collect de-duped retrieval_log_ids from each pattern's match_factors
        let mut log_ids: Vec<i64> = Vec::new();
        for pattern in &search_result.similar_patterns {
            if let Some(mf) = &pattern.match_factors {
                if let Some(log_id) = mf.retrieval_log_id {
                    if !log_ids.contains(&log_id) {
                        log_ids.push(log_id);
                    }
                }
            }
        }

        let now_ms = chrono::Utc::now().timestamp_millis();
        let anchor = TaskAnchor {
            session_id: self.session_id.clone(),
            org_id: self.org_id.clone(),
            project_id: self.project_id.clone(),
            retrieval_id,
            retrieval_log_ids: log_ids,
            created_at_ms: now_ms,
            expires_at_ms: now_ms + self.ttl_ms,
        };

        // Generate a fresh pin_uuid for the filename discriminator
        let pin_uuid = new_uuid4();
        self.write_pin_atomic(&anchor, &pin_uuid);
    }

    /// Glob all `<session_id>__*.json` pin files, union their retrieval_log_ids,
    /// pick the earliest pin's retrieval_id, stamp the trace, then reap all
    /// matched files (best-effort).
    ///
    /// Always sets `trace.session_id` to `self.session_id`.  If surviving pins
    /// exist, also sets `trace.retrieval_id` and `trace.applied_log_ids`
    /// when they are not already provided.  Returns the mutated trace.
    pub fn anchor_trace(&self, mut trace: ExecutionTrace) -> ExecutionTrace {
        // Always stamp the task session_id
        trace.session_id = Some(self.session_id.clone());

        let now_ms = chrono::Utc::now().timestamp_millis();

        // Glob all per-pin files for this session
        let pin_files = self.glob_pin_files();

        // Read and filter surviving (non-expired) pins.
        // Carry the filename alongside the anchor for deterministic tie-breaking.
        let mut pins: Vec<(String, TaskAnchor)> = Vec::new();
        let mut to_reap: Vec<PathBuf> = Vec::new();

        for path in &pin_files {
            to_reap.push(path.clone());
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(anchor) = serde_json::from_str::<TaskAnchor>(&data) {
                    if anchor.expires_at_ms >= now_ms {
                        pins.push((filename, anchor));
                    }
                    // expired pins are still added to to_reap above
                }
            }
        }

        if !pins.is_empty() {
            // retrieval_id: pick the pin with the smallest (created_at_ms, filename)
            // composite key — deterministic even when two pins share the same
            // millisecond timestamp (equal timestamps resolve by filename order).
            let earliest = pins
                .iter()
                .filter(|(_, p)| p.retrieval_id.is_some())
                .min_by_key(|(fname, p)| (p.created_at_ms, fname.as_str()));

            if trace.retrieval_id.is_none() {
                trace.retrieval_id = earliest.and_then(|(_, p)| p.retrieval_id.clone());
            }

            // Union all retrieval_log_ids (deduplicated, sorted for determinism).
            if trace.applied_log_ids.is_none() {
                let mut union_ids: Vec<i64> = Vec::new();
                for (_, pin) in &pins {
                    for &id in &pin.retrieval_log_ids {
                        if !union_ids.contains(&id) {
                            union_ids.push(id);
                        }
                    }
                }
                if !union_ids.is_empty() {
                    union_ids.sort_unstable();
                    trace.applied_log_ids = Some(union_ids);
                }
            }
        }

        // REAP: delete all globbed per-pin files (best-effort, ignore ENOENT)
        for path in &to_reap {
            std::fs::remove_file(path).ok();
        }

        trace
    }

    /// Non-reaping F-080 peek: compute the same view [`anchor_trace`] would
    /// stamp, but **without deleting pin files**.
    ///
    /// Steps:
    /// 1. Glob `<session_id>__*.json` (prefix-scoped, same as `anchor_trace`).
    /// 2. Skip expired pins (`expires_at_ms < now`); best-effort delete expired
    ///    only (mirrors the expired-pin handling in `anchor_trace`).
    /// 3. Union all surviving `retrieval_log_ids` (dedup, sorted ascending).
    /// 4. `retrieval_id` = earliest surviving pin's non-null `retrieval_id`,
    ///    breaking ties by `(created_at_ms, filename)` — identical tie-break
    ///    to `anchor_trace`.
    /// 5. **Never delete live pin files.**
    ///
    /// Returns [`F080View`] with empty `applied_log_ids` and `None`
    /// `retrieval_id` when no surviving pins exist.
    pub fn read_f080(&self) -> F080View {
        let now_ms = chrono::Utc::now().timestamp_millis();

        let pin_files = self.glob_pin_files();

        let mut pins: Vec<(String, TaskAnchor)> = Vec::new();

        for path in &pin_files {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(anchor) = serde_json::from_str::<TaskAnchor>(&data) {
                    if anchor.expires_at_ms >= now_ms {
                        pins.push((filename, anchor));
                    } else {
                        // Best-effort delete expired pin (same as anchor_trace)
                        std::fs::remove_file(path).ok();
                    }
                }
            }
        }

        if pins.is_empty() {
            return F080View {
                retrieval_id: None,
                applied_log_ids: vec![],
            };
        }

        // retrieval_id: earliest surviving pin (same tie-break as anchor_trace)
        let retrieval_id = pins
            .iter()
            .filter(|(_, p)| p.retrieval_id.is_some())
            .min_by_key(|(fname, p)| (p.created_at_ms, fname.as_str()))
            .and_then(|(_, p)| p.retrieval_id.clone());

        // Union all retrieval_log_ids (dedup, sorted)
        let mut union_ids: Vec<i64> = Vec::new();
        for (_, pin) in &pins {
            for &id in &pin.retrieval_log_ids {
                if !union_ids.contains(&id) {
                    union_ids.push(id);
                }
            }
        }
        union_ids.sort_unstable();

        F080View {
            retrieval_id,
            applied_log_ids: union_ids,
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Glob all per-pin files for this session: `<session_id>__*.json`.
    ///
    /// PREFIX-SCOPED — only scans `<org>__<project>/` and filters by the
    /// `<session_id>__` prefix in the filename. Does not scan the whole
    /// sessions root.
    fn glob_pin_files(&self) -> Vec<PathBuf> {
        let prefix = format!("{}__", self.session_id);
        let entries = match std::fs::read_dir(&self.project_dir) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        entries
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                let name = path.file_name()?.to_str()?.to_string();
                // Must start with `<session_id>__` and end with `.json`
                if name.starts_with(&prefix) && name.ends_with(".json") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Atomic write of a per-pin anchor: write to `.tmp` then rename.
    ///
    /// File name: `<session_id>__<pin_uuid>.json`
    /// Calls `create_dir_all` immediately before the write so the directory is
    /// always present even if the best-effort `create_dir_all` in
    /// `begin_task_session` failed silently.
    fn write_pin_atomic(&self, anchor: &TaskAnchor, pin_uuid: &str) {
        let filename = format!("{}__{}.json", self.session_id, pin_uuid);
        let final_path = self.project_dir.join(&filename);
        let tmp_path = self
            .project_dir
            .join(format!("{}__{}.json.tmp", self.session_id, pin_uuid));

        let json = match serde_json::to_string(anchor) {
            Ok(j) => j,
            Err(_) => return,
        };

        // Best-effort mkdir — consistent with `begin_task_session` pattern
        std::fs::create_dir_all(&self.project_dir).ok();

        // Best-effort: ignore any I/O errors
        if std::fs::write(&tmp_path, &json).is_ok() {
            std::fs::rename(&tmp_path, &final_path).ok();
        }
    }

    /// GC: sweep the project dir, delete per-pin files whose `expires_at_ms < now`.
    /// Best-effort — any error (missing dir, permission, concurrent delete) is ignored.
    fn gc(&self) {
        let now_ms = chrono::Utc::now().timestamp_millis();
        let entries = match std::fs::read_dir(&self.project_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            // Only process `.json` files (skip `.json.tmp`)
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(anchor) = serde_json::from_str::<TaskAnchor>(&data) {
                    if anchor.expires_at_ms < now_ms {
                        std::fs::remove_file(&path).ok();
                    }
                }
            }
        }
    }
}

/// Generate a fresh UUID v4 string without adding a `uuid` or `getrandom`
/// dependency.
///
/// # Uniqueness guarantee
///
/// A process-global `AtomicU64` counter (`UUID_COUNTER`) is mixed into every
/// call.  Even when two calls land in the exact same nanosecond on the same
/// thread (coarse-clock systems, rapid unit-test loops, etc.) the counter
/// increments monotonically, so the 128-bit state differs and collisions are
/// structurally impossible within a single process.
///
/// The mix is: `nanos ⊕ rotate(pid, 17) ⊕ rotate(tid_hash, 13) ⊕ rotate(counter, 7)`.
/// UUID v4 version/variant bits are applied after mixing.
fn new_uuid4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    // --- entropy sources ---
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    let pid = std::process::id() as u64;

    let thread_id = std::thread::current().id();
    let tid_hash = format!("{:?}", thread_id)
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));

    // Monotonically increasing counter — guarantees uniqueness within the process
    // even when all other sources are identical (coarse clock, same thread, same PID).
    let seq = UUID_COUNTER.fetch_add(1, Ordering::Relaxed);

    // --- mixing ---
    let a = nanos
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let b = pid
        .wrapping_mul(2862933555777941757)
        .wrapping_add(3037000493)
        ^ seq.rotate_left(7);
    let c = tid_hash.wrapping_mul(1103515245).wrapping_add(12345) ^ seq.rotate_right(11);

    // Build 128 bits
    let hi: u64 = a ^ b.rotate_left(17);
    let lo: u64 = b ^ c.rotate_right(13);

    // Format as UUID v4: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    // Set version bits (4) and variant bits (10xx)
    let hi_a = (hi >> 32) as u32;
    let hi_b = ((hi >> 16) & 0xffff) as u16;
    let hi_c = (0x4000u16) | ((hi & 0x0fff) as u16); // version 4
    let lo_a = (0x8000u16) | (((lo >> 48) & 0x3fff) as u16); // variant 10xx
    let lo_b = lo & 0xffffffffffff;

    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        hi_a, hi_b, hi_c, lo_a, lo_b
    )
}

fn default_sessions_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ace-cache")
        .join("sessions")
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ExecutionResult, MatchFactors, Pattern, SearchResponse15};
    use tempfile::TempDir;

    fn tmp_opts(dir: &TempDir) -> TaskSessionOptions {
        TaskSessionOptions {
            sessions_dir: Some(dir.path().to_path_buf()),
            ..Default::default()
        }
    }

    fn make_pattern(id: &str, log_id: Option<i64>, retrieval_id: Option<&str>) -> Pattern {
        Pattern {
            id: id.to_string(),
            name: String::new(),
            domain: None,
            content: "test content".to_string(),
            confidence: 0.8,
            observations: 0.0,
            helpful: 0.0,
            harmful: 0.0,
            section: "strategies_and_hard_rules".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: None,
            last_used: None,
            evidence: vec![],
            retrieval_count: 0,
            root_cause: String::new(),
            error_context: String::new(),
            source: None,
            source_project_id: None,
            source_project_name: None,
            local_helpful: 0.0,
            local_harmful: 0.0,
            payload_version: None,
            n_hot_pos: None,
            n_hot_neg: None,
            n_warm_pos: None,
            n_warm_neg: None,
            n_cold_pos: None,
            n_cold_neg: None,
            cumulative_v15_reward: None,
            n_retrieval_no_apply: None,
            task_intent: None,
            effectiveness: None,
            match_factors: Some(MatchFactors {
                retrieval_log_id: log_id,
                retrieval_id: retrieval_id.map(|s| s.to_string()),
                ..Default::default()
            }),
            root_cause_present: None,
            has_error_context: None,
            birth_primary_lang: None,
            domain_cluster_id: None,
            abstract_domain: None,
            root_cause_cluster_id: None,
            birth_first_tool_bucket: None,
            birth_n_steps_bucket: None,
            birth_has_error: None,
            last_citation_score: None,
            citation_score_ema_30d: None,
            merge_winner_count: None,
            merged_from: vec![],
        }
    }

    fn make_trace() -> ExecutionTrace {
        ExecutionTrace {
            task: "test task".to_string(),
            trajectory: vec![],
            result: ExecutionResult {
                success: true,
                output: "done".to_string(),
                error: None,
                summary: None,
            },
            playbook_used: vec![],
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            git: None,
            session_id: None,
            agent_id: None,
            agent_type: None,
            parent_agent_id: None,
            retrieval_id: None,
            applied_log_ids: None,
        }
    }

    /// Helper: list per-pin files for a session in the given project dir.
    fn list_pin_files(project_dir: &PathBuf, session_id: &str) -> Vec<PathBuf> {
        let prefix = format!("{}__", session_id);
        std::fs::read_dir(project_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_str()?.to_string();
                if name.starts_with(&prefix) && name.ends_with(".json") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }

    // -------------------------------------------------------------------------
    // begin_task_session: fresh uuid4, GC runs, dir created
    // -------------------------------------------------------------------------

    #[test]
    fn begin_task_session_generates_unique_session_ids() {
        // Generate 1 000 IDs in the same thread back-to-back (stress test for
        // same-nanosecond-tick collisions). The atomic counter in `new_uuid4`
        // guarantees each call produces a distinct value even when the system
        // clock does not advance between calls.
        let tmp = TempDir::new().unwrap();
        let mut ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for _ in 0..1_000 {
            let ts = begin_task_session("org1", "proj1", Some(tmp_opts(&tmp)));
            let inserted = ids.insert(ts.session_id.clone());
            assert!(
                inserted,
                "duplicate session_id generated: {}",
                ts.session_id
            );
        }
        assert_eq!(ids.len(), 1_000, "all 1 000 session_ids must be unique");
    }

    #[test]
    fn begin_task_session_creates_project_dir() {
        let tmp = TempDir::new().unwrap();
        begin_task_session("my-org", "my-proj", Some(tmp_opts(&tmp)));
        let project_dir = tmp.path().join("my-org__my-proj");
        assert!(project_dir.exists(), "project subdir must be created");
    }

    #[test]
    fn begin_task_session_sanitises_path_separators() {
        let tmp = TempDir::new().unwrap();
        begin_task_session("org/slash", "proj\\back", Some(tmp_opts(&tmp)));
        // Slashes and backslashes should be replaced with '_'
        let project_dir = tmp.path().join("org_slash__proj_back");
        assert!(project_dir.exists(), "sanitised dir must exist");
    }

    #[test]
    fn sanitise_matches_go_ts_kotlin_character_set() {
        // Verify that all 9 unsafe characters (matching Go/TS/Kotlin) are replaced.
        // Input contains: / \ : * ? " < > | plus a safe char and NUL.
        let input = "a/b\\c:d*e?f\"g<h>i|j\0k";
        let expected = "a_b_c_d_e_f_g_h_i_j_k";
        assert_eq!(
            sanitise(input),
            expected,
            "all 9+NUL unsafe chars must become '_'"
        );

        // Safe alphanumeric + hyphen/dot must pass through unchanged.
        assert_eq!(sanitise("my-org.v2"), "my-org.v2");
        assert_eq!(sanitise("proj_123"), "proj_123");
    }

    #[test]
    fn pin_search_creates_dir_if_missing() {
        // Verify that write_pin_atomic calls create_dir_all so the anchor
        // is persisted even when begin_task_session's mkdir succeeded but the
        // directory was later removed (or never existed because of a race).
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session("org", "proj", Some(tmp_opts(&tmp)));

        // Remove the project dir after begin_task_session created it.
        let project_dir = tmp.path().join("org__proj");
        std::fs::remove_dir_all(&project_dir).unwrap();
        assert!(!project_dir.exists(), "pre-condition: dir removed");

        let search = SearchResponse15 {
            similar_patterns: vec![],
            retrieval_id: Some("rid-recreate".to_string()),
            count: 0,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);

        // At least one per-pin file must exist under the project dir
        let pins = list_pin_files(&project_dir, &ts.session_id);
        assert!(
            !pins.is_empty(),
            "pin file must be written even when project_dir was removed after begin_task_session"
        );
    }

    // -------------------------------------------------------------------------
    // pin_search: per-pin file written, fields correct
    // -------------------------------------------------------------------------

    #[test]
    fn pin_search_writes_per_pin_file() {
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session("org", "proj", Some(tmp_opts(&tmp)));
        let search = SearchResponse15 {
            similar_patterns: vec![],
            retrieval_id: Some("rid-abc".to_string()),
            count: 0,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);

        let project_dir = tmp.path().join("org__proj");
        let pins = list_pin_files(&project_dir, &ts.session_id);
        assert_eq!(pins.len(), 1, "exactly one per-pin file must be written");
        // Filename must follow <session_id>__<pin_uuid>.json pattern
        let name = pins[0].file_name().unwrap().to_str().unwrap();
        assert!(name.starts_with(&format!("{}__", ts.session_id)));
        assert!(name.ends_with(".json"));
    }

    #[test]
    fn pin_search_two_calls_write_two_distinct_files() {
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session("org", "proj", Some(tmp_opts(&tmp)));
        let search = SearchResponse15 {
            similar_patterns: vec![],
            retrieval_id: Some("rid".to_string()),
            count: 0,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);
        ts.pin_search(&search);

        let project_dir = tmp.path().join("org__proj");
        let pins = list_pin_files(&project_dir, &ts.session_id);
        assert_eq!(
            pins.len(),
            2,
            "two pin_search calls must write two distinct files"
        );
    }

    #[test]
    fn pin_search_stores_retrieval_id_in_pin_file() {
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session("org", "proj", Some(tmp_opts(&tmp)));
        let search = SearchResponse15 {
            similar_patterns: vec![],
            retrieval_id: Some("rid-xyz".to_string()),
            count: 0,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);

        let project_dir = tmp.path().join("org__proj");
        let pins = list_pin_files(&project_dir, &ts.session_id);
        assert_eq!(pins.len(), 1);
        let data = std::fs::read_to_string(&pins[0]).unwrap();
        let anchor: TaskAnchor = serde_json::from_str(&data).unwrap();
        assert_eq!(anchor.retrieval_id, Some("rid-xyz".to_string()));
        assert_eq!(anchor.session_id, ts.session_id);
    }

    #[test]
    fn pin_search_collects_deduped_retrieval_log_ids() {
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session("org", "proj", Some(tmp_opts(&tmp)));
        let search = SearchResponse15 {
            similar_patterns: vec![
                make_pattern("p1", Some(101), Some("rid-1")),
                make_pattern("p2", Some(102), Some("rid-1")),
                make_pattern("p3", Some(101), Some("rid-1")), // duplicate log_id
                make_pattern("p4", None, None),               // no log_id
            ],
            retrieval_id: Some("rid-1".to_string()),
            count: 4,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);

        let project_dir = tmp.path().join("org__proj");
        let pins = list_pin_files(&project_dir, &ts.session_id);
        assert_eq!(pins.len(), 1);
        let data = std::fs::read_to_string(&pins[0]).unwrap();
        let anchor: TaskAnchor = serde_json::from_str(&data).unwrap();
        let mut ids = anchor.retrieval_log_ids.clone();
        ids.sort();
        assert_eq!(
            ids,
            vec![101, 102],
            "duplicates must be dropped within a single pin"
        );
    }

    #[test]
    fn pin_search_persists_even_when_retrieval_id_absent() {
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session("org", "proj", Some(tmp_opts(&tmp)));
        let search = SearchResponse15 {
            similar_patterns: vec![],
            retrieval_id: None, // absent
            count: 0,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);

        // Per-pin file must still exist even with no retrieval_id
        let project_dir = tmp.path().join("org__proj");
        let pins = list_pin_files(&project_dir, &ts.session_id);
        assert!(
            !pins.is_empty(),
            "pin file must be written even without retrieval_id"
        );
    }

    // -------------------------------------------------------------------------
    // anchor_trace: session_id always set; F-080 fields merged from union
    // -------------------------------------------------------------------------

    #[test]
    fn anchor_trace_always_sets_session_id() {
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session("org", "proj", Some(tmp_opts(&tmp)));
        let trace = make_trace();
        let out = ts.anchor_trace(trace);
        assert_eq!(out.session_id, Some(ts.session_id.clone()));
    }

    #[test]
    fn anchor_trace_merges_f080_fields_from_anchor() {
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session("org", "proj", Some(tmp_opts(&tmp)));

        // Pin a search result first
        let search = SearchResponse15 {
            similar_patterns: vec![
                make_pattern("p1", Some(10), Some("rid-A")),
                make_pattern("p2", Some(20), Some("rid-A")),
            ],
            retrieval_id: Some("rid-A".to_string()),
            count: 2,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);

        let trace = make_trace();
        let out = ts.anchor_trace(trace);

        assert_eq!(out.session_id, Some(ts.session_id.clone()));
        assert_eq!(out.retrieval_id, Some("rid-A".to_string()));
        let mut ids = out.applied_log_ids.unwrap();
        ids.sort();
        assert_eq!(ids, vec![10, 20]);
    }

    #[test]
    fn anchor_trace_does_not_overwrite_explicit_retrieval_id() {
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session("org", "proj", Some(tmp_opts(&tmp)));

        let search = SearchResponse15 {
            similar_patterns: vec![make_pattern("p1", Some(10), Some("rid-from-search"))],
            retrieval_id: Some("rid-from-search".to_string()),
            count: 1,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);

        let mut trace = make_trace();
        trace.retrieval_id = Some("explicit-override".to_string());
        let out = ts.anchor_trace(trace);

        // Explicit retrieval_id must NOT be overwritten
        assert_eq!(out.retrieval_id, Some("explicit-override".to_string()));
    }

    #[test]
    fn anchor_trace_reaps_all_pin_files() {
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session("org", "proj", Some(tmp_opts(&tmp)));

        let search = SearchResponse15 {
            similar_patterns: vec![],
            retrieval_id: Some("rid".to_string()),
            count: 0,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        // Write two pin files
        ts.pin_search(&search);
        ts.pin_search(&search);

        let project_dir = tmp.path().join("org__proj");
        let pins_before = list_pin_files(&project_dir, &ts.session_id);
        assert_eq!(
            pins_before.len(),
            2,
            "two pin files must exist before anchor_trace"
        );

        let trace = make_trace();
        ts.anchor_trace(trace);

        let pins_after = list_pin_files(&project_dir, &ts.session_id);
        assert!(
            pins_after.is_empty(),
            "all pin files must be reaped after anchor_trace"
        );
    }

    #[test]
    fn anchor_trace_no_anchor_only_sets_session_id() {
        // When there's no pinned search, only session_id is set; no F-080 fields
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session("org", "proj", Some(tmp_opts(&tmp)));
        // No pin_search called
        let trace = make_trace();
        let out = ts.anchor_trace(trace);

        assert_eq!(out.session_id, Some(ts.session_id.clone()));
        assert!(out.retrieval_id.is_none());
        assert!(out.applied_log_ids.is_none());
    }

    // -------------------------------------------------------------------------
    // Accumulation tests (AccumTest phase)
    // -------------------------------------------------------------------------

    #[test]
    fn accum_two_pins_union_log_ids_first_retrieval_id_wins() {
        // Main pin: log_ids [42, 99], retrieval_id "rid-main" (earlier created_at)
        // Second pin via loadTaskSession: log_ids [77], retrieval_id "rid-second"
        // Expected: union [42, 77, 99] (sorted), retrieval_id = "rid-main" (earliest)
        let tmp = TempDir::new().unwrap();
        let session_id = "accum-test-session-001";

        // Pin 1 (main / process A)
        let ts_a = begin_task_session(
            "org",
            "proj",
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                session_id: Some(session_id.to_string()),
                ..Default::default()
            }),
        );
        let search_main = SearchResponse15 {
            similar_patterns: vec![
                make_pattern("p1", Some(42), Some("rid-main")),
                make_pattern("p2", Some(99), Some("rid-main")),
            ],
            retrieval_id: Some("rid-main".to_string()),
            count: 2,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts_a.pin_search(&search_main);

        // Sleep 2 ms so the second pin gets a strictly later created_at_ms,
        // making "main wins" unambiguous even without the filename tie-break.
        std::thread::sleep(std::time::Duration::from_millis(2));

        // Pin 2 (domain-shift / process B re-pin via loadTaskSession)
        let ts_b = load_task_session(
            "org",
            "proj",
            session_id,
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );
        let search_domain = SearchResponse15 {
            similar_patterns: vec![make_pattern("p3", Some(77), Some("rid-second"))],
            retrieval_id: Some("rid-second".to_string()),
            count: 1,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts_b.pin_search(&search_domain);

        // Two pin files must exist
        let project_dir = tmp.path().join("org__proj");
        let pins = list_pin_files(&project_dir, session_id);
        assert_eq!(
            pins.len(),
            2,
            "two pin files must exist (one per pin_search call)"
        );

        // anchor_trace: union
        let ts_stop = load_task_session(
            "org",
            "proj",
            session_id,
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );
        let out = ts_stop.anchor_trace(make_trace());

        assert_eq!(out.session_id, Some(session_id.to_string()));
        // retrieval_id: earliest (main) wins
        assert_eq!(out.retrieval_id, Some("rid-main".to_string()));
        // log_ids: union of [42,99] and [77]
        let mut ids = out.applied_log_ids.expect("must have applied_log_ids");
        ids.sort();
        assert_eq!(ids, vec![42, 77, 99], "union of all pin log_ids");

        // All pin files reaped
        let pins_after = list_pin_files(&project_dir, session_id);
        assert!(
            pins_after.is_empty(),
            "all pin files must be reaped after anchor_trace"
        );
    }

    #[test]
    fn accum_three_pins_union_deduplicates() {
        // Pin 1: [42, 99]
        // Pin 2: [77]
        // Pin 3: [42, 55]  — 42 is a duplicate across pins
        // Expected union: [42, 55, 77, 99]
        let tmp = TempDir::new().unwrap();
        let session_id = "accum-dedup-session-002";

        let ts = begin_task_session(
            "org",
            "proj",
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                session_id: Some(session_id.to_string()),
                ..Default::default()
            }),
        );

        let make_sr = |log_ids: &[i64]| SearchResponse15 {
            similar_patterns: log_ids
                .iter()
                .enumerate()
                .map(|(i, &id)| make_pattern(&format!("p{}", i), Some(id), Some("rid-x")))
                .collect(),
            retrieval_id: Some("rid-x".to_string()),
            count: log_ids.len() as u32,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };

        ts.pin_search(&make_sr(&[42, 99]));
        ts.pin_search(&make_sr(&[77]));
        ts.pin_search(&make_sr(&[42, 55]));

        let out = ts.anchor_trace(make_trace());
        let mut ids = out.applied_log_ids.expect("must have applied_log_ids");
        ids.sort();
        assert_eq!(
            ids,
            vec![42, 55, 77, 99],
            "cross-pin duplicates must be deduped in union"
        );
    }

    #[test]
    fn accum_second_anchor_trace_call_is_idempotent() {
        // After anchor_trace reaps all pin files, a second call is a no-op
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session("org", "proj", Some(tmp_opts(&tmp)));

        let search = SearchResponse15 {
            similar_patterns: vec![make_pattern("p1", Some(10), Some("rid"))],
            retrieval_id: Some("rid".to_string()),
            count: 1,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);

        let out1 = ts.anchor_trace(make_trace());
        assert_eq!(out1.retrieval_id, Some("rid".to_string()));

        // Second call: no pin files remain
        let out2 = ts.anchor_trace(make_trace());
        assert_eq!(
            out2.session_id,
            Some(ts.session_id.clone()),
            "session_id still stamped"
        );
        assert!(
            out2.retrieval_id.is_none(),
            "second anchor_trace: no retrieval_id (idempotent)"
        );
        assert!(
            out2.applied_log_ids.is_none(),
            "second anchor_trace: no log_ids (idempotent)"
        );
    }

    #[test]
    fn accum_pin_with_null_retrieval_id_log_ids_still_accumulated() {
        // A domain-shift pin may have null retrieval_id but still have log_ids.
        // Those log_ids must be unioned even though their retrieval_id is null.
        let tmp = TempDir::new().unwrap();
        let session_id = "accum-null-rid-session-003";

        let ts = begin_task_session(
            "org",
            "proj",
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                session_id: Some(session_id.to_string()),
                ..Default::default()
            }),
        );

        // Main pin: has retrieval_id + log_ids [42]
        ts.pin_search(&SearchResponse15 {
            similar_patterns: vec![make_pattern("p1", Some(42), Some("rid-main"))],
            retrieval_id: Some("rid-main".to_string()),
            count: 1,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        });

        // Domain-shift pin: null retrieval_id, log_ids [99]
        ts.pin_search(&SearchResponse15 {
            similar_patterns: vec![make_pattern("p2", Some(99), None)],
            retrieval_id: None,
            count: 1,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        });

        let out = ts.anchor_trace(make_trace());
        // retrieval_id from main (only one with non-null rid)
        assert_eq!(out.retrieval_id, Some("rid-main".to_string()));
        // log_ids: union [42, 99]
        let mut ids = out.applied_log_ids.expect("must have applied_log_ids");
        ids.sort();
        assert_eq!(ids, vec![42, 99]);
    }

    // -------------------------------------------------------------------------
    // GC: expired per-pin anchors are cleaned on begin_task_session
    // -------------------------------------------------------------------------

    #[test]
    fn gc_removes_expired_per_pin_anchors() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path().join("org__proj");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Write an already-expired per-pin anchor file manually
        let expired = TaskAnchor {
            session_id: "expired-session".to_string(),
            org_id: "org".to_string(),
            project_id: "proj".to_string(),
            retrieval_id: None,
            retrieval_log_ids: vec![],
            created_at_ms: 0,
            expires_at_ms: 1, // already expired
        };
        let pin_path = project_dir.join("expired-session__some-pin-uuid.json");
        std::fs::write(&pin_path, serde_json::to_string(&expired).unwrap()).unwrap();
        assert!(pin_path.exists(), "expired pin anchor must exist before GC");

        // begin_task_session triggers GC
        begin_task_session("org", "proj", Some(tmp_opts(&tmp)));

        assert!(
            !pin_path.exists(),
            "expired per-pin anchor must be removed by GC"
        );
    }

    #[test]
    fn gc_leaves_fresh_per_pin_anchors_intact() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path().join("org__proj");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Write a fresh (non-expired) per-pin anchor
        let now_ms = chrono::Utc::now().timestamp_millis();
        let fresh = TaskAnchor {
            session_id: "fresh-session".to_string(),
            org_id: "org".to_string(),
            project_id: "proj".to_string(),
            retrieval_id: None,
            retrieval_log_ids: vec![],
            created_at_ms: now_ms,
            expires_at_ms: now_ms + DEFAULT_ANCHOR_TTL_MS,
        };
        let pin_path = project_dir.join("fresh-session__some-uuid.json");
        std::fs::write(&pin_path, serde_json::to_string(&fresh).unwrap()).unwrap();

        begin_task_session("org", "proj", Some(tmp_opts(&tmp)));

        assert!(
            pin_path.exists(),
            "fresh per-pin anchor must NOT be removed by GC"
        );
    }

    // -------------------------------------------------------------------------
    // Anchor JSON shape (cross-language byte-identity check)
    // -------------------------------------------------------------------------

    #[test]
    fn anchor_json_has_correct_field_names() {
        let anchor = TaskAnchor {
            session_id: "sid-123".to_string(),
            org_id: "my-org".to_string(),
            project_id: "my-proj".to_string(),
            retrieval_id: Some("rid-456".to_string()),
            retrieval_log_ids: vec![1, 2, 3],
            created_at_ms: 1_700_000_000_000,
            expires_at_ms: 1_700_086_400_000,
        };

        let json = serde_json::to_value(&anchor).unwrap();
        assert_eq!(json["session_id"], "sid-123");
        assert_eq!(json["org_id"], "my-org");
        assert_eq!(json["project_id"], "my-proj");
        assert_eq!(json["retrieval_id"], "rid-456");
        assert_eq!(json["retrieval_log_ids"][0], 1);
        assert_eq!(json["created_at_ms"], 1_700_000_000_000i64);
        assert_eq!(json["expires_at_ms"], 1_700_086_400_000i64);
    }

    #[test]
    fn anchor_json_null_retrieval_id_serialises_correctly() {
        let anchor = TaskAnchor {
            session_id: "sid".to_string(),
            org_id: "o".to_string(),
            project_id: "p".to_string(),
            retrieval_id: None,
            retrieval_log_ids: vec![],
            created_at_ms: 0,
            expires_at_ms: DEFAULT_ANCHOR_TTL_MS,
        };
        let json = serde_json::to_value(&anchor).unwrap();
        // retrieval_id must be present as null (cross-language consistency)
        assert!(json.get("retrieval_id").is_some());
        assert!(json["retrieval_id"].is_null());
    }

    // -------------------------------------------------------------------------
    // A — Re-entry API: injected session_id + load_task_session + anchor_trace
    // -------------------------------------------------------------------------

    #[test]
    fn begin_task_session_uses_injected_session_id() {
        let tmp = TempDir::new().unwrap();
        let opts = TaskSessionOptions {
            sessions_dir: Some(tmp.path().to_path_buf()),
            session_id: Some("my-plugin-session-id".to_string()),
            ..Default::default()
        };
        let ts = begin_task_session("org", "proj", Some(opts));
        assert_eq!(
            ts.session_id, "my-plugin-session-id",
            "begin_task_session must use the caller-provided session_id verbatim"
        );
    }

    #[test]
    fn begin_task_session_generates_fresh_id_when_none() {
        let tmp = TempDir::new().unwrap();
        let opts = TaskSessionOptions {
            sessions_dir: Some(tmp.path().to_path_buf()),
            session_id: None,
            ..Default::default()
        };
        let ts = begin_task_session("org", "proj", Some(opts));
        // Should be a non-empty UUID-shaped string
        assert!(!ts.session_id.is_empty());
        assert_ne!(ts.session_id, "my-plugin-session-id");
    }

    #[test]
    fn load_task_session_binds_to_given_session_id() {
        let tmp = TempDir::new().unwrap();
        let ts = load_task_session(
            "org",
            "proj",
            "existing-session-999",
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );
        assert_eq!(ts.session_id, "existing-session-999");
    }

    #[test]
    fn load_task_session_creates_project_dir() {
        let tmp = TempDir::new().unwrap();
        load_task_session(
            "myorg",
            "myproj",
            "s-abc",
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );
        let project_dir = tmp.path().join("myorg__myproj");
        assert!(
            project_dir.exists(),
            "load_task_session must create the project dir"
        );
    }

    #[test]
    fn load_task_session_recall_anchor_written_by_begin() {
        // Simulates the multi-process pattern:
        //   Process A: begin_task_session → pin_search → writes per-pin anchor file
        //   Process B: load_task_session → anchor_trace → reads, unions & reaps pin files
        let tmp = TempDir::new().unwrap();

        // Process A
        let ts_a = begin_task_session(
            "org",
            "proj",
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                session_id: Some("shared-session-id".to_string()),
                ..Default::default()
            }),
        );
        let search = SearchResponse15 {
            similar_patterns: vec![make_pattern("p1", Some(42), Some("rid-multi"))],
            retrieval_id: Some("rid-multi".to_string()),
            count: 1,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts_a.pin_search(&search);

        // Verify per-pin file exists
        let project_dir = tmp.path().join("org__proj");
        let pins = list_pin_files(&project_dir, "shared-session-id");
        assert!(
            !pins.is_empty(),
            "per-pin file must exist after process A's pin_search"
        );

        // Process B (stateless re-entry by session_id)
        let ts_b = load_task_session(
            "org",
            "proj",
            "shared-session-id",
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );
        let trace = ts_b.anchor_trace(make_trace());

        assert_eq!(
            trace.session_id,
            Some("shared-session-id".to_string()),
            "anchor_trace must stamp the shared session_id"
        );
        assert_eq!(
            trace.retrieval_id,
            Some("rid-multi".to_string()),
            "anchor_trace must carry retrieval_id from process A's pin"
        );
        assert_eq!(
            trace.applied_log_ids,
            Some(vec![42]),
            "anchor_trace must carry log_ids from process A's pin"
        );

        // All per-pin files must be reaped
        let pins_after = list_pin_files(&project_dir, "shared-session-id");
        assert!(
            pins_after.is_empty(),
            "all per-pin files must be reaped after anchor_trace"
        );
    }

    #[test]
    fn module_anchor_trace_stamps_trace_with_matching_anchor() {
        // module-level anchor_trace derives session from trace.session_id
        let tmp = TempDir::new().unwrap();
        let opts_begin = TaskSessionOptions {
            sessions_dir: Some(tmp.path().to_path_buf()),
            session_id: Some("mod-anchor-session".to_string()),
            ..Default::default()
        };
        let ts = begin_task_session("org", "proj", Some(opts_begin));
        let search = SearchResponse15 {
            similar_patterns: vec![make_pattern("p1", Some(77), Some("rid-mod"))],
            retrieval_id: Some("rid-mod".to_string()),
            count: 1,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);

        // Build a trace that already carries session_id (as process B would)
        let mut trace = make_trace();
        trace.session_id = Some("mod-anchor-session".to_string());

        let out = anchor_trace(
            "org",
            "proj",
            trace,
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );

        assert_eq!(out.session_id, Some("mod-anchor-session".to_string()));
        assert_eq!(out.retrieval_id, Some("rid-mod".to_string()));
        assert_eq!(out.applied_log_ids, Some(vec![77]));
    }

    #[test]
    fn module_anchor_trace_noop_when_no_session_id() {
        // If trace.session_id is None, module-level anchor_trace is a no-op
        let tmp = TempDir::new().unwrap();
        let trace = make_trace(); // session_id = None
        let out = anchor_trace(
            "org",
            "proj",
            trace,
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );
        // session_id still None, no error
        assert!(out.session_id.is_none());
        assert!(out.retrieval_id.is_none());
    }

    #[test]
    fn module_anchor_trace_noop_when_no_anchor_exists() {
        // session_id is set but no anchor file → only session_id is untouched
        // (the trace is returned with its existing session_id unchanged)
        let tmp = TempDir::new().unwrap();
        let mut trace = make_trace();
        trace.session_id = Some("nonexistent-anchor-session".to_string());
        let out = anchor_trace(
            "org",
            "proj",
            trace,
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );
        // session_id keeps the value (load_task_session.anchor_trace always stamps it)
        assert_eq!(
            out.session_id,
            Some("nonexistent-anchor-session".to_string())
        );
        // No F-080 fields populated (no anchor)
        assert!(out.retrieval_id.is_none());
        assert!(out.applied_log_ids.is_none());
    }

    // -------------------------------------------------------------------------
    // C — Configurable TTL
    // -------------------------------------------------------------------------

    #[test]
    fn begin_task_session_custom_ttl_stored_in_anchor() {
        let tmp = TempDir::new().unwrap();
        let custom_ttl_ms: i64 = 1_000; // 1 second
        let ts = begin_task_session(
            "org",
            "proj",
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ttl_ms: Some(custom_ttl_ms),
                ..Default::default()
            }),
        );
        let search = SearchResponse15 {
            similar_patterns: vec![],
            retrieval_id: Some("rid-ttl".to_string()),
            count: 0,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);

        // Read the single pin file
        let project_dir = tmp.path().join("org__proj");
        let pins = list_pin_files(&project_dir, &ts.session_id);
        assert_eq!(pins.len(), 1);
        let data = std::fs::read_to_string(&pins[0]).unwrap();
        let anchor: TaskAnchor = serde_json::from_str(&data).unwrap();
        let expected_ttl = anchor.expires_at_ms - anchor.created_at_ms;
        assert_eq!(
            expected_ttl, custom_ttl_ms,
            "pin_search must use the custom TTL ({} ms) not the default",
            custom_ttl_ms
        );
    }

    #[test]
    fn load_task_session_custom_ttl_propagated() {
        let tmp = TempDir::new().unwrap();
        let custom_ttl_ms: i64 = 2_000;
        let ts = load_task_session(
            "org",
            "proj",
            "ttl-session",
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ttl_ms: Some(custom_ttl_ms),
                ..Default::default()
            }),
        );
        // Pin a search to exercise the TTL path
        let search = SearchResponse15 {
            similar_patterns: vec![],
            retrieval_id: Some("rid".to_string()),
            count: 0,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);

        let project_dir = tmp.path().join("org__proj");
        let pins = list_pin_files(&project_dir, "ttl-session");
        assert_eq!(pins.len(), 1);
        let data = std::fs::read_to_string(&pins[0]).unwrap();
        let anchor: TaskAnchor = serde_json::from_str(&data).unwrap();
        assert_eq!(
            anchor.expires_at_ms - anchor.created_at_ms,
            custom_ttl_ms,
            "load_task_session must propagate custom TTL to pin_search"
        );
    }

    #[test]
    fn default_ttl_is_24h() {
        let tmp = TempDir::new().unwrap();
        let ts = begin_task_session(
            "org",
            "proj",
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );
        let search = SearchResponse15 {
            similar_patterns: vec![],
            retrieval_id: None,
            count: 0,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        };
        ts.pin_search(&search);

        let project_dir = tmp.path().join("org__proj");
        let pins = list_pin_files(&project_dir, &ts.session_id);
        assert_eq!(pins.len(), 1);
        let data = std::fs::read_to_string(&pins[0]).unwrap();
        let anchor: TaskAnchor = serde_json::from_str(&data).unwrap();
        let ttl = anchor.expires_at_ms - anchor.created_at_ms;
        assert_eq!(
            ttl, DEFAULT_ANCHOR_TTL_MS,
            "default TTL must be 86_400_000 ms (24 h)"
        );
    }
}

// =============================================================================
// Concurrency test matrix (F-080 plugin-team requirements)
//
// Asserts:
//   1. N concurrent threads writing to the SAME (org,project) anchor store →
//      every distinct session_id's per-pin files recall correctly; no corruption.
//   2. N concurrent threads writing to the SAME shared GraphCache db →
//      NO rusqlite errors (busy_timeout makes writes retry).
//   3. Different projects in parallel → cross-project isolation.
//   4. abort/abandon: pinned but never anchored → expires via GC, no mis-credit.
// =============================================================================

#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use crate::cache::GraphCache;
    use crate::types::{ExecutionResult, MatchFactors, Pattern, SearchResponse15};
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    // ── Helpers ───────────────────────────────────────────────────────────────

    #[allow(dead_code)]
    fn tmp_opts(dir: &TempDir) -> TaskSessionOptions {
        TaskSessionOptions {
            sessions_dir: Some(dir.path().to_path_buf()),
            ..Default::default()
        }
    }

    fn make_search(retrieval_id: &str, log_ids: &[i64]) -> SearchResponse15 {
        let patterns: Vec<Pattern> = log_ids
            .iter()
            .enumerate()
            .map(|(i, &lid)| Pattern {
                id: format!("p{}", i),
                name: String::new(),
                domain: None,
                content: format!("content-{}", i),
                confidence: 0.9,
                observations: 0.0,
                helpful: 0.0,
                harmful: 0.0,
                section: "strategies_and_hard_rules".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: None,
                last_used: None,
                evidence: vec![],
                retrieval_count: 0,
                root_cause: String::new(),
                error_context: String::new(),
                source: None,
                source_project_id: None,
                source_project_name: None,
                local_helpful: 0.0,
                local_harmful: 0.0,
                payload_version: None,
                n_hot_pos: None,
                n_hot_neg: None,
                n_warm_pos: None,
                n_warm_neg: None,
                n_cold_pos: None,
                n_cold_neg: None,
                cumulative_v15_reward: None,
                n_retrieval_no_apply: None,
                task_intent: None,
                effectiveness: None,
                match_factors: Some(MatchFactors {
                    retrieval_log_id: Some(lid),
                    retrieval_id: Some(retrieval_id.to_string()),
                    ..Default::default()
                }),
                root_cause_present: None,
                has_error_context: None,
                birth_primary_lang: None,
                domain_cluster_id: None,
                abstract_domain: None,
                root_cause_cluster_id: None,
                birth_first_tool_bucket: None,
                birth_n_steps_bucket: None,
                birth_has_error: None,
                last_citation_score: None,
                citation_score_ema_30d: None,
                merge_winner_count: None,
                merged_from: vec![],
            })
            .collect();
        SearchResponse15 {
            similar_patterns: patterns,
            retrieval_id: Some(retrieval_id.to_string()),
            count: log_ids.len() as u32,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        }
    }

    fn make_trace() -> ExecutionTrace {
        ExecutionTrace {
            task: "test task".to_string(),
            trajectory: vec![],
            result: ExecutionResult {
                success: true,
                output: "done".to_string(),
                error: None,
                summary: None,
            },
            playbook_used: vec![],
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            git: None,
            session_id: None,
            agent_id: None,
            agent_type: None,
            parent_agent_id: None,
            retrieval_id: None,
            applied_log_ids: None,
        }
    }

    // ── 1. N concurrent writers to the SAME anchor store ─────────────────────

    #[test]
    fn concurrent_anchor_writes_all_correct_no_corruption() {
        const N: usize = 20;
        let tmp = TempDir::new().unwrap();

        let ids: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let handles: Vec<_> = (0..N)
            .map(|i| {
                let tmp_path = tmp.path().to_path_buf();
                let ids_c = Arc::clone(&ids);
                let errors_c = Arc::clone(&errors);
                std::thread::spawn(move || {
                    let opts = TaskSessionOptions {
                        sessions_dir: Some(tmp_path.clone()),
                        ..Default::default()
                    };
                    let ts = begin_task_session("org-conc", "proj-conc", Some(opts));
                    let search =
                        make_search(&format!("ret-{}", i), &[i as i64 * 10, i as i64 * 10 + 1]);
                    ts.pin_search(&search);

                    // Verify the per-pin file was written and is parseable
                    let project_dir = tmp_path.join("org-conc__proj-conc");
                    let prefix = format!("{}__", ts.session_id);
                    let pin_files: Vec<_> = std::fs::read_dir(&project_dir)
                        .into_iter()
                        .flatten()
                        .flatten()
                        .filter_map(|e| {
                            let p = e.path();
                            let name = p.file_name()?.to_str()?.to_string();
                            if name.starts_with(&prefix) && name.ends_with(".json") {
                                Some(p)
                            } else {
                                None
                            }
                        })
                        .collect();

                    if pin_files.is_empty() {
                        errors_c
                            .lock()
                            .unwrap()
                            .push(format!("thread {}: no pin file found", i));
                        return;
                    }

                    let data = std::fs::read_to_string(&pin_files[0]).unwrap();
                    let a: TaskAnchor = match serde_json::from_str(&data) {
                        Ok(a) => a,
                        Err(e) => {
                            errors_c
                                .lock()
                                .unwrap()
                                .push(format!("thread {}: parse error: {}", i, e));
                            return;
                        }
                    };
                    if a.retrieval_log_ids.len() != 2 {
                        errors_c.lock().unwrap().push(format!(
                            "thread {}: expected 2 log_ids, got {:?}",
                            i, a.retrieval_log_ids
                        ));
                    }

                    ids_c.lock().unwrap().push(ts.session_id.clone());
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let errs = errors.lock().unwrap();
        for e in errs.iter() {
            panic!("concurrency error: {}", e);
        }

        // All session IDs must be unique
        let all_ids = ids.lock().unwrap();
        assert_eq!(all_ids.len(), N, "expected {} session IDs", N);
        let unique: HashSet<_> = all_ids.iter().collect();
        assert_eq!(unique.len(), N, "all {} session IDs must be unique", N);
    }

    #[test]
    fn concurrent_anchor_recall_no_session_id_bleed() {
        const N: usize = 15;
        let tmp = TempDir::new().unwrap();

        // Create N sessions with distinct retrieval_ids, then anchor_trace all concurrently
        let sessions: Vec<(TaskSession, String, i64)> = (0..N)
            .map(|i| {
                let opts = TaskSessionOptions {
                    sessions_dir: Some(tmp.path().to_path_buf()),
                    ..Default::default()
                };
                let ts = begin_task_session("org-bleed", "proj-bleed", Some(opts));
                let rid = format!("ret-bleed-{}", i);
                ts.pin_search(&make_search(&rid, &[i as i64]));
                (ts, rid, i as i64)
            })
            .collect();

        let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let handles: Vec<_> = sessions
            .into_iter()
            .map(|(ts, expected_rid, expected_log)| {
                let errors_c = Arc::clone(&errors);
                std::thread::spawn(move || {
                    let trace = ts.anchor_trace(make_trace());
                    if trace.retrieval_id.as_deref() != Some(&expected_rid) {
                        errors_c.lock().unwrap().push(format!(
                            "retrieval_id bleed: got {:?}, want {:?}",
                            trace.retrieval_id, expected_rid
                        ));
                    }
                    let got_ids = trace.applied_log_ids.unwrap_or_default();
                    if got_ids != vec![expected_log] {
                        errors_c.lock().unwrap().push(format!(
                            "applied_log_ids bleed: got {:?}, want [{:?}]",
                            got_ids, expected_log
                        ));
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let errs = errors.lock().unwrap();
        for e in errs.iter() {
            panic!("bleed error: {}", e);
        }
    }

    // ── 2. N concurrent writers to the SAME GraphCache db — no BUSY error ────

    #[test]
    fn concurrent_graph_cache_writes_no_busy_error() {
        const N: usize = 15;
        let tmp = TempDir::new().unwrap();

        // Open N distinct GraphCache instances all pointing at the SAME db file.
        // The db uses a Mutex internally so concurrent access is safe at the
        // connection level; busy_timeout handles cross-process/connection contention.
        let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let handles: Vec<_> = (0..N)
            .map(|i| {
                let cache_dir = tmp.path().to_path_buf();
                let errors_c = Arc::clone(&errors);
                std::thread::spawn(move || {
                    let gc = GraphCache::new("orgbusy", "projbusy", Some(cache_dir)).unwrap();
                    let payload = format!(r#"{{"id":"p{}"}}"#, i);
                    if let Err(e) = gc.upsert_pattern(&format!("p{}", i), &payload, 0.0) {
                        let msg = e.to_string();
                        if msg.contains("locked") || msg.contains("busy") || msg.contains("BUSY") {
                            errors_c
                                .lock()
                                .unwrap()
                                .push(format!("thread {}: SQLITE_BUSY: {}", i, msg));
                        }
                        // Other transient errors (like connection pool) are ignored —
                        // only SQLITE_BUSY is a correctness failure here.
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let errs = errors.lock().unwrap();
        for e in errs.iter() {
            panic!("unexpected SQLITE_BUSY: {}", e);
        }
    }

    #[test]
    fn graph_cache_busy_timeout_verified_at_runtime() {
        let tmp = TempDir::new().unwrap();
        let gc = GraphCache::new("org", "proj", Some(tmp.path().to_path_buf())).unwrap();
        let timeout = gc.get_busy_timeout().unwrap();
        assert_eq!(
            timeout, 5000,
            "GraphCache busy_timeout must be 5000 ms after WAL"
        );
    }

    // ── 3. Cross-project isolation ────────────────────────────────────────────

    #[test]
    fn cross_project_isolation_anchors_never_bleed() {
        const TASKS_PER_PROJECT: usize = 5;
        let projects = ["alpha", "beta", "gamma", "delta"];
        let tmp = TempDir::new().unwrap();

        let all_sessions: Arc<Mutex<Vec<(String, &'static str, String)>>> =
            Arc::new(Mutex::new(Vec::new()));
        let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let handles: Vec<_> = projects
            .iter()
            .flat_map(|&proj| (0..TASKS_PER_PROJECT).map(move |i| (proj, i)))
            .map(|(proj, i)| {
                let tmp_path = tmp.path().to_path_buf();
                let all_c = Arc::clone(&all_sessions);
                let errors_c = Arc::clone(&errors);
                std::thread::spawn(move || {
                    let opts = TaskSessionOptions {
                        sessions_dir: Some(tmp_path.clone()),
                        ..Default::default()
                    };
                    let ts = begin_task_session("orgiso", proj, Some(opts));
                    let rid = format!("ret-{}-{}", proj, i);
                    ts.pin_search(&make_search(&rid, &[i as i64]));

                    // Verify at least one per-pin file exists under the correct project dir
                    let expected_dir = ts.project_dir.clone();
                    let prefix = format!("{}__", ts.session_id);
                    let pin_exists = std::fs::read_dir(&expected_dir)
                        .into_iter()
                        .flatten()
                        .flatten()
                        .any(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            name.starts_with(&prefix) && name.ends_with(".json")
                        });

                    if !pin_exists {
                        errors_c.lock().unwrap().push(format!(
                            "proj {} task {}: pin file missing in {:?}",
                            proj, i, expected_dir
                        ));
                    }

                    all_c
                        .lock()
                        .unwrap()
                        .push((ts.session_id.clone(), proj, rid));
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let errs = errors.lock().unwrap();
        for e in errs.iter() {
            panic!("isolation error: {}", e);
        }

        // Cross-check: no session appears in any other project's directory
        let tmp_path = tmp.path().to_path_buf();
        let all = all_sessions.lock().unwrap();
        for &proj in &projects {
            let proj_dir = tmp_path.join(format!("orgiso__{}", proj));
            if !proj_dir.exists() {
                continue;
            }
            // Collect all session_ids that appear in this project dir (by prefix)
            let session_ids_in_dir: HashSet<String> = std::fs::read_dir(&proj_dir)
                .unwrap()
                .flatten()
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    // Extract session_id from <session_id>__<pin_uuid>.json
                    if name.ends_with(".json") {
                        let without_ext = name.trim_end_matches(".json");
                        // session_id is everything before the last `__<pin_uuid>` suffix
                        // pin_uuid is a UUID4 (36 chars: 8-4-4-4-12)
                        if let Some(pos) = without_ext.rfind("__") {
                            Some(without_ext[..pos].to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            for (sid, owner_proj, _) in all.iter() {
                if *owner_proj == proj {
                    assert!(
                        session_ids_in_dir.contains(sid),
                        "project {} missing its own session {}",
                        proj,
                        sid
                    );
                } else {
                    assert!(
                        !session_ids_in_dir.contains(sid),
                        "project {} has foreign session {} (from {})",
                        proj,
                        sid,
                        owner_proj
                    );
                }
            }
        }
    }

    // ── 4. Abandon/abort — expires via GC, no mis-credit ─────────────────────

    #[test]
    fn abandoned_expired_anchor_swept_by_gc_not_mis_credited() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path().join("orgabandon__projabandon");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Write an already-expired per-pin anchor (simulates abandoned task)
        let abandoned_id = "abandoned-session-id";
        let expired = TaskAnchor {
            session_id: abandoned_id.to_string(),
            org_id: "orgabandon".to_string(),
            project_id: "projabandon".to_string(),
            retrieval_id: Some("ret-abandoned".to_string()),
            retrieval_log_ids: vec![999],
            created_at_ms: 0,
            expires_at_ms: 1, // already expired
        };
        let data = serde_json::to_string(&expired).unwrap();
        // Write as a per-pin file
        std::fs::write(
            project_dir.join(format!("{}__some-pin-uuid.json", abandoned_id)),
            &data,
        )
        .unwrap();

        // Write a live per-pin anchor that must NOT be touched by GC
        let live_id = "live-session-id";
        let now_ms = chrono::Utc::now().timestamp_millis();
        let live = TaskAnchor {
            session_id: live_id.to_string(),
            org_id: "orgabandon".to_string(),
            project_id: "projabandon".to_string(),
            retrieval_id: Some("ret-live".to_string()),
            retrieval_log_ids: vec![888],
            created_at_ms: now_ms,
            expires_at_ms: now_ms + DEFAULT_ANCHOR_TTL_MS,
        };
        let live_data = serde_json::to_string(&live).unwrap();
        let live_path = project_dir.join(format!("{}__live-pin-uuid.json", live_id));
        std::fs::write(&live_path, &live_data).unwrap();

        // Trigger GC
        let opts = TaskSessionOptions {
            sessions_dir: Some(tmp.path().to_path_buf()),
            ..Default::default()
        };
        let fresh = begin_task_session("orgabandon", "projabandon", Some(opts));

        // Expired per-pin anchor must be gone
        let expired_path = project_dir.join(format!("{}__some-pin-uuid.json", abandoned_id));
        assert!(
            !expired_path.exists(),
            "expired per-pin anchor must be swept by GC"
        );
        // Live per-pin anchor must remain
        assert!(
            live_path.exists(),
            "live per-pin anchor must NOT be swept by GC"
        );

        // Fresh session must not carry abandoned data
        let trace = fresh.anchor_trace(make_trace());
        assert!(
            trace.retrieval_id.is_none(),
            "fresh task must not carry abandoned retrieval_id: {:?}",
            trace.retrieval_id
        );
    }

    #[test]
    fn abandoned_not_foreign_reaped_by_other_session() {
        let tmp = TempDir::new().unwrap();

        // Session A: pin search, then expire its pin anchor
        let opts_a = TaskSessionOptions {
            sessions_dir: Some(tmp.path().to_path_buf()),
            ..Default::default()
        };
        let ts_a = begin_task_session("orgfr", "projfr", Some(opts_a));
        ts_a.pin_search(&make_search("ret-A", &[1]));

        // Find session A's pin file and manually expire it
        let project_dir = ts_a.project_dir.clone();
        let prefix_a = format!("{}__", ts_a.session_id);
        let pin_a: Vec<_> = std::fs::read_dir(&project_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                let name = p.file_name()?.to_str()?.to_string();
                if name.starts_with(&prefix_a) && name.ends_with(".json") {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();
        assert!(!pin_a.is_empty());
        let data = std::fs::read_to_string(&pin_a[0]).unwrap();
        let mut anchor: TaskAnchor = serde_json::from_str(&data).unwrap();
        anchor.expires_at_ms = chrono::Utc::now().timestamp_millis() - 1;
        std::fs::write(&pin_a[0], serde_json::to_string(&anchor).unwrap()).unwrap();

        // Session B: fresh session — calls anchor_trace WITHOUT pin_search
        let opts_b = TaskSessionOptions {
            sessions_dir: Some(tmp.path().to_path_buf()),
            ..Default::default()
        };
        let ts_b = begin_task_session("orgfr", "projfr", Some(opts_b));
        let trace_b = ts_b.anchor_trace(make_trace());

        // B must NOT have received A's retrieval data
        assert!(
            trace_b.retrieval_id.is_none(),
            "session B got foreign retrieval_id: {:?}",
            trace_b.retrieval_id
        );
        assert!(
            trace_b.applied_log_ids.is_none(),
            "session B got foreign applied_log_ids: {:?}",
            trace_b.applied_log_ids
        );
        assert_eq!(
            trace_b.session_id,
            Some(ts_b.session_id.clone()),
            "session B has wrong session_id"
        );
    }

    #[test]
    fn live_abandoned_anchor_not_swept_by_gc() {
        let tmp = TempDir::new().unwrap();

        // Pin an anchor but never call anchor_trace
        let opts = TaskSessionOptions {
            sessions_dir: Some(tmp.path().to_path_buf()),
            ..Default::default()
        };
        let ts_abandoned = begin_task_session("orgliveabandon", "projliveabandon", Some(opts));
        ts_abandoned.pin_search(&make_search("ret-live-abandon", &[42]));

        // Find the pin file
        let project_dir = ts_abandoned.project_dir.clone();
        let prefix = format!("{}__", ts_abandoned.session_id);
        let pin_files: Vec<_> = std::fs::read_dir(&project_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                let name = p.file_name()?.to_str()?.to_string();
                if name.starts_with(&prefix) && name.ends_with(".json") {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();
        assert!(!pin_files.is_empty(), "pin file must exist before GC");

        // Trigger GC
        let opts2 = TaskSessionOptions {
            sessions_dir: Some(tmp.path().to_path_buf()),
            ..Default::default()
        };
        begin_task_session("orgliveabandon", "projliveabandon", Some(opts2));

        // Live pin file must still exist
        for pf in &pin_files {
            assert!(pf.exists(), "live pin anchor must NOT be swept by GC");
            let data = std::fs::read_to_string(pf).unwrap();
            let anchor: TaskAnchor = serde_json::from_str(&data).unwrap();
            assert_eq!(
                anchor.retrieval_id,
                Some("ret-live-abandon".to_string()),
                "live pin anchor data must be intact"
            );
        }
    }
}

// =============================================================================
// Multi-process re-entry test matrix (in-process simulation)
//
// Mirrors the 5-test matrix in the TypeScript test suite
// (task-session-multiprocess.test.ts). The Rust version uses a shared TempDir
// on disk but drops the process-A handle before constructing process B, so
// there is no shared *heap* state between A and B — only the per-pin anchor
// files on disk. This faithfully models the SubagentStart → SubagentStop split.
//
//  1. A pins (injected sessionId S) → B via load_task_session(S): retrieval
//     fields populated; all pin files reaped.
//  2. A pins (injected sessionId S) → B via module-level anchor_trace with
//     trace.session_id = S: same result.
//  3. Negative: B with UNKNOWN session_id → session_id stamped only; no crash.
//  4. Cross-process reap: files A wrote are deleted by B.
//  5. TTL: anchor whose expires_at_ms is in the past → treated as missing.
// =============================================================================

#[cfg(test)]
mod multiprocess_reentry_tests {
    use super::*;
    use crate::types::{ExecutionResult, MatchFactors, Pattern, SearchResponse15};
    use tempfile::TempDir;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn opts(dir: &TempDir) -> TaskSessionOptions {
        TaskSessionOptions {
            sessions_dir: Some(dir.path().to_path_buf()),
            ..Default::default()
        }
    }

    fn opts_with_id(dir: &TempDir, session_id: &str) -> TaskSessionOptions {
        TaskSessionOptions {
            sessions_dir: Some(dir.path().to_path_buf()),
            session_id: Some(session_id.to_string()),
            ..Default::default()
        }
    }

    fn make_search_response(retrieval_id: &str, log_ids: &[i64]) -> SearchResponse15 {
        let patterns: Vec<Pattern> = log_ids
            .iter()
            .enumerate()
            .map(|(i, &lid)| Pattern {
                id: format!("p{}", i),
                name: String::new(),
                domain: None,
                content: format!("content-{}", i),
                confidence: 0.9,
                observations: 0.0,
                helpful: 0.0,
                harmful: 0.0,
                section: "strategies_and_hard_rules".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: None,
                last_used: None,
                evidence: vec![],
                retrieval_count: 0,
                root_cause: String::new(),
                error_context: String::new(),
                source: None,
                source_project_id: None,
                source_project_name: None,
                local_helpful: 0.0,
                local_harmful: 0.0,
                payload_version: None,
                n_hot_pos: None,
                n_hot_neg: None,
                n_warm_pos: None,
                n_warm_neg: None,
                n_cold_pos: None,
                n_cold_neg: None,
                cumulative_v15_reward: None,
                n_retrieval_no_apply: None,
                task_intent: None,
                effectiveness: None,
                match_factors: Some(MatchFactors {
                    retrieval_log_id: Some(lid),
                    retrieval_id: Some(retrieval_id.to_string()),
                    ..Default::default()
                }),
                root_cause_present: None,
                has_error_context: None,
                birth_primary_lang: None,
                domain_cluster_id: None,
                abstract_domain: None,
                root_cause_cluster_id: None,
                birth_first_tool_bucket: None,
                birth_n_steps_bucket: None,
                birth_has_error: None,
                last_citation_score: None,
                citation_score_ema_30d: None,
                merge_winner_count: None,
                merged_from: vec![],
            })
            .collect();
        SearchResponse15 {
            similar_patterns: patterns,
            retrieval_id: Some(retrieval_id.to_string()),
            count: log_ids.len() as u32,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        }
    }

    fn make_trace() -> crate::types::ExecutionTrace {
        crate::types::ExecutionTrace {
            task: "task from process B".to_string(),
            trajectory: vec![],
            result: ExecutionResult {
                success: true,
                output: "done".to_string(),
                error: None,
                summary: None,
            },
            playbook_used: vec![],
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            git: None,
            session_id: None,
            agent_id: None,
            agent_type: None,
            parent_agent_id: None,
            retrieval_id: None,
            applied_log_ids: None,
        }
    }

    /// Helper: list per-pin files for a session in the given project dir.
    fn list_pin_files(
        project_dir: &std::path::PathBuf,
        session_id: &str,
    ) -> Vec<std::path::PathBuf> {
        let prefix = format!("{}__", session_id);
        std::fs::read_dir(project_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_str()?.to_string();
                if name.starts_with(&prefix) && name.ends_with(".json") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }

    // ── Test 1: A pins (injected id) → B via load_task_session ──────────────

    /// A pins with injected session_id S; B re-enters via load_task_session(S).
    /// Trace receives retrieval_id + applied_log_ids from A's pin; pin files reaped.
    #[test]
    fn reentry_1_load_task_session_receives_a_pin_fields() {
        let tmp = TempDir::new().unwrap();
        let session_id = "subagent-id-derived-rust-0001";
        let rid = "rid-from-a-rust-0001";

        // ── Process A (in-process, then handle dropped) ───────────────────────
        {
            let ts_a = begin_task_session(
                "org-rust",
                "proj-rust",
                Some(opts_with_id(&tmp, session_id)),
            );
            ts_a.pin_search(&make_search_response(rid, &[42, 99]));
            // ts_a is dropped here — simulating process A exiting
        }

        // Per-pin files must exist on disk
        let project_dir = tmp.path().join("org-rust__proj-rust");
        let pins = list_pin_files(&project_dir, session_id);
        assert!(!pins.is_empty(), "per-pin files must exist after process A");

        // ── Process B (fresh load — no shared heap state from A) ──────────────
        let ts_b = load_task_session("org-rust", "proj-rust", session_id, Some(opts(&tmp)));
        let trace = ts_b.anchor_trace(make_trace());

        assert_eq!(
            trace.session_id,
            Some(session_id.to_string()),
            "session_id stamped"
        );
        assert_eq!(
            trace.retrieval_id,
            Some(rid.to_string()),
            "retrieval_id from A's pin"
        );

        let mut ids = trace.applied_log_ids.unwrap_or_default();
        ids.sort();
        assert_eq!(ids, vec![42, 99], "applied_log_ids from A's pin");

        // All pin files must be reaped by B
        let pins_after = list_pin_files(&project_dir, session_id);
        assert!(pins_after.is_empty(), "all pin files reaped by process B");
    }

    // ── Test 2: A pins (injected id) → B via module-level anchor_trace ───────

    /// A pins with injected session_id S; B uses module-level anchor_trace
    /// with trace.session_id = S. Same retrieval fields; pin files reaped.
    #[test]
    fn reentry_2_module_level_anchor_trace_receives_a_pin_fields() {
        let tmp = TempDir::new().unwrap();
        let session_id = "subagent-id-derived-rust-0002";
        let rid = "rid-from-a-rust-0002";

        // ── Process A ─────────────────────────────────────────────────────────
        {
            let ts_a = begin_task_session(
                "org-rust",
                "proj-rust",
                Some(opts_with_id(&tmp, session_id)),
            );
            ts_a.pin_search(&make_search_response(rid, &[10, 20]));
        }

        let project_dir = tmp.path().join("org-rust__proj-rust");
        let pins = list_pin_files(&project_dir, session_id);
        assert!(!pins.is_empty(), "pre-condition: per-pin files exist");

        // ── Process B (module-level anchor_trace, derives session from trace) ─
        let mut trace = make_trace();
        trace.session_id = Some(session_id.to_string());

        let out = anchor_trace(
            "org-rust",
            "proj-rust",
            trace,
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );

        assert_eq!(out.session_id, Some(session_id.to_string()));
        assert_eq!(out.retrieval_id, Some(rid.to_string()));
        let mut ids = out.applied_log_ids.unwrap_or_default();
        ids.sort();
        assert_eq!(ids, vec![10, 20]);

        // All pin files reaped
        let pins_after = list_pin_files(&project_dir, session_id);
        assert!(
            pins_after.is_empty(),
            "all pin files reaped by module-level anchor_trace"
        );
    }

    // ── Test 3: Negative — B with unknown session_id ─────────────────────────

    /// B uses an unknown session_id (no prior A, no pin files).
    /// session_id is still stamped on the trace; no retrieval fields; no crash.
    #[test]
    fn reentry_3_negative_unknown_session_id_no_crash_no_fields() {
        let tmp = TempDir::new().unwrap();
        let unknown_id = "completely-unknown-session-rust-9999";

        // No process A — no pin files on disk
        let ts_b = load_task_session("org-neg", "proj-neg", unknown_id, Some(opts(&tmp)));
        let trace = ts_b.anchor_trace(make_trace());

        assert_eq!(
            trace.session_id,
            Some(unknown_id.to_string()),
            "session_id stamped even without anchor"
        );
        assert!(
            trace.retrieval_id.is_none(),
            "no retrieval_id for unknown session"
        );
        assert!(
            trace.applied_log_ids.is_none(),
            "no applied_log_ids for unknown session"
        );
    }

    // ── Test 4: Cross-process reap ────────────────────────────────────────────

    /// The per-pin files written by A (with a specific session_id) can be deleted
    /// by B without any shared handle — reap is keyed purely by session_id prefix on disk.
    #[test]
    fn reentry_4_cross_process_reap_file_a_deleted_by_b() {
        let tmp = TempDir::new().unwrap();
        let session_id = "cross-proc-reap-rust-4444";
        let rid = "rid-reap-rust";

        // ── A: write per-pin anchor ────────────────────────────────────────────
        {
            let ts_a = begin_task_session(
                "org-reap",
                "proj-reap",
                Some(opts_with_id(&tmp, session_id)),
            );
            ts_a.pin_search(&make_search_response(rid, &[1]));
        }

        let project_dir = tmp.path().join("org-reap__proj-reap");
        let pins = list_pin_files(&project_dir, session_id);
        assert!(!pins.is_empty(), "A wrote the per-pin anchor");

        // Confirm pin anchor belongs to the right session
        let raw = std::fs::read_to_string(&pins[0]).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed["session_id"], session_id);

        // ── B: load by session_id and reap (no handle from A) ─────────────────
        let ts_b = load_task_session("org-reap", "proj-reap", session_id, Some(opts(&tmp)));
        ts_b.anchor_trace(make_trace());

        // All pin files must be gone — B deleted them using only the session_id
        let pins_after = list_pin_files(&project_dir, session_id);
        assert!(
            pins_after.is_empty(),
            "B deleted all pin files written by A"
        );
    }

    // ── Test 5: TTL — expired anchor treated as missing ───────────────────────

    /// A per-pin file whose expires_at_ms is in the past is skipped (treated as missing).
    /// trace.session_id is stamped but no retrieval fields are populated.
    #[test]
    fn reentry_5_expired_anchor_treated_as_missing() {
        let tmp = TempDir::new().unwrap();
        let session_id = "ttl-override-rust-5555";
        let pin_uuid = "expired-pin-uuid-1234";

        // Write a per-pin file with expires_at_ms already in the past
        let dir = tmp.path().join("org-ttl__proj-ttl");
        std::fs::create_dir_all(&dir).unwrap();

        let now_ms = chrono::Utc::now().timestamp_millis();
        let expired = TaskAnchor {
            session_id: session_id.to_string(),
            org_id: "org-ttl".to_string(),
            project_id: "proj-ttl".to_string(),
            retrieval_id: Some("rid-expired".to_string()),
            retrieval_log_ids: vec![77],
            created_at_ms: now_ms - 10_000,
            expires_at_ms: now_ms - 5_000, // expired 5 s ago
        };
        let pin_file = dir.join(format!("{}__{}.json", session_id, pin_uuid));
        std::fs::write(&pin_file, serde_json::to_string(&expired).unwrap()).unwrap();
        assert!(
            pin_file.exists(),
            "pre-condition: expired per-pin anchor on disk"
        );

        // B loads the session — the expired pin must be treated as missing (skipped)
        let ts_b = load_task_session(
            "org-ttl",
            "proj-ttl",
            session_id,
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );
        let trace = ts_b.anchor_trace(make_trace());

        assert_eq!(
            trace.session_id,
            Some(session_id.to_string()),
            "session_id stamped"
        );
        assert!(
            trace.retrieval_id.is_none(),
            "expired pin gives no retrieval_id"
        );
        assert!(
            trace.applied_log_ids.is_none(),
            "expired pin gives no applied_log_ids"
        );

        // The expired pin file should be reaped (it was in the glob result)
        assert!(
            !pin_file.exists(),
            "expired per-pin file reaped by anchor_trace"
        );
    }
}

// =============================================================================
// read_f080: non-reaping F-080 peek tests
// =============================================================================

#[cfg(test)]
mod read_f080_tests {
    use super::*;
    use crate::types::{ExecutionResult, MatchFactors, Pattern, SearchResponse15};
    use tempfile::TempDir;

    fn tmp_opts(dir: &TempDir) -> TaskSessionOptions {
        TaskSessionOptions {
            sessions_dir: Some(dir.path().to_path_buf()),
            ..Default::default()
        }
    }

    fn tmp_opts_with_id(dir: &TempDir, session_id: &str) -> TaskSessionOptions {
        TaskSessionOptions {
            sessions_dir: Some(dir.path().to_path_buf()),
            session_id: Some(session_id.to_string()),
            ..Default::default()
        }
    }

    fn make_pattern_rf(id: &str, log_id: Option<i64>, retrieval_id: Option<&str>) -> Pattern {
        Pattern {
            id: id.to_string(),
            name: String::new(),
            domain: None,
            content: "test content".to_string(),
            confidence: 0.8,
            observations: 0.0,
            helpful: 0.0,
            harmful: 0.0,
            section: "strategies_and_hard_rules".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: None,
            last_used: None,
            evidence: vec![],
            retrieval_count: 0,
            root_cause: String::new(),
            error_context: String::new(),
            source: None,
            source_project_id: None,
            source_project_name: None,
            local_helpful: 0.0,
            local_harmful: 0.0,
            payload_version: None,
            n_hot_pos: None,
            n_hot_neg: None,
            n_warm_pos: None,
            n_warm_neg: None,
            n_cold_pos: None,
            n_cold_neg: None,
            cumulative_v15_reward: None,
            n_retrieval_no_apply: None,
            task_intent: None,
            effectiveness: None,
            match_factors: Some(MatchFactors {
                retrieval_log_id: log_id,
                retrieval_id: retrieval_id.map(|s| s.to_string()),
                ..Default::default()
            }),
            root_cause_present: None,
            has_error_context: None,
            birth_primary_lang: None,
            domain_cluster_id: None,
            abstract_domain: None,
            root_cause_cluster_id: None,
            birth_first_tool_bucket: None,
            birth_n_steps_bucket: None,
            birth_has_error: None,
            last_citation_score: None,
            citation_score_ema_30d: None,
            merge_winner_count: None,
            merged_from: vec![],
        }
    }

    fn make_trace_rf() -> crate::types::ExecutionTrace {
        crate::types::ExecutionTrace {
            task: "test task".to_string(),
            trajectory: vec![],
            result: ExecutionResult {
                success: true,
                output: "done".to_string(),
                error: None,
                summary: None,
            },
            playbook_used: vec![],
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            git: None,
            session_id: None,
            agent_id: None,
            agent_type: None,
            parent_agent_id: None,
            retrieval_id: None,
            applied_log_ids: None,
        }
    }

    fn list_pin_files_rf(
        project_dir: &std::path::PathBuf,
        session_id: &str,
    ) -> Vec<std::path::PathBuf> {
        let prefix = format!("{}__", session_id);
        std::fs::read_dir(project_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_str()?.to_string();
                if name.starts_with(&prefix) && name.ends_with(".json") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }

    // ── Test: unknown session → empty view ────────────────────────────────────

    #[test]
    fn read_f080_unknown_session_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let ts = load_task_session("org", "proj", "no-such-session", Some(tmp_opts(&tmp)));
        let view = ts.read_f080();
        assert_eq!(view.retrieval_id, None);
        assert!(view.applied_log_ids.is_empty());
    }

    // ── Test: two pins → union + earliest retrieval_id, files NOT reaped ─────

    #[test]
    fn read_f080_two_pins_union_and_files_survive() {
        let tmp = TempDir::new().unwrap();
        let session_id = "rf080-two-pins-session";

        let ts = begin_task_session("org", "proj", Some(tmp_opts_with_id(&tmp, session_id)));

        // Pin 1: log_ids [10, 20], retrieval_id "rid-A"
        ts.pin_search(&SearchResponse15 {
            similar_patterns: vec![
                make_pattern_rf("p1", Some(10), Some("rid-A")),
                make_pattern_rf("p2", Some(20), Some("rid-A")),
            ],
            retrieval_id: Some("rid-A".to_string()),
            count: 2,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        });

        // Ensure the second pin gets a later created_at_ms
        std::thread::sleep(std::time::Duration::from_millis(2));

        // Pin 2: log_ids [30], retrieval_id "rid-B" (later → should NOT win)
        ts.pin_search(&SearchResponse15 {
            similar_patterns: vec![make_pattern_rf("p3", Some(30), Some("rid-B"))],
            retrieval_id: Some("rid-B".to_string()),
            count: 1,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        });

        let project_dir = tmp.path().join("org__proj");

        // Pre-condition: two pin files on disk
        let pins_before = list_pin_files_rf(&project_dir, session_id);
        assert_eq!(pins_before.len(), 2, "pre-condition: two pin files on disk");

        // read_f080: non-reaping peek
        let view = ts.read_f080();

        // Union of log_ids [10, 20, 30] (sorted)
        assert_eq!(view.applied_log_ids, vec![10, 20, 30]);
        // Earliest pin's retrieval_id wins
        assert_eq!(view.retrieval_id, Some("rid-A".to_string()));

        // CRITICAL: pin files must still exist (NOT reaped)
        let pins_after = list_pin_files_rf(&project_dir, session_id);
        assert_eq!(
            pins_after.len(),
            2,
            "read_f080 must NOT reap pin files — both must still be on disk"
        );
    }

    // ── Test: read_f080 then anchor_trace still works and reaps ──────────────

    #[test]
    fn read_f080_followed_by_anchor_trace_reaps_correctly() {
        let tmp = TempDir::new().unwrap();
        let session_id = "rf080-then-anchor-session";

        let ts = begin_task_session("org", "proj", Some(tmp_opts_with_id(&tmp, session_id)));

        ts.pin_search(&SearchResponse15 {
            similar_patterns: vec![make_pattern_rf("p1", Some(42), Some("rid-X"))],
            retrieval_id: Some("rid-X".to_string()),
            count: 1,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        });

        let project_dir = tmp.path().join("org__proj");

        // read_f080 first (must not reap)
        let view = ts.read_f080();
        assert_eq!(view.retrieval_id, Some("rid-X".to_string()));
        assert_eq!(view.applied_log_ids, vec![42]);

        // Files still present after read_f080
        let pins_after_peek = list_pin_files_rf(&project_dir, session_id);
        assert_eq!(pins_after_peek.len(), 1, "pin file must survive read_f080");

        // anchor_trace must still work and produce correct output
        let trace = ts.anchor_trace(make_trace_rf());
        assert_eq!(trace.session_id, Some(session_id.to_string()));
        assert_eq!(trace.retrieval_id, Some("rid-X".to_string()));
        assert_eq!(trace.applied_log_ids, Some(vec![42]));

        // Now files must be reaped
        let pins_after_anchor = list_pin_files_rf(&project_dir, session_id);
        assert!(
            pins_after_anchor.is_empty(),
            "anchor_trace must reap all pin files"
        );
    }

    // ── Test: module-level read_f080 convenience ──────────────────────────────

    #[test]
    fn module_read_f080_returns_correct_view() {
        let tmp = TempDir::new().unwrap();
        let session_id = "rf080-module-level-session";

        let ts = begin_task_session("org", "proj", Some(tmp_opts_with_id(&tmp, session_id)));

        ts.pin_search(&SearchResponse15 {
            similar_patterns: vec![
                make_pattern_rf("p1", Some(5), Some("rid-mod")),
                make_pattern_rf("p2", Some(7), Some("rid-mod")),
            ],
            retrieval_id: Some("rid-mod".to_string()),
            count: 2,
            local_count: 0,
            shared_count: 0,
            domains_summary: None,
            search_params: None,
            tokens_in_response: 0,
            expanded: None,
        });

        // Module-level read_f080
        let view = read_f080(
            "org",
            "proj",
            session_id,
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );

        assert_eq!(view.retrieval_id, Some("rid-mod".to_string()));
        assert_eq!(view.applied_log_ids, vec![5, 7]);

        // Pin files still on disk
        let project_dir = tmp.path().join("org__proj");
        let pins = list_pin_files_rf(&project_dir, session_id);
        assert_eq!(
            pins.len(),
            1,
            "module-level read_f080 must not reap pin files"
        );
    }

    // ── Test: expired pins skipped (best-effort deleted), surviving only ──────

    #[test]
    fn read_f080_skips_expired_pins() {
        let tmp = TempDir::new().unwrap();
        let session_id = "rf080-expired-session";

        let project_dir = tmp.path().join("org__proj");
        std::fs::create_dir_all(&project_dir).unwrap();

        let now_ms = chrono::Utc::now().timestamp_millis();

        // Write an expired pin file directly
        let expired_uuid = "expired-pin-uuid-0001";
        let expired = TaskAnchor {
            session_id: session_id.to_string(),
            org_id: "org".to_string(),
            project_id: "proj".to_string(),
            retrieval_id: Some("rid-expired".to_string()),
            retrieval_log_ids: vec![999],
            created_at_ms: now_ms - 10_000,
            expires_at_ms: now_ms - 5_000, // expired
        };
        let expired_path = project_dir.join(format!("{}__{}.json", session_id, expired_uuid));
        std::fs::write(&expired_path, serde_json::to_string(&expired).unwrap()).unwrap();

        // Write a live pin file
        let live_uuid = "live-pin-uuid-0002";
        let live = TaskAnchor {
            session_id: session_id.to_string(),
            org_id: "org".to_string(),
            project_id: "proj".to_string(),
            retrieval_id: Some("rid-live".to_string()),
            retrieval_log_ids: vec![42],
            created_at_ms: now_ms,
            expires_at_ms: now_ms + DEFAULT_ANCHOR_TTL_MS,
        };
        let live_path = project_dir.join(format!("{}__{}.json", session_id, live_uuid));
        std::fs::write(&live_path, serde_json::to_string(&live).unwrap()).unwrap();

        let ts = load_task_session(
            "org",
            "proj",
            session_id,
            Some(TaskSessionOptions {
                sessions_dir: Some(tmp.path().to_path_buf()),
                ..Default::default()
            }),
        );
        let view = ts.read_f080();

        // Only the live pin should contribute
        assert_eq!(view.retrieval_id, Some("rid-live".to_string()));
        assert_eq!(view.applied_log_ids, vec![42]);

        // Expired pin file must be gone (best-effort delete)
        assert!(
            !expired_path.exists(),
            "expired pin must be deleted by read_f080"
        );
        // Live pin file must still exist
        assert!(
            live_path.exists(),
            "live pin must NOT be deleted by read_f080"
        );
    }
}
