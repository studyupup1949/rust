//! Shared types used across the core library.

use serde::{Deserialize, Serialize};

/// Target host for Adaptive Card rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Host {
    Generic,
    Teams,
    Outlook,
    WebChat,
    Windows,
    VivaConnections,
    Webex,
}

impl Host {
    /// Parse a host name from a lowercase string (accepts common aliases).
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "generic" | "" => Some(Self::Generic),
            "teams" | "microsoft_teams" => Some(Self::Teams),
            "outlook" => Some(Self::Outlook),
            "webchat" | "web_chat" => Some(Self::WebChat),
            "windows" => Some(Self::Windows),
            "viva" | "viva_connections" => Some(Self::VivaConnections),
            "webex" => Some(Self::Webex),
            _ => None,
        }
    }

    /// Maximum Adaptive Card version this host supports.
    #[must_use]
    pub fn max_version(&self) -> CardVersion {
        match self {
            Self::Outlook | Self::VivaConnections => CardVersion::V1_4,
            Self::Webex => CardVersion::V1_3,
            _ => CardVersion::V1_6,
        }
    }

    /// Maximum number of actions this host renders. `None` means no hard limit.
    #[must_use]
    pub fn max_actions(&self) -> Option<usize> {
        match self {
            Self::Teams => Some(6),
            Self::Outlook | Self::VivaConnections => Some(4),
            Self::Webex => Some(5),
            _ => None,
        }
    }

    /// Whether this host supports the given element type.
    /// Element type names match the `type` field values in Adaptive Card JSON.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn supports_element(&self, type_name: &str) -> bool {
        // Common elements supported everywhere
        const UNIVERSAL: &[&str] = &[
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
        if UNIVERSAL.contains(&type_name) {
            return true;
        }
        match (self, type_name) {
            // Table and RichTextBlock need v1.5+
            (Self::Webex | Self::Outlook | Self::VivaConnections, "Table" | "RichTextBlock") => {
                false
            }
            // Media needs v1.1+ but Outlook doesn't render it
            (Self::Outlook, "Media") => false,
            // CodeBlock is v1.6+
            (Self::Outlook | Self::VivaConnections | Self::Webex, "CodeBlock") => false,
            _ => true,
        }
    }

    /// Whether this host supports the given action type.
    #[must_use]
    pub fn supports_action(&self, type_name: &str) -> bool {
        match (self, type_name) {
            (Self::Webex | Self::Outlook, "Action.Execute") => false,
            _ => matches!(
                type_name,
                "Action.OpenUrl"
                    | "Action.Submit"
                    | "Action.Execute"
                    | "Action.ShowCard"
                    | "Action.ToggleVisibility"
            ),
        }
    }

    /// Return all known hosts in declaration order.
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::Generic,
            Self::Teams,
            Self::Outlook,
            Self::WebChat,
            Self::Windows,
            Self::VivaConnections,
            Self::Webex,
        ]
    }
}

/// Adaptive Card schema version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CardVersion {
    V1_0,
    V1_1,
    V1_2,
    V1_3,
    V1_4,
    V1_5,
    V1_6,
}

impl CardVersion {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1_0 => "1.0",
            Self::V1_1 => "1.1",
            Self::V1_2 => "1.2",
            Self::V1_3 => "1.3",
            Self::V1_4 => "1.4",
            Self::V1_5 => "1.5",
            Self::V1_6 => "1.6",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "1.0" => Some(Self::V1_0),
            "1.1" => Some(Self::V1_1),
            "1.2" => Some(Self::V1_2),
            "1.3" => Some(Self::V1_3),
            "1.4" => Some(Self::V1_4),
            "1.5" => Some(Self::V1_5),
            "1.6" => Some(Self::V1_6),
            _ => None,
        }
    }
}

use serde_json::Value;

/// Combined validation report (schema + a11y + host compat).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub schema_errors: Vec<SchemaError>,
    pub accessibility: AccessibilityReport,
    pub host_compat: Option<HostCompatReport>,
    pub card_version: Option<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaError {
    /// JSON Pointer path to the error location, e.g. `/body/0/items/2`.
    pub path: String,
    pub message: String,
    /// JSON Schema keyword that failed: `required`, `type`, `enum`, ...
    pub keyword: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityReport {
    /// Score 0-100.
    pub score: u8,
    pub issues: Vec<A11yIssue>,
    pub passes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yIssue {
    pub severity: A11ySeverity,
    pub rule: String,
    pub path: String,
    pub message: String,
    pub fix_hint: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum A11ySeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostCompatReport {
    pub host: Host,
    pub compatible: bool,
    pub version_ok: bool,
    pub unsupported_elements: Vec<String>,
    pub unsupported_actions: Vec<String>,
    /// `(actual_count, max_allowed)` when the card exceeds host action limit.
    pub too_many_actions: Option<(usize, usize)>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardAnalysis {
    pub element_count: usize,
    pub action_count: usize,
    pub nesting_depth: usize,
    pub unique_element_types: Vec<String>,
    pub duplicate_ids: Vec<String>,
    pub total_text_length: usize,
    pub has_images: bool,
    pub has_inputs: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptimizeOpts {
    #[serde(default)]
    pub accessibility: bool,
    #[serde(default)]
    pub performance: bool,
    #[serde(default)]
    pub modernize: bool,
    #[serde(default)]
    pub target_host: Option<Host>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformTarget {
    pub version: Option<CardVersion>,
    pub host: Option<Host>,
    #[serde(default)]
    pub strict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformReport {
    pub card: Value,
    pub removed: Vec<String>,
    pub downgraded: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateResult {
    pub template: Value,
    pub sample_data: Value,
    pub bindings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataToCardOpts {
    pub title: Option<String>,
    pub presentation: Option<Presentation>,
    pub host: Host,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Presentation {
    Table,
    FactSet,
    List,
    Chart,
    Auto,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_from_str_roundtrip() {
        for host in Host::all() {
            let s = serde_json::to_value(host).unwrap();
            let back: Host = serde_json::from_value(s).unwrap();
            assert_eq!(*host, back);
        }
    }

    #[test]
    fn host_from_str_aliases() {
        assert_eq!(Host::from_str("Teams"), Some(Host::Teams));
        assert_eq!(Host::from_str("microsoft_teams"), Some(Host::Teams));
        assert_eq!(Host::from_str("web_chat"), Some(Host::WebChat));
        assert_eq!(Host::from_str("viva"), Some(Host::VivaConnections));
        assert_eq!(Host::from_str(""), Some(Host::Generic));
        assert_eq!(Host::from_str("unknown"), None);
    }

    #[test]
    fn card_version_parse_roundtrip() {
        for v in &[
            CardVersion::V1_0,
            CardVersion::V1_1,
            CardVersion::V1_2,
            CardVersion::V1_3,
            CardVersion::V1_4,
            CardVersion::V1_5,
            CardVersion::V1_6,
        ] {
            assert_eq!(CardVersion::parse(v.as_str()), Some(*v));
        }
    }

    #[test]
    fn card_version_ordering() {
        assert!(CardVersion::V1_4 < CardVersion::V1_6);
        assert!(CardVersion::V1_6 > CardVersion::V1_3);
    }

    #[test]
    fn host_max_version_matrix() {
        assert_eq!(Host::Generic.max_version(), CardVersion::V1_6);
        assert_eq!(Host::Teams.max_version(), CardVersion::V1_6);
        assert_eq!(Host::WebChat.max_version(), CardVersion::V1_6);
        assert_eq!(Host::Windows.max_version(), CardVersion::V1_6);
        assert_eq!(Host::Outlook.max_version(), CardVersion::V1_4);
        assert_eq!(Host::VivaConnections.max_version(), CardVersion::V1_4);
        assert_eq!(Host::Webex.max_version(), CardVersion::V1_3);
    }

    #[test]
    fn host_max_actions() {
        assert_eq!(Host::Teams.max_actions(), Some(6));
        assert_eq!(Host::Outlook.max_actions(), Some(4));
        assert_eq!(Host::Webex.max_actions(), Some(5));
        assert_eq!(Host::Generic.max_actions(), None);
    }

    #[test]
    fn webex_rejects_table_and_execute() {
        assert!(!Host::Webex.supports_element("Table"));
        assert!(!Host::Webex.supports_action("Action.Execute"));
        assert!(Host::Webex.supports_element("TextBlock"));
    }

    #[test]
    fn outlook_rejects_execute_and_media() {
        assert!(!Host::Outlook.supports_action("Action.Execute"));
        assert!(!Host::Outlook.supports_element("Media"));
        assert!(Host::Outlook.supports_element("TextBlock"));
    }

    #[test]
    fn validation_report_serde_roundtrip() {
        let report = ValidationReport {
            valid: true,
            schema_errors: vec![],
            accessibility: AccessibilityReport {
                score: 100,
                issues: vec![],
                passes: vec!["has-speak".to_string()],
            },
            host_compat: None,
            card_version: Some("1.6".to_string()),
            suggestions: vec![],
        };
        let json = serde_json::to_value(&report).unwrap();
        let back: ValidationReport = serde_json::from_value(json).unwrap();
        assert_eq!(back.valid, report.valid);
        assert_eq!(back.accessibility.score, 100);
    }

    #[test]
    fn optimize_opts_default_all_false() {
        let opts = OptimizeOpts::default();
        assert!(!opts.accessibility);
        assert!(!opts.performance);
        assert!(!opts.modernize);
        assert!(opts.target_host.is_none());
    }
}
