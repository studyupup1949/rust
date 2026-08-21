//! HTML report generation.

use std::path::Path;

use crate::audit::result::{AuditResult, AuditStatus};
use crate::masking::engine::MaskingEngine;

pub fn build_html(
    result: &AuditResult,
    before_root: &Path,
    after_root: &Path,
    definition_path: Option<&Path>,
    masker: Option<&MaskingEngine>,
) -> String {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let s = &result.summary;
    let verdict = if s.is_passing() { "PASSED" } else { "FAILED" };
    let verdict_color = if s.is_passing() { "#2d7a2d" } else { "#a01010" };

    let mut rows = String::new();
    for r in &result.results {
        if r.diff.diff_type == crate::diff::entry::DiffType::Unchanged { continue; }
        let status_bg = match r.status {
            AuditStatus::Ok      => "#d4edda",
            AuditStatus::Pending => "#fff3cd",
            AuditStatus::Failed  => "#f8d7da",
            AuditStatus::Error   => "#e2cbf7",
            AuditStatus::Ignored => "#e9ecef",
        };
        let reason = r.entry.as_ref().map(|e| {
            masker.map(|m| m.mask(&e.reason)).unwrap_or_else(|| e.reason.clone())
        }).unwrap_or_default();
        let ticket = r.entry.as_ref().and_then(|e| e.ticket.as_ref())
            .map(|t| format!("<span class=\"ticket\">[{t}]</span> "))
            .unwrap_or_default();
        let binary_badge = if r.diff.is_binary { "<span class=\"badge binary\">binary</span>" } else { "" };
        let stats_txt = r.diff.stats.as_ref()
            .map(|st| format!("<span class=\"stats\">+{} −{}</span>", st.lines_added, st.lines_removed))
            .unwrap_or_default();
        let size_txt = r.diff.size_change_label()
            .map(|l| format!("<span class=\"size\">{l}</span>"))
            .unwrap_or_default();
        rows.push_str(&format!(
            r#"<tr style="background:{status_bg}">
  <td><code>{path}</code> {binary_badge}</td>
  <td>{dtype}</td>
  <td><strong>{status}</strong></td>
  <td>{ticket}{reason}</td>
  <td>{stats_txt} {size_txt}</td>
</tr>"#,
            path   = html_escape(&r.diff.path),
            dtype  = r.diff.diff_type,
            status = r.status,
            reason = html_escape(&reason),
        ));
    }

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>aaai Audit Report — {verdict}</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
         margin: 0; padding: 24px; color: #1a1a2e; background: #f8f9fa; }}
  h1   {{ font-size: 1.6rem; margin-bottom: 4px; }}
  .verdict {{ display: inline-block; padding: 6px 18px; border-radius: 6px;
              color: #fff; background: {verdict_color}; font-weight: 700;
              font-size: 1.1rem; margin-bottom: 16px; }}
  .meta  {{ font-size: 0.85rem; color: #555; margin-bottom: 20px; }}
  .cards {{ display: flex; gap: 12px; flex-wrap: wrap; margin-bottom: 24px; }}
  .card  {{ background: #fff; border-radius: 8px; padding: 16px 24px;
            box-shadow: 0 1px 4px rgba(0,0,0,.1); min-width: 100px; text-align: center; }}
  .card .num {{ font-size: 2rem; font-weight: 700; }}
  .card .lbl {{ font-size: 0.8rem; color: #777; text-transform: uppercase; letter-spacing: .05em; }}
  .ok   {{ color: #2d7a2d; }} .pending {{ color: #856404; }}
  .failed {{ color: #721c24; }} .error {{ color: #5a1a6b; }}
  table  {{ width: 100%; border-collapse: collapse; background: #fff;
            border-radius: 8px; overflow: hidden; box-shadow: 0 1px 4px rgba(0,0,0,.1); }}
  th     {{ background: #343a40; color: #fff; padding: 10px 14px;
            text-align: left; font-size: 0.85rem; }}
  td     {{ padding: 9px 14px; font-size: 0.85rem; border-bottom: 1px solid #eee; }}
  code   {{ font-family: "SFMono-Regular", Consolas, monospace; font-size: 0.8rem; }}
  .ticket {{ background: #e8f0fe; color: #1a56db; border-radius: 4px;
             padding: 1px 6px; font-size: 0.78rem; font-weight: 600; }}
  .badge.binary {{ background: #6c757d; color: #fff; border-radius: 4px;
                   padding: 1px 5px; font-size: 0.72rem; }}
  .stats {{ color: #555; font-size: 0.78rem; }}
  .size  {{ color: #888; font-size: 0.78rem; }}
</style>
</head>
<body>
<h1>aaai Audit Report</h1>
<div class="verdict">{verdict}</div>
<div class="meta">
  Run at: <strong>{now}</strong> &nbsp;·&nbsp;
  Before: <code>{before}</code> &nbsp;·&nbsp;
  After: <code>{after}</code>
  {def_line}
</div>
<div class="cards">
  <div class="card"><div class="num ok">{ok}</div><div class="lbl">OK</div></div>
  <div class="card"><div class="num pending">{pending}</div><div class="lbl">Pending</div></div>
  <div class="card"><div class="num failed">{failed}</div><div class="lbl">Failed</div></div>
  <div class="card"><div class="num error">{error}</div><div class="lbl">Error</div></div>
  <div class="card"><div class="num">{total}</div><div class="lbl">Total</div></div>
</div>
<table>
<thead><tr><th>Path</th><th>Diff type</th><th>Status</th><th>Reason</th><th>Stats</th></tr></thead>
<tbody>
{rows}
</tbody>
</table>
<p style="margin-top:20px;font-size:0.75rem;color:#aaa">Generated by aaai v0.5.0</p>
</body></html>"#,
        verdict_color = verdict_color,
        before   = html_escape(&before_root.display().to_string()),
        after    = html_escape(&after_root.display().to_string()),
        def_line = definition_path.map(|p| format!(
            "&nbsp;·&nbsp; Definition: <code>{}</code>", html_escape(&p.display().to_string())
        )).unwrap_or_default(),
        ok = s.ok, pending = s.pending, failed = s.failed,
        error = s.error, total = s.total,
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}
