//! Host capability matrix.
//!
//! Values derived from official Microsoft Adaptive Cards documentation:
//! - <https://learn.microsoft.com/en-us/adaptive-cards/resources/partners>
//! - <https://adaptivecards.microsoft.com/?topic=hostConfig>
//!
//! Update this table when Microsoft publishes new host capability info.

use crate::types::{CardVersion, Host};

/// Element types supported on every host (core set).
pub const UNIVERSAL_ELEMENTS: &[&str] = &[
    "TextBlock",
    "Image",
    "Container",
    "ColumnSet",
    "Column",
    "FactSet",
    "ImageSet",
    "ActionSet",
    "Input.Text",
    "Input.Number",
    "Input.Date",
    "Input.Time",
    "Input.Toggle",
    "Input.ChoiceSet",
];

/// Element types newer than v1.3 — not supported on hosts capped at v1.3.
pub const ELEMENTS_V1_5_PLUS: &[&str] = &["Table", "TableCell", "TableRow"];
pub const ELEMENTS_V1_2_PLUS: &[&str] = &["RichTextBlock"];
pub const ELEMENTS_V1_6_PLUS: &[&str] = &["CodeBlock"];
pub const ELEMENTS_V1_1_PLUS: &[&str] = &["Media"];

/// Action types that need Action.Execute support (v1.4+ for most hosts).
pub const ACTIONS_V1_4_PLUS: &[&str] = &["Action.Execute"];

/// Return all elements a host supports.
#[must_use]
pub fn supported_elements(host: Host) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = UNIVERSAL_ELEMENTS.to_vec();

    let max = host.max_version();

    if max >= CardVersion::V1_1 && host != Host::Outlook {
        out.extend(ELEMENTS_V1_1_PLUS);
    }
    if max >= CardVersion::V1_2 {
        out.extend(ELEMENTS_V1_2_PLUS);
    }
    if max >= CardVersion::V1_5 {
        out.extend(ELEMENTS_V1_5_PLUS);
    }
    if max >= CardVersion::V1_6 {
        out.extend(ELEMENTS_V1_6_PLUS);
    }
    out
}

/// Return all actions a host supports.
#[must_use]
pub fn supported_actions(host: Host) -> Vec<&'static str> {
    let mut out = vec![
        "Action.OpenUrl",
        "Action.Submit",
        "Action.ShowCard",
        "Action.ToggleVisibility",
    ];
    if host.supports_action("Action.Execute") {
        out.push("Action.Execute");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_supports_everything_up_to_v1_6() {
        let els = supported_elements(Host::Generic);
        assert!(els.contains(&"Table"));
        assert!(els.contains(&"CodeBlock"));
        assert!(els.contains(&"TextBlock"));
    }

    #[test]
    fn webex_excludes_table_and_codeblock() {
        let els = supported_elements(Host::Webex);
        assert!(!els.contains(&"Table"));
        assert!(!els.contains(&"CodeBlock"));
    }

    #[test]
    fn outlook_excludes_media_and_codeblock() {
        let els = supported_elements(Host::Outlook);
        assert!(!els.contains(&"Media"));
        assert!(!els.contains(&"CodeBlock"));
    }

    #[test]
    fn webex_supported_actions_exclude_execute() {
        let acts = supported_actions(Host::Webex);
        assert!(!acts.contains(&"Action.Execute"));
        assert!(acts.contains(&"Action.Submit"));
    }
}
