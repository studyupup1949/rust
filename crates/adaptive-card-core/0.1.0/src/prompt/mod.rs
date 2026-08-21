//! Build LLM system prompts for Adaptive Card generation.

pub mod examples;
pub mod rules;

use crate::knowledge::KnowledgeBase;
use crate::types::Host;
use std::fmt::Write as _;

/// Options for [`build_system_prompt`].
#[derive(Debug, Clone, Default)]
pub struct PromptOpts<'a> {
    /// Entry IDs to include verbatim as examples.
    pub include_examples: Vec<String>,
    /// Target host for host-specific rules.
    pub target_host: Option<Host>,
    /// Tools the LLM is allowed to call (by name).
    pub available_tools: Vec<&'static str>,
    /// Knowledge base handle for looking up example entries.
    pub knowledge_base: Option<&'a KnowledgeBase>,
    /// Optional user query — when provided, the builder auto-selects up to
    /// 3 matching KB entries and injects them as few-shot examples. This
    /// gives the LLM concrete reference cards to emulate.
    pub user_query: Option<String>,
}

/// Build a system prompt guiding an LLM to generate valid Adaptive Cards.
#[must_use]
pub fn build_system_prompt(opts: &PromptOpts) -> String {
    let mut out = String::from(
        "You are an expert Adaptive Cards v1.6 designer. Your job: generate valid AdaptiveCard JSON for the user's request.\n\n",
    );

    // Core rules (always)
    out.push_str(rules::SCHEMA_RULES);
    out.push('\n');
    out.push_str(rules::A11Y_RULES);
    out.push('\n');

    // Complete element reference (elements, inputs, charts, actions, patterns, templating)
    out.push_str(rules::ELEMENT_REFERENCE);
    out.push('\n');

    // NOTE: DESIGN_PATTERNS (23 annotated entries, ~25KB) is NOT injected into
    // every prompt to avoid attention dilution. It is available via the
    // `suggest_layout` tool — the LLM can browse patterns on demand.
    // The element reference already contains enough composition guidance.

    // Host-specific rules (when a target host is specified)
    if let Some(host) = opts.target_host {
        out.push_str(&rules::host_rules(host));
        out.push('\n');
    }

    // Available tools
    if !opts.available_tools.is_empty() {
        out.push_str("\nAVAILABLE TOOLS:\n");
        for tool in &opts.available_tools {
            let _ = writeln!(out, "  - {tool}");
        }
        out.push_str("\nWORKFLOW: Use tools to browse examples, validate output, and refine. Always validate before returning. Never output invalid JSON.\n");
    }

    // Explicitly requested examples (by ID)
    if let (Some(kb), false) = (opts.knowledge_base, opts.include_examples.is_empty()) {
        out.push_str("\nINLINED EXAMPLES:\n");
        for id in &opts.include_examples {
            if let Some(entry) = kb.by_id(id) {
                let _ = write!(
                    out,
                    "\n## {}\n{}\n```json\n{}\n```\n",
                    entry.title,
                    entry.description,
                    truncate_json(&entry.card, 3000),
                );
            }
        }
    }

    // Auto-selected few-shot examples (based on user query).
    // We fetch up to 5 matches so the LLM can combine patterns from
    // multiple reference cards instead of copying any single one.
    if let (Some(kb), Some(query)) = (opts.knowledge_base, &opts.user_query)
        && !query.is_empty()
    {
        let suggestions = kb.suggest(query, 5);
        if !suggestions.is_empty() {
            out.push_str("\nREFERENCE EXAMPLES (combine patterns from multiple cards — do NOT copy any single one verbatim):\n");
            for (i, s) in suggestions.iter().enumerate() {
                if let Some(entry) = kb.by_id(&s.entry_id) {
                    let _ = write!(
                        out,
                        "\n### Example {} — {} ({})\n{}\n```json\n{}\n```\n",
                        i + 1,
                        entry.title,
                        entry.complexity_str(),
                        entry.description,
                        truncate_json(&entry.card, 3000),
                    );
                }
            }
            out.push_str(
                "\nCOMBINE STRATEGY: Treat these as a palette, not templates. Identify the \
best pattern from each and combine them into something NEW:\n\
- Header: pick the cleanest icon+title ColumnSet layout\n\
- Sections: pick the most effective Container styles (emphasis/good/attention)\n\
- Forms: pick the best input grouping and labeling\n\
- Lists: pick the most readable ColumnSet / FactSet patterns\n\
- Actions: pick the most intuitive button placement and style\n\
Your output should feel ORIGINAL — inspired by these examples but not derivative.\n",
            );
        }
    }

    out
}

/// Serialize a JSON value to a pretty string and truncate to `max_chars`.
fn truncate_json(value: &serde_json::Value, max_chars: usize) -> String {
    let full = serde_json::to_string_pretty(value).unwrap_or_default();
    if full.len() <= max_chars {
        full
    } else {
        let mut s = full[..max_chars].to_string();
        s.push_str("\n... (truncated)");
        s
    }
}

/// Build an example showcase snippet (delegates to `examples::showcase`).
#[must_use]
pub fn build_example_showcase(kb: &KnowledgeBase, query: &str, limit: usize) -> String {
    examples::showcase(kb, query, limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_schema_rules_always() {
        let opts = PromptOpts::default();
        let prompt = build_system_prompt(&opts);
        assert!(prompt.contains("CARD SCHEMA RULES"));
        assert!(prompt.contains("ACCESSIBILITY RULES"));
    }

    #[test]
    fn includes_element_reference() {
        let opts = PromptOpts::default();
        let prompt = build_system_prompt(&opts);
        // Full element reference loaded from markdown
        assert!(prompt.contains("TextBlock"));
        assert!(prompt.contains("ColumnSet"));
        assert!(prompt.contains("FactSet"));
        assert!(prompt.contains("Action.Execute"));
        assert!(prompt.contains("Input.ChoiceSet"));
        // Design patterns section in the reference
        assert!(prompt.contains("Multi-Step Wizard"));
        assert!(prompt.contains("KPI Metric"));
    }

    #[test]
    fn includes_host_rules_when_set() {
        let opts = PromptOpts {
            target_host: Some(Host::Outlook),
            ..Default::default()
        };
        let prompt = build_system_prompt(&opts);
        assert!(prompt.contains("Outlook"));
    }

    #[test]
    fn lists_tools_when_provided() {
        let opts = PromptOpts {
            available_tools: vec!["validate_card", "optimize_card"],
            ..Default::default()
        };
        let prompt = build_system_prompt(&opts);
        assert!(prompt.contains("validate_card"));
        assert!(prompt.contains("optimize_card"));
        assert!(prompt.contains("WORKFLOW"));
    }
}
