//! `acctl codegen-tags` — local regeneration of `www/src/AutoCoreTags.ts`
//! from `project.json` variables marked with `"ux": true`.
//!
//! # Output file layout
//!
//! The generated file has two arrays — `acTagSpecGenerated` (emitted by this
//! command, bracketed by sentinel comments) and `acTagSpecCustom` (hand-edited
//! overrides) — combined into the exported `acTagSpec`. On subsequent runs
//! **only the block between the sentinel comments is replaced**, preserving
//! anything the user has written in `acTagSpecCustom`.
//!
//! # Regeneration-from-scratch conditions
//!
//! The file is rewritten in its entirety (and the old contents saved to a
//! `.bak` sibling) when any of these are true:
//!
//! - `www/src/AutoCoreTags.ts` does not exist
//! - the file is missing either sentinel comment
//! - the file has no `acTagSpecCustom` declaration
//! - the caller passed `--force`

use anyhow::{anyhow, Context, Result};
use colored::*;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const GENERATED_BEGIN: &str = "// autocore-codegen:generated-start";
const GENERATED_END: &str = "// autocore-codegen:generated-end";
const TAGS_PATH: &str = "www/src/AutoCoreTags.ts";

pub fn cmd_codegen_tags(force: bool) -> Result<()> {
    let project_json_path = PathBuf::from("project.json");
    if !project_json_path.exists() {
        return Err(anyhow!(
            "project.json not found in current directory — run from an AutoCore project root"
        ));
    }

    let project_text = fs::read_to_string(&project_json_path).context("reading project.json")?;
    let project: Value =
        serde_json::from_str(&project_text).context("parsing project.json")?;

    let tags = collect_tags(&project);
    println!(
        "{} from project.json variables with {}",
        "Collected".bold(),
        "\"ux\": true".cyan(),
    );
    println!("  {} tag{}", tags.len(), if tags.len() == 1 { "" } else { "s" });
    if tags.is_empty() {
        println!(
            "  {}",
            "(set \"ux\": true on variables you want exposed to the web UI)".dimmed(),
        );
    }

    let tags_path = PathBuf::from(TAGS_PATH);
    let generated_block = render_generated_block(&tags);

    let action = decide_action(&tags_path, force)?;
    match action {
        Action::ReplaceInPlace { existing } => {
            let new_contents = splice_generated_block(&existing, &generated_block)?;
            fs::write(&tags_path, new_contents)
                .with_context(|| format!("writing {}", tags_path.display()))?;
            println!(
                "  {} {}  (acTagSpecCustom preserved)",
                "updated".green(),
                tags_path.display(),
            );
        }
        Action::FullRegen { backed_up } => {
            if let Some(bak) = backed_up {
                println!(
                    "  {} {}",
                    "backed up".yellow(),
                    bak.display(),
                );
            }
            let full = render_full_file(&generated_block);
            if let Some(parent) = tags_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            fs::write(&tags_path, full)
                .with_context(|| format!("writing {}", tags_path.display()))?;
            println!("  {} {}", "wrote".green(), tags_path.display());
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tag collection
// ---------------------------------------------------------------------------

struct Tag {
    tag_name: String,
    fqdn: String,
    value_type: &'static str,
}

fn collect_tags(project: &Value) -> Vec<Tag> {
    let empty = serde_json::Map::new();
    let vars = project
        .get("variables")
        .and_then(|v| v.as_object())
        .unwrap_or(&empty);

    let mut tags: Vec<Tag> = Vec::new();
    for (name, cfg) in vars {
        let ux = cfg.get("ux").and_then(|v| v.as_bool()).unwrap_or(false);
        if !ux {
            continue;
        }
        let ty = cfg.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let value_type = match var_type_to_ts(ty) {
            Some(t) => t,
            None => {
                eprintln!(
                    "{}: skipping variable '{}': unsupported type '{}' for UI generation",
                    "warning".yellow(),
                    name,
                    ty,
                );
                continue;
            }
        };
        tags.push(Tag {
            tag_name: snake_to_camel(name),
            fqdn: format!("gm.{}", name),
            value_type,
        });
    }
    tags.sort_by(|a, b| a.tag_name.cmp(&b.tag_name));
    tags
}

fn var_type_to_ts(t: &str) -> Option<&'static str> {
    match t {
        "bool" => Some("boolean"),
        "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "f32" | "f64" => Some("number"),
        "string" => Some("string"),
        _ => None,
    }
}

/// snake_case → camelCase. `all_axes_homed` → `allAxesHomed`.
/// Preserves digits; collapses consecutive underscores.
fn snake_to_camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut capitalize_next = false;
    let mut first = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
            continue;
        }
        if first {
            out.extend(c.to_lowercase());
            first = false;
        } else if capitalize_next {
            out.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render_generated_block(tags: &[Tag]) -> String {
    let mut out = String::new();
    out.push_str(GENERATED_BEGIN);
    out.push('\n');
    out.push_str("// DO NOT EDIT: this block is regenerated by `acctl codegen-tags`.\n");
    out.push_str("export const acTagSpecGenerated = [\n");
    for t in tags {
        out.push_str(&format!(
            "    {{ \"tagName\": \"{}\", \"fqdn\": \"{}\", \"valueType\": \"{}\" }},\n",
            t.tag_name, t.fqdn, t.value_type,
        ));
    }
    out.push_str("] as const satisfies readonly TagConfig[];\n");
    out.push_str(GENERATED_END);
    out
}

/// Complete file for the from-scratch path.
fn render_full_file(generated_block: &str) -> String {
    const HEADER: &str = r#"/**
 * AutoCore Tag Definitions
 *
 * The exported `acTagSpec` is the union of two arrays:
 *   - `acTagSpecGenerated` — produced by `acctl codegen-tags` from every
 *     variable in `project.json` with `"ux": true`. Do NOT edit this block
 *     by hand; it will be overwritten on the next codegen run.
 *   - `acTagSpecCustom`    — hand-written tags and per-tag overrides
 *     (subscription cycle times, scales, etc.). Safe to edit; preserved
 *     across codegen runs as long as the sentinel comments stay intact.
 */

import type { TagConfig } from "@adcops/autocore-react/core/AutoCoreTagTypes";

"#;

    const FOOTER: &str = r#"

// Hand-written tags and overrides. Safe to edit.
//
// Example:
//   {
//     tagName: "liftPosition",
//     fqdn: "gm.lift_axis_position",
//     valueType: "number",
//     subscriptionOptions: { sampling_interval_ms: 300 },
//     scale: "position",
//   },
export const acTagSpecCustom = [
] as const satisfies readonly TagConfig[];

export const acTagSpec = [
    ...acTagSpecGenerated,
    ...acTagSpecCustom,
] as const satisfies readonly TagConfig[];

export default acTagSpec;
"#;

    let mut out = String::with_capacity(HEADER.len() + generated_block.len() + FOOTER.len());
    out.push_str(HEADER);
    out.push_str(generated_block);
    out.push_str(FOOTER);
    out
}

// ---------------------------------------------------------------------------
// File action decision + in-place splice
// ---------------------------------------------------------------------------

enum Action {
    /// Replace only the sentinel-bracketed block; acTagSpecCustom preserved.
    ReplaceInPlace { existing: String },
    /// Rewrite the whole file. If the old file existed, it was backed up.
    FullRegen { backed_up: Option<PathBuf> },
}

fn decide_action(tags_path: &Path, force: bool) -> Result<Action> {
    if !tags_path.exists() {
        return Ok(Action::FullRegen { backed_up: None });
    }

    let existing =
        fs::read_to_string(tags_path).with_context(|| format!("reading {}", tags_path.display()))?;

    let has_sentinels = existing.contains(GENERATED_BEGIN) && existing.contains(GENERATED_END);
    let has_custom = existing.contains("acTagSpecCustom");

    if !force && has_sentinels && has_custom {
        return Ok(Action::ReplaceInPlace { existing });
    }

    // Full regen — back up the old file so hand-edits are recoverable.
    let bak = tags_path.with_extension("ts.bak");
    fs::copy(tags_path, &bak)
        .with_context(|| format!("backing up {} → {}", tags_path.display(), bak.display()))?;
    Ok(Action::FullRegen { backed_up: Some(bak) })
}

/// Replace the sentinel-bracketed block in `existing` with `generated_block`.
/// Whitespace outside the sentinels is preserved byte-for-byte.
fn splice_generated_block(existing: &str, generated_block: &str) -> Result<String> {
    let begin_idx = existing
        .find(GENERATED_BEGIN)
        .ok_or_else(|| anyhow!("sentinel '{}' not found", GENERATED_BEGIN))?;
    let end_idx = existing
        .find(GENERATED_END)
        .ok_or_else(|| anyhow!("sentinel '{}' not found", GENERATED_END))?;
    if end_idx <= begin_idx {
        return Err(anyhow!(
            "sentinels are out of order — end appears before begin"
        ));
    }
    let after_end = end_idx + GENERATED_END.len();

    let mut out = String::with_capacity(existing.len() + generated_block.len());
    out.push_str(&existing[..begin_idx]);
    out.push_str(generated_block);
    out.push_str(&existing[after_end..]);
    Ok(out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_to_camel_basic() {
        assert_eq!(snake_to_camel("all_axes_homed"), "allAxesHomed");
        assert_eq!(snake_to_camel("req_start_auto"), "reqStartAuto");
        assert_eq!(snake_to_camel("x"), "x");
        assert_eq!(snake_to_camel(""), "");
        assert_eq!(snake_to_camel("already_camelcase_ish"), "alreadyCamelcaseIsh");
    }

    #[test]
    fn snake_to_camel_preserves_digits_and_leading_caps() {
        assert_eq!(snake_to_camel("axis_1_position"), "axis1Position");
        // First char is forced lowercase (vars shouldn't be SCREAMING_SNAKE
        // but if one is, we don't want to emit a type name).
        assert_eq!(snake_to_camel("Foo_bar"), "fooBar");
    }

    #[test]
    fn var_type_mapping() {
        assert_eq!(var_type_to_ts("bool"), Some("boolean"));
        assert_eq!(var_type_to_ts("f32"), Some("number"));
        assert_eq!(var_type_to_ts("i64"), Some("number"));
        assert_eq!(var_type_to_ts("string"), Some("string"));
        assert_eq!(var_type_to_ts("struct_something"), None);
    }

    #[test]
    fn collect_tags_filters_and_sorts() {
        let project = serde_json::json!({
            "variables": {
                "all_axes_homed": { "type": "bool", "ux": true },
                "hidden_internal": { "type": "u32" }, // ux missing → default false, excluded
                "z_explicit_false": { "type": "bool", "ux": false },
                "jog_distance_mm": { "type": "f64", "ux": true },
                "info_test_id": { "type": "string", "ux": true },
            }
        });
        let tags = collect_tags(&project);
        let names: Vec<_> = tags.iter().map(|t| t.tag_name.as_str()).collect();
        assert_eq!(names, vec!["allAxesHomed", "infoTestId", "jogDistanceMm"]);
        // Spot-check mapping of the first
        assert_eq!(tags[0].fqdn, "gm.all_axes_homed");
        assert_eq!(tags[0].value_type, "boolean");
        assert_eq!(tags[2].value_type, "number");
    }

    #[test]
    fn splice_replaces_only_generated_block() {
        let existing = "header\n\
            // autocore-codegen:generated-start\n\
            OLD CONTENT\n\
            // autocore-codegen:generated-end\n\
            custom kept\n\
            export const acTagSpecCustom = [];\n";
        let new_block = "// autocore-codegen:generated-start\n\
            NEW CONTENT\n\
            // autocore-codegen:generated-end";
        let spliced = splice_generated_block(existing, new_block).unwrap();
        assert!(spliced.contains("NEW CONTENT"));
        assert!(!spliced.contains("OLD CONTENT"));
        assert!(spliced.starts_with("header\n"));
        assert!(spliced.contains("custom kept"));
        assert!(spliced.contains("export const acTagSpecCustom = [];"));
    }

    #[test]
    fn splice_errors_on_missing_sentinel() {
        let existing = "no sentinels here";
        let new_block = "irrelevant";
        assert!(splice_generated_block(existing, new_block).is_err());
    }

    #[test]
    fn render_generated_block_emits_sorted_records() {
        let tags = vec![
            Tag { tag_name: "aFirst".into(), fqdn: "gm.a_first".into(), value_type: "boolean" },
            Tag { tag_name: "bSecond".into(), fqdn: "gm.b_second".into(), value_type: "number" },
        ];
        let block = render_generated_block(&tags);
        assert!(block.starts_with(GENERATED_BEGIN));
        assert!(block.trim_end().ends_with(GENERATED_END));
        assert!(block.contains("\"tagName\": \"aFirst\""));
        assert!(block.contains("\"fqdn\": \"gm.b_second\""));
        assert!(block.contains("as const satisfies readonly TagConfig[]"));
    }
}
