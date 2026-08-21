//! Built-in rule templates for common audit patterns.
//!
//! A [`RuleTemplate`] carries a display name, description, and a factory that
//! produces a pre-filled [`AuditStrategy`] ready to insert into an inspector
//! or a snap-generated entry.

use crate::config::definition::{AuditStrategy, LineAction, LineRule, RegexTarget};
use crate::diff::entry::DiffType;

/// A named, reusable audit-strategy pattern.
#[derive(Debug, Clone)]
pub struct RuleTemplate {
    pub id:          &'static str,
    pub name:        &'static str,
    pub name_ja:     &'static str,
    pub description: &'static str,
    pub diff_type:   DiffType,
    /// Produce a strategy instance for this template.
    pub strategy:    fn() -> AuditStrategy,
}

/// The built-in template library.
pub static TEMPLATES: &[RuleTemplate] = &[
    RuleTemplate {
        id:          "version_bump",
        name:        "Version number update",
        name_ja:     "バージョン番号の更新",
        description: "A semver-like version string changed in a modified file (Regex).",
        diff_type:   DiffType::Modified,
        strategy:    || AuditStrategy::Regex {
            pattern: r"^\d+\.\d+\.\d+".to_string(),
            target:  RegexTarget::AddedLines,
        },
    },
    RuleTemplate {
        id:          "port_change",
        name:        "Port number change",
        name_ja:     "ポート番号の変更",
        description: "A `port = N` line changed to `port = M` (LineMatch template — fill in actual values).",
        diff_type:   DiffType::Modified,
        strategy:    || AuditStrategy::LineMatch {
            rules: vec![
                LineRule { action: LineAction::Removed, line: "port = ".to_string() },
                LineRule { action: LineAction::Added,   line: "port = ".to_string() },
            ],
        },
    },
    RuleTemplate {
        id:          "file_added_any",
        name:        "File added (content not inspected)",
        name_ja:     "ファイルの追加（内容確認なし）",
        description: "A new file was intentionally added; no content check required.",
        diff_type:   DiffType::Added,
        strategy:    || AuditStrategy::None,
    },
    RuleTemplate {
        id:          "file_removed_any",
        name:        "File removed (content not inspected)",
        name_ja:     "ファイルの削除（内容確認なし）",
        description: "A file was intentionally deleted; no content check required.",
        diff_type:   DiffType::Removed,
        strategy:    || AuditStrategy::None,
    },
    RuleTemplate {
        id:          "config_line_change",
        name:        "Config key=value change",
        name_ja:     "設定値の変更",
        description: "A `key = value` pair changed (LineMatch template — fill in key and values).",
        diff_type:   DiffType::Modified,
        strategy:    || AuditStrategy::LineMatch {
            rules: vec![
                LineRule { action: LineAction::Removed, line: "key = old_value".to_string() },
                LineRule { action: LineAction::Added,   line: "key = new_value".to_string() },
            ],
        },
    },
    RuleTemplate {
        id:          "date_string_change",
        name:        "Date string update",
        name_ja:     "日付文字列の更新",
        description: "A date like 2025-01-15 changed; validates ISO-8601 format (Regex).",
        diff_type:   DiffType::Modified,
        strategy:    || AuditStrategy::Regex {
            pattern: r"^\d{4}-\d{2}-\d{2}".to_string(),
            target:  RegexTarget::AddedLines,
        },
    },
    RuleTemplate {
        id:          "exact_binary",
        name:        "Binary / generated file (checksum)",
        name_ja:     "バイナリ・生成ファイル（チェックサム）",
        description: "Verify a binary or generated file by its SHA-256 digest (fill in hash).",
        diff_type:   DiffType::Modified,
        strategy:    || AuditStrategy::Checksum {
            expected_sha256: String::new(),
        },
    },
    RuleTemplate {
        id:          "feature_flag_toggle",
        name:        "Feature flag toggle",
        name_ja:     "フィーチャーフラグの切り替え",
        description: "A boolean flag changed from false to true or vice versa (Regex).",
        diff_type:   DiffType::Modified,
        strategy:    || AuditStrategy::Regex {
            pattern: r"^(true|false)$".to_string(),
            target:  RegexTarget::AddedLines,
        },
    },
];

/// Find a template by id.
pub fn find(id: &str) -> Option<&'static RuleTemplate> {
    TEMPLATES.iter().find(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_templates_produce_valid_strategies() {
        for tmpl in TEMPLATES {
            let strat = (tmpl.strategy)();
            // None and LineMatch with placeholder are valid (or expected to need filling)
            // Just ensure no panics
            let _ = strat.label();
        }
    }

    #[test]
    fn find_by_id_works() {
        assert!(find("version_bump").is_some());
        assert!(find("nonexistent").is_none());
    }

    #[test]
    fn no_duplicate_ids() {
        let mut ids: Vec<&str> = TEMPLATES.iter().map(|t| t.id).collect();
        let original_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "duplicate template IDs found");
    }
}
