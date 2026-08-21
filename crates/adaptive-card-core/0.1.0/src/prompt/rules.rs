//! Canonical rule text snippets injected into system prompts.

use crate::types::Host;
use std::fmt::Write as _;

pub const SCHEMA_RULES: &str = "\
CARD SCHEMA RULES:
- Root must have: type='AdaptiveCard', version='1.6', body=[elements]
- Max 15 top-level body elements, max 4 levels of nesting
- Max 6 actions per card (fewer on restrictive hosts)
- Every Input.* requires an 'id' field
- Every Image requires 'url' and 'altText'
";

pub const A11Y_RULES: &str = "\
ACCESSIBILITY RULES:
- Add root 'speak' property summarizing the card (for screen readers)
- Every Image must have 'altText' describing what it shows
- TextBlock with text > 60 chars must set 'wrap: true'
- Every Input.* must have a 'label' describing its purpose
- Use 'heading' style for section titles so screen readers announce hierarchy
";

/// Complete element reference covering every Adaptive Card v1.6 element,
/// input, chart, action, design pattern, and templating guide. Loaded from
/// the markdown file at build time via `include_str!`.
///
/// This replaces the shorter `ELEMENT_REFERENCE` + `DESIGN_PATTERNS` constants
/// with the full 1462-line reference that includes copy-ready JSON examples
/// for every element type, property tables, and composition patterns.
pub const ELEMENT_REFERENCE: &str = include_str!("../../data/adaptive-cards-element-reference.md");

/// 23 annotated design patterns with `design_notes` explaining WHY each
/// pattern works, `visual_patterns_used` for composition, and anti-patterns.
/// This teaches the LLM creative composition techniques (timeline tricks,
/// carousel patterns, split panels, view-swap toggles, etc.) rather than
/// just element syntax.
pub const DESIGN_PATTERNS: &str = include_str!("../../data/adaptive-cards-design-patterns.md");

#[must_use]
pub fn host_rules(host: Host) -> String {
    let mut s = format!("HOST TARGET: {host:?}\n");
    let _ = writeln!(s, "- Max card version: {}", host.max_version().as_str());
    if let Some(max) = host.max_actions() {
        let _ = writeln!(s, "- Max actions per card: {max}");
    }
    match host {
        Host::Outlook => {
            s.push_str("- No Media, CodeBlock, Action.Execute\n");
            s.push_str("- Prefer Action.Submit over Action.Execute\n");
        }
        Host::Webex => {
            s.push_str("- No Table, CodeBlock, Action.Execute\n");
            s.push_str("- Version capped at 1.3 — avoid v1.4+ features\n");
        }
        Host::VivaConnections => {
            s.push_str("- No CodeBlock, limited Table support\n");
        }
        Host::Teams => {
            s.push_str("- Prefer Action.Execute with 'verb' for modern bots\n");
        }
        Host::WebChat | Host::Windows | Host::Generic => {
            s.push_str("- Full v1.6 support\n");
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outlook_rules_mention_no_media() {
        let text = host_rules(Host::Outlook);
        assert!(text.contains("Outlook"));
        assert!(text.contains("No Media"));
    }

    #[test]
    fn webex_rules_mention_v1_3_cap() {
        let text = host_rules(Host::Webex);
        assert!(text.contains("1.3"));
    }

    #[test]
    fn element_reference_covers_key_elements() {
        assert!(ELEMENT_REFERENCE.contains("TextBlock"));
        assert!(ELEMENT_REFERENCE.contains("ColumnSet"));
        assert!(ELEMENT_REFERENCE.contains("Container"));
        assert!(ELEMENT_REFERENCE.contains("FactSet"));
        assert!(ELEMENT_REFERENCE.contains("Input.Text"));
        assert!(ELEMENT_REFERENCE.contains("Input.ChoiceSet"));
        assert!(ELEMENT_REFERENCE.contains("Action.Execute"));
        assert!(ELEMENT_REFERENCE.contains("Action.ToggleVisibility"));
    }

    #[test]
    fn element_reference_covers_charts() {
        assert!(ELEMENT_REFERENCE.contains("Chart.Donut"));
        assert!(ELEMENT_REFERENCE.contains("Chart.Line"));
        assert!(ELEMENT_REFERENCE.contains("Chart.Gauge"));
    }

    #[test]
    fn element_reference_covers_design_patterns() {
        assert!(ELEMENT_REFERENCE.contains("Multi-Step Wizard"));
        assert!(ELEMENT_REFERENCE.contains("KPI Metric"));
        assert!(ELEMENT_REFERENCE.contains("User Profile"));
    }
}
