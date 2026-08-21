//! Content-audit strategy execution.


use similar::{ChangeTag, TextDiff};

use crate::config::definition::{AuditStrategy, LineAction, RegexTarget};
use crate::diff::entry::DiffEntry;

/// Execute the content-audit strategy and return Ok(()) or Err(detail).
pub fn evaluate(strategy: &AuditStrategy, diff: &DiffEntry) -> Result<(), String> {
    match strategy {
        AuditStrategy::None => Ok(()),

        AuditStrategy::Checksum { expected_sha256 } => {
            match &diff.after_sha256 {
                Some(actual) => {
                    if actual.to_lowercase() == expected_sha256.to_lowercase() {
                        Ok(())
                    } else {
                        Err(format!(
                            "Checksum mismatch.\n  Expected: {expected_sha256}\n  Actual:   {actual}"
                        ))
                    }
                }
                None => Err(
                    "Checksum: cannot compute digest — file is not readable as bytes.".into()
                ),
            }
        }

        AuditStrategy::LineMatch { rules } => {
            let before = diff.before_text.as_deref().unwrap_or("");
            let after = diff.after_text.as_deref().unwrap_or("");
            let text_diff = TextDiff::from_lines(before, after);

            let mut added_lines: Vec<String> = Vec::new();
            let mut removed_lines: Vec<String> = Vec::new();

            for change in text_diff.iter_all_changes() {
                let line = change.value().trim_end_matches('\n').to_string();
                match change.tag() {
                    ChangeTag::Insert => added_lines.push(line),
                    ChangeTag::Delete => removed_lines.push(line),
                    ChangeTag::Equal => {}
                }
            }

            let mut missing: Vec<String> = Vec::new();
            for rule in rules {
                let found = match rule.action {
                    LineAction::Added => added_lines.contains(&rule.line),
                    LineAction::Removed => removed_lines.contains(&rule.line),
                };
                if !found {
                    missing.push(format!(
                        "  {} {:?}  (not found in diff)",
                        rule.action, rule.line
                    ));
                }
            }

            if missing.is_empty() {
                Ok(())
            } else {
                Err(format!("LineMatch: expected lines not found:\n{}", missing.join("\n")))
            }
        }

        AuditStrategy::Regex { pattern, target } => {
            let re = regex::Regex::new(pattern)
                .map_err(|e| format!("Regex: invalid pattern — {e}"))?;

            let before = diff.before_text.as_deref().unwrap_or("");
            let after = diff.after_text.as_deref().unwrap_or("");
            let text_diff = TextDiff::from_lines(before, after);

            let mut to_check: Vec<String> = Vec::new();
            for change in text_diff.iter_all_changes() {
                let line = change.value().trim_end_matches('\n').to_string();
                let include = match target {
                    RegexTarget::AddedLines => change.tag() == ChangeTag::Insert,
                    RegexTarget::RemovedLines => change.tag() == ChangeTag::Delete,
                    RegexTarget::AllChangedLines => {
                        change.tag() == ChangeTag::Insert
                            || change.tag() == ChangeTag::Delete
                    }
                };
                if include {
                    to_check.push(line);
                }
            }

            if to_check.is_empty() {
                return Err("Regex: no changed lines to match against.".into());
            }

            let mut failures: Vec<String> = Vec::new();
            for line in &to_check {
                if !re.is_match(line) {
                    failures.push(format!("  Line does not match pattern: {line:?}"));
                }
            }

            if failures.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "Regex: pattern {:?} did not match:\n{}",
                    pattern,
                    failures.join("\n")
                ))
            }
        }

        AuditStrategy::Exact { expected_content } => {
            match &diff.after_text {
                Some(actual) => {
                    if actual == expected_content {
                        Ok(())
                    } else {
                        Err("Exact: file content does not match the expected content.".into())
                    }
                }
                None => Err(
                    "Exact: cannot read file as text — binary file or encoding error.".into()
                ),
            }
        }
    }
}
