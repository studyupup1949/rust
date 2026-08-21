//! RFC 103 §5.4 — the cross-surface guard (S1, layer 1: function level).
//!
//! One adversarial `AuditResult`, populated with an untrusted value in every
//! RFC 103 §4 field, rendered through every surface this crate defines
//! (Markdown, HTML, JSON, SARIF), asserting encoding on every field and
//! masking on the maskable ones. CSV/TSV and the proof that a real CLI
//! invocation actually enables masking live in layer 2, in
//! `crates/aaai-cli/tests/cli.rs` — `export.rs` and `report.rs`'s wiring are
//! not reachable from this crate.
//!
//! Written before any of RFC 103's fixes: expected red. Do not weaken an
//! assertion to make it pass.

use super::*;
use crate::audit::result::{AuditResult, AuditStatus, FileAuditResult};
use crate::config::definition::{AuditEntry, AuditStrategy, RegexTarget};
use crate::diff::entry::{DiffEntry, DiffType};
use crate::masking::engine::MaskingEngine;
use std::path::Path;

/// A value matching the built-in "AWS access key" pattern
/// (`crates/aaai/src/masking/patterns.rs`) — self-contained, no surrounding
/// punctuation required, safe to embed inside other adversarial payloads.
pub(super) const CANARY: &str = "AKIAAAAAAAAAAAAAAAAA";

/// Builds the one `AuditResult` RFC 103 §5.4 requires: an adversarial value
/// in every §4 field.
pub(super) fn adversarial_result() -> AuditResult {
    let diff = DiffEntry {
        // §4 `path` — a formula-injection payload and an unescaped pipe.
        // Encode-only: this must appear verbatim (once encoded) in every
        // surface, never masked — masking a path corrupts the audit's
        // identifier. Deliberately does NOT contain CANARY: path must
        // legitimately survive every assertion below that requires CANARY
        // to disappear.
        path: "=cmd|'/c calc'!A1".to_string(),
        diff_type: DiffType::Modified,
        is_dir: false,
        before_text: None,
        after_text: None,
        is_binary: false,
        before_size: None,
        after_size: None,
        before_sha256: None,
        after_sha256: None,
        stats: None,
        // §4 `error_detail` — may embed a path; contains `<` and `|`.
        // Not currently rendered by any surface; carried here so the guard
        // fails the moment a future change starts displaying it unmasked.
        error_detail: Some(format!("blocked <script>a|b {CANARY}")),
    };

    let entry = AuditEntry {
        path: diff.path.clone(),
        diff_type: DiffType::Modified,
        // §4 `reason` — human-written free text that may paste a secret.
        reason: format!("legitimate-looking review note {CANARY}"),
        // §4 "strategy rule content" — user-supplied pattern. Not currently
        // rendered by any surface (only `.label()`, the fixed variant name,
        // is); carried here for the same forward-looking reason as
        // `error_detail`.
        strategy: AuditStrategy::Regex {
            pattern: format!("secret-pattern-{CANARY}"),
            target: RegexTarget::default(),
        },
        enabled: true,
        // §4 `ticket` — free-form, unvalidated; F2's HTML XSS payload plus
        // the mask canary, so one field exercises both obligations.
        ticket: Some(format!("<img src=x onerror=alert(1)>{CANARY}")),
        approved_by: None,
        approved_at: None,
        expires_at: None,
        note: None,
        created_at: None,
        updated_at: None,
    };

    let file_result = FileAuditResult {
        diff,
        entry: Some(entry),
        status: AuditStatus::Failed,
        // Rendered today by Markdown and SARIF, unmasked. Not named in §4
        // as its own row, but it is free-text audit-engine output reaching
        // the same surfaces `reason` does, so it is held to the same bar.
        detail: Some(format!("rule mismatch, see {CANARY}")),
        warnings: Vec::new(),
    };

    AuditResult::new(vec![file_result])
}

/// §4 root paths — may embed usernames or hostnames; contain `"` and a
/// leading `-`, plus the mask canary (root paths must be masked, per §4,
/// and are not today on any surface).
pub(super) fn adversarial_root(label: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("-weird\"root-{label}-{CANARY}"))
}

fn engine() -> MaskingEngine {
    MaskingEngine::builtin()
}

#[test]
fn every_surface_encodes_and_masks_every_untrusted_field() {
    let result = adversarial_result();
    let before_root = adversarial_root("before");
    let after_root = adversarial_root("after");
    let definition_path = adversarial_root("definition");
    let masker = engine();
    let masking = crate::masking::Masking::Enabled(&masker);

    // ── Markdown ──────────────────────────────────────────────────────
    let md = ReportGenerator::build_markdown_string(
        &result,
        &before_root,
        &after_root,
        Some(Path::new(&definition_path)),
        masking,
        false,
    );
    assert!(
        !md.contains(CANARY),
        "Markdown must not disclose the mask canary anywhere:\n{md}"
    );
    assert!(
        md.contains("=cmd|'/c calc'!A1"),
        "Markdown must still show the (encode-only) path verbatim"
    );

    // ── HTML ──────────────────────────────────────────────────────────
    let html = crate::report::html::build_html(
        &result,
        &before_root,
        &after_root,
        Some(Path::new(&definition_path)),
        masking,
    );
    assert!(
        !html.contains(CANARY),
        "HTML must not disclose the mask canary anywhere:\n{html}"
    );
    assert!(
        !html.contains("<img src=x onerror=alert(1)>"),
        "HTML must not contain the raw, unescaped ticket XSS payload (F2)"
    );
    assert!(
        !html.contains("<script>"),
        "HTML must not contain an unescaped '<script>' from field content"
    );

    // ── JSON ──────────────────────────────────────────────────────────
    let json = ReportGenerator::build_json(
        &result,
        &before_root,
        &after_root,
        Some(Path::new(&definition_path)),
        masking,
    )
    .expect("build_json must succeed");
    assert!(
        !json.contains(CANARY),
        "JSON must not disclose the mask canary anywhere (F5):\n{json}"
    );

    // ── SARIF ─────────────────────────────────────────────────────────
    // Field-targeted, not a blanket text search: `originalUriBaseIds` and
    // each result's `artifactLocation.uri` legitimately embed the root path
    // unmasked, the same reason `path` itself is exempt — they are
    // navigation targets a SARIF consumer resolves back to a real file, and
    // masking them would break that. `message`, `ticket`, and the
    // run-level `properties.before`/`after` are the fields actually
    // required to be masked (F3).
    let sarif = crate::report::sarif::build_sarif(&result, &before_root, &after_root, masking);
    let run = &sarif["runs"][0];
    let message = run["results"][0]["message"]["text"].as_str().unwrap();
    let ticket = run["results"][0]["properties"]["ticket"].as_str().unwrap();
    let props_before = run["properties"]["before"].as_str().unwrap();
    let props_after = run["properties"]["after"].as_str().unwrap();
    assert!(!message.contains(CANARY), "SARIF message must be masked (F3): {message}");
    assert!(!ticket.contains(CANARY), "SARIF properties.ticket must be masked (F3): {ticket}");
    assert!(!props_before.contains(CANARY), "SARIF properties.before must be masked: {props_before}");
    assert!(!props_after.contains(CANARY), "SARIF properties.after must be masked: {props_after}");
}
