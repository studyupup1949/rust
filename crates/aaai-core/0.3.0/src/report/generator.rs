//! Report generation — Markdown and JSON output.

use std::path::Path;
use chrono::Local;

use crate::audit::result::{AuditResult, AuditStatus};

pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate a Markdown report and write to `output_path`.
    pub fn write_markdown(
        result: &AuditResult,
        before_root: &Path,
        after_root: &Path,
        definition_path: Option<&Path>,
        output_path: &Path,
    ) -> anyhow::Result<()> {
        let md = Self::build_markdown(result, before_root, after_root, definition_path);
        std::fs::write(output_path, md.as_bytes())?;
        log::info!("Markdown report written to {}", output_path.display());
        Ok(())
    }

    /// Generate a JSON report and write to `output_path`.
    pub fn write_json(
        result: &AuditResult,
        before_root: &Path,
        after_root: &Path,
        definition_path: Option<&Path>,
        output_path: &Path,
    ) -> anyhow::Result<()> {
        let json = Self::build_json(result, before_root, after_root, definition_path)?;
        std::fs::write(output_path, json.as_bytes())?;
        log::info!("JSON report written to {}", output_path.display());
        Ok(())
    }

    fn build_markdown(
        result: &AuditResult,
        before_root: &Path,
        after_root: &Path,
        definition_path: Option<&Path>,
    ) -> String {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S %Z").to_string();
        let s = &result.summary;
        let verdict = if s.is_passing() { "PASSED" } else { "FAILED" };

        let mut md = String::new();
        md.push_str("# aaai Audit Report\n\n");
        md.push_str(&format!("**Result: {verdict}**\n\n"));
        md.push_str("## Summary\n\n");
        md.push_str(&format!("| Item | Count |\n|------|-------|\n"));
        md.push_str(&format!("| Total | {} |\n", s.total));
        md.push_str(&format!("| OK | {} |\n", s.ok));
        md.push_str(&format!("| Pending | {} |\n", s.pending));
        md.push_str(&format!("| Failed | {} |\n", s.failed));
        md.push_str(&format!("| Ignored | {} |\n", s.ignored));
        md.push_str(&format!("| Error | {} |\n", s.error));
        md.push('\n');

        md.push_str("## Execution Details\n\n");
        md.push_str(&format!("- **Run at:** {now}\n"));
        md.push_str(&format!("- **Before:** `{}`\n", before_root.display()));
        md.push_str(&format!("- **After:** `{}`\n", after_root.display()));
        if let Some(dp) = definition_path {
            md.push_str(&format!("- **Definition:** `{}`\n", dp.display()));
        }
        md.push('\n');

        // Issues first
        for status_header in [
            (AuditStatus::Failed, "## Failed Entries"),
            (AuditStatus::Pending, "## Pending Entries"),
            (AuditStatus::Error, "## Error Entries"),
            (AuditStatus::Ok, "## OK Entries"),
            (AuditStatus::Ignored, "## Ignored Entries"),
        ] {
            let (status, header) = status_header;
            let entries: Vec<_> = result.results.iter()
                .filter(|r| r.status == status)
                .collect();
            if entries.is_empty() { continue; }
            md.push_str(&format!("{header}\n\n"));
            for r in &entries {
                md.push_str(&format!("### `{}`\n\n", r.diff.path));
                md.push_str(&format!("- **Status:** {}\n", r.status));
                md.push_str(&format!("- **Diff type:** {}\n", r.diff.diff_type));
                if let Some(entry) = &r.entry {
                    md.push_str(&format!("- **Reason:** {}\n", entry.reason));
                    md.push_str(&format!("- **Strategy:** {}\n", entry.strategy.label()));
                    if let Some(note) = &entry.note {
                        md.push_str(&format!("- **Note:** {note}\n"));
                    }
                }
                if let Some(detail) = &r.detail {
                    md.push_str(&format!("- **Detail:** {detail}\n"));
                }
                md.push('\n');
            }
        }

        md
    }

    fn build_json(
        result: &AuditResult,
        before_root: &Path,
        after_root: &Path,
        definition_path: Option<&Path>,
    ) -> anyhow::Result<String> {
        use serde_json::{json, Value};
        let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let s = &result.summary;

        let entries: Vec<Value> = result.results.iter().map(|r| {
            json!({
                "path": r.diff.path,
                "diff_type": r.diff.diff_type.to_string(),
                "status": r.status.to_string(),
                "reason": r.entry.as_ref().map(|e| &e.reason),
                "strategy": r.entry.as_ref().map(|e| e.strategy.label()),
                "detail": r.detail,
            })
        }).collect();

        let doc = json!({
            "app": "aaai",
            "run_at": now,
            "before": before_root.display().to_string(),
            "after": after_root.display().to_string(),
            "definition": definition_path.map(|p| p.display().to_string()),
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
