//! Report generation — Markdown and JSON output.

use super::html::build_html;
use std::path::Path;
use chrono::Local;

use crate::audit::result::{AuditResult, AuditStatus};
use crate::masking::Masking;

pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate a Markdown report and write to `output_path`.
    pub fn write_markdown(
        result: &AuditResult,
        before_root: &Path,
        after_root: &Path,
        definition_path: Option<&Path>,
        output_path: &Path,
        masking: Masking<'_>,
    ) -> anyhow::Result<()> {
        let md = Self::build_markdown(result, before_root, after_root, definition_path, masking);
        // (include_diff variant available via build_markdown_string)
        std::fs::write(output_path, md.as_bytes())?;
        log::info!("Markdown report written to {}", output_path.display());
        Ok(())
    }

    /// Generate a SARIF v2.1.0 report for CI/CD annotation systems.
    pub fn write_sarif(
        result: &AuditResult,
        before_root: &Path,
        after_root: &Path,
        output_path: &Path,
        masking: Masking<'_>,
    ) -> anyhow::Result<()> {
        let sarif = crate::report::sarif::build_sarif(result, before_root, after_root, masking);
        let json = serde_json::to_string_pretty(&sarif)?;
        std::fs::write(output_path, json.as_bytes())?;
        log::info!("SARIF report written to {}", output_path.display());
        Ok(())
    }

    /// Generate a JSON report and write to `output_path`.
    pub fn write_json(
        result: &AuditResult,
        before_root: &Path,
        after_root: &Path,
        definition_path: Option<&Path>,
        output_path: &Path,
        masking: Masking<'_>,
    ) -> anyhow::Result<()> {
        let json = Self::build_json(result, before_root, after_root, definition_path, masking)?;
        std::fs::write(output_path, json.as_bytes())?;
        log::info!("JSON report written to {}", output_path.display());
        Ok(())
    }

    /// Build a Markdown report string.
    /// `include_diff`: embed actual diff text for Modified entries.
    pub fn build_markdown_string(
        result: &AuditResult,
        before_root: &Path,
        after_root: &Path,
        definition_path: Option<&Path>,
        masking: Masking<'_>,
        include_diff: bool,
    ) -> String {
        let mut md = Self::build_markdown(result, before_root, after_root, definition_path, masking);
        if include_diff {
            md.push_str("
## Diff Details

");
            for r in &result.results {
                if r.diff.diff_type != crate::diff::entry::DiffType::Modified { continue; }
                if r.diff.is_binary { continue; }
                let before = r.diff.before_text.as_deref().unwrap_or("");
                let after  = r.diff.after_text.as_deref().unwrap_or("");
                if before == after { continue; }
                md.push_str(&format!("### `{}`

```diff
", r.diff.path));
                use similar::{ChangeTag, TextDiff};
                let td = TextDiff::from_lines(before, after);
                for change in td.iter_all_changes() {
                    let prefix = match change.tag() {
                        ChangeTag::Insert => "+",
                        ChangeTag::Delete => "-",
                        ChangeTag::Equal  => " ",
                    };
                    let line = change.value().trim_end_matches('\n');
                    md.push_str(&format!("{prefix}{}
", masking.mask(line)));
                }
                md.push_str("```

");
            }
        }
        md
    }

    fn build_markdown(
        result: &AuditResult,
        before_root: &Path,
        after_root: &Path,
        definition_path: Option<&Path>,
        masking: Masking<'_>,
    ) -> String {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S %Z").to_string();
        let s = &result.summary;
        let (verdict_sym, verdict_word) =
            if s.is_passing() { ("✓", "PASSED") } else { ("✗", "FAILED") };

        let mut md = String::new();
        // ── Zone 1: Header ───────────────────────────────────────────
        md.push_str("# aaai Audit Report\n\n");
        md.push_str(&format!("**Result: {verdict_sym} {verdict_word}**\n\n"));

        // ── Zone 2: Summary (issues-first column order) ──────────────
        md.push_str("## Summary\n\n");
        md.push_str("| Status | Count |\n|---|---:|\n");
        if s.failed  > 0 { md.push_str(&format!("| ✗ Failed  | {} |\n", s.failed)); }
        if s.pending > 0 { md.push_str(&format!("| ⚠ Pending | {} |\n", s.pending)); }
        if s.error   > 0 { md.push_str(&format!("| ! Error   | {} |\n", s.error)); }
        md.push_str(&format!("| ✓ OK      | {} |\n", s.ok));
        if s.ignored > 0 { md.push_str(&format!("| — Ignored | {} |\n", s.ignored)); }
        md.push_str(&format!("| **Total** | **{}** |\n", s.total));
        md.push('\n');

        // ── Zone 3: Execution Details ────────────────────────────────
        // §4 root paths — masked, matching every other free-text field.
        md.push_str("## Execution Details\n\n");
        md.push_str(&format!("- **Run at:** {now}\n"));
        md.push_str(&format!("- **Before:** `{}`\n", masking.mask(&before_root.display().to_string())));
        md.push_str(&format!("- **After:** `{}`\n", masking.mask(&after_root.display().to_string())));
        if let Some(dp) = definition_path {
            md.push_str(&format!("- **Definition:** `{}`\n", masking.mask(&dp.display().to_string())));
        }
        md.push('\n');

        // ── Zone 4: Action Required (Failed + Pending + Error) ───────
        let attention: Vec<_> = result.results.iter()
            .filter(|r| matches!(r.status,
                AuditStatus::Failed | AuditStatus::Pending | AuditStatus::Error))
            .collect();
        if !attention.is_empty() {
            let counts = format!("Failed: {}, Pending: {}, Error: {}",
                s.failed, s.pending, s.error);
            md.push_str(&format!("## ⚠ Action Required ({counts})\n\n"));
            for r in &attention {
                Self::md_entry(&mut md, r, masking);
            }
        }

        // ── Zone 5: Passed entries ───────────────────────────────────
        let ok_entries: Vec<_> = result.results.iter()
            .filter(|r| r.status == AuditStatus::Ok)
            .collect();
        if !ok_entries.is_empty() {
            md.push_str(&format!("## ✓ Passed Entries ({})\n\n", ok_entries.len()));
            for r in &ok_entries {
                Self::md_entry(&mut md, r, masking);
            }
        }

        // ── Zone 6: Ignored entries ──────────────────────────────────
        let ignored: Vec<_> = result.results.iter()
            .filter(|r| r.status == AuditStatus::Ignored)
            .collect();
        if !ignored.is_empty() {
            md.push_str(&format!("## — Ignored Entries ({})\n\n", ignored.len()));
            for r in &ignored {
                Self::md_entry(&mut md, r, masking);
            }
        }

        md
    }

    fn md_entry(
        md: &mut String,
        r: &crate::audit::result::FileAuditResult,
        masking: Masking<'_>,
    ) {
        let sym = match r.status {
            AuditStatus::Ok      => "✓",
            AuditStatus::Pending => "⚠",
            AuditStatus::Failed  => "✗",
            AuditStatus::Error   => "!",
            AuditStatus::Ignored => "—",
        };
        md.push_str(&format!("### `{}` — {} {}\n\n", r.diff.path, sym, r.status));
        md.push_str(&format!("- **Diff type:** {}\n", r.diff.diff_type));

        if let Some(entry) = &r.entry {
            let raw_reason = entry.reason.trim();
            let reason = if raw_reason.is_empty() {
                "*(no reason provided)*".to_string()
            } else {
                masking.mask(raw_reason)
            };
            md.push_str(&format!("- **Reason:** {}\n", reason));
            md.push_str(&format!("- **Strategy:** {}\n", entry.strategy.label()));
            if let Some(t)  = &entry.ticket      { md.push_str(&format!("- **Ticket:** {}\n", masking.mask(t))); }
            if let Some(ab) = &entry.approved_by { md.push_str(&format!("- **Approved by:** {ab}\n")); }
            if let Some(at) = &entry.approved_at {
                md.push_str(&format!("- **Approved at:** {}\n", at.format("%Y-%m-%d %H:%M UTC")));
            }
            if let Some(exp) = &entry.expires_at { md.push_str(&format!("- **Expires:** {exp}\n")); }
            if let Some(note) = &entry.note      { md.push_str(&format!("- **Note:** {note}\n")); }
        }
        if r.diff.is_binary { md.push_str("- **Type:** Binary file\n"); }
        if let Some(label) = r.diff.size_change_label() {
            md.push_str(&format!("- **Size:** {label}\n"));
        }
        if let Some(stats) = &r.diff.stats {
            md.push_str(&format!("- **Lines:** +{} −{}\n",
                stats.lines_added, stats.lines_removed));
        }
        // Audit check detail — shown as blockquote for visibility. Masked:
        // engine-generated free text that is rendered the same way `reason`
        // is, and is not held to a lower bar just because it is not §4's
        // own named field.
        if let Some(detail) = &r.detail {
            md.push_str(&format!("\n> {sym} {}\n", masking.mask(detail)));
        }
        md.push('\n');
    }

    /// Generate an HTML report and write to `output_path`.
    pub fn write_html(
        result: &AuditResult,
        before_root: &Path,
        after_root: &Path,
        definition_path: Option<&Path>,
        output_path: &Path,
        masking: Masking<'_>,
    ) -> anyhow::Result<()> {
        let html = build_html(result, before_root, after_root, definition_path, masking);
        std::fs::write(output_path, html.as_bytes())?;
        log::info!("HTML report written to {}", output_path.display());
        Ok(())
    }

    fn build_json(
        result: &AuditResult,
        before_root: &Path,
        after_root: &Path,
        definition_path: Option<&Path>,
        masking: Masking<'_>,
    ) -> anyhow::Result<String> {
        use serde_json::{json, Value};
        let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let s = &result.summary;

        let entries: Vec<Value> = result.results.iter().map(|r| {
            json!({
                "path": r.diff.path,
                "diff_type": r.diff.diff_type.to_string(),
                "status": r.status.to_string(),
                // F5 — this used to ignore its masker entirely.
                "reason": r.entry.as_ref().map(|e| masking.mask(&e.reason)),
                "strategy": r.entry.as_ref().map(|e| e.strategy.label()),
                "detail": r.detail.as_ref().map(|d| masking.mask(d)),
            })
        }).collect();

        let doc = json!({
            "app": "aaai",
            "run_at": now,
            "before": masking.mask(&before_root.display().to_string()),
            "after": masking.mask(&after_root.display().to_string()),
            "definition": definition_path.map(|p| masking.mask(&p.display().to_string())),
            "result": if s.is_passing() { "PASSED" } else { "FAILED" },
            "summary": {
                "total": s.total,
                "ok": s.ok,
                "pending": s.pending,
                "failed": s.failed,
                "ignored": s.ignored,
                "error": s.error,
            },
            "entries": entries,
        });

        Ok(serde_json::to_string_pretty(&doc)?)
    }
}

#[cfg(test)]
mod tests;
