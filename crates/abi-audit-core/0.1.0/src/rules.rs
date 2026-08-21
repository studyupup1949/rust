use crate::model::Severity;

#[derive(Debug, Clone, Copy)]
pub struct RuleMetadata {
    pub code: &'static str,
    pub default_severity: Severity,
    pub title: &'static str,
    pub summary: &'static str,
}

const RULES: &[RuleMetadata] = &[
    RuleMetadata {
        code: "non-ffi-safe-signature",
        default_severity: Severity::Error,
        title: "Non-FFI-safe signature",
        summary: "An exported C ABI function uses a Rust-specific or unsupported signature shape.",
    },
    RuleMetadata {
        code: "missing-repr",
        default_severity: Severity::Error,
        title: "Missing stable repr",
        summary: "A local type crosses the C ABI boundary by value without a stable representation the tool can model.",
    },
    RuleMetadata {
        code: "missing-export-attr",
        default_severity: Severity::Warning,
        title: "Missing explicit export attribute",
        summary: "An exported C ABI function does not declare `no_mangle` or `export_name`.",
    },
    RuleMetadata {
        code: "missing-artifact-target",
        default_severity: Severity::Warning,
        title: "Missing native artifact target",
        summary: "A package exposes ABI exports but is not configured as a `cdylib` or `staticlib`.",
    },
    RuleMetadata {
        code: "artifact-build-missing",
        default_severity: Severity::Warning,
        title: "Artifact build missing",
        summary: "A package declares native library crate types but the audit did not capture matching build artifacts.",
    },
    RuleMetadata {
        code: "artifact-inspection-unavailable",
        default_severity: Severity::Warning,
        title: "Artifact inspection unavailable",
        summary: "A dynamic library was built, but host-native symbol inspection was unavailable.",
    },
    RuleMetadata {
        code: "artifact-missing-export",
        default_severity: Severity::Warning,
        title: "Artifact missing expected export",
        summary: "A compiled `cdylib` does not expose an explicitly named source export.",
    },
    RuleMetadata {
        code: "header-missing-export",
        default_severity: Severity::Warning,
        title: "Header missing Rust export",
        summary: "A configured public header declares a symbol that was not found in the Rust export set.",
    },
    RuleMetadata {
        code: "export-missing-header",
        default_severity: Severity::Warning,
        title: "Export missing public header",
        summary: "A Rust export is not declared in any configured public header.",
    },
    RuleMetadata {
        code: "header-signature-mismatch",
        default_severity: Severity::Warning,
        title: "Header signature mismatch",
        summary: "Rust and header declarations agree on a symbol name but not on the normalized C ABI signature shape.",
    },
    RuleMetadata {
        code: "header-sync-missing-output",
        default_severity: Severity::Warning,
        title: "Header sync output missing",
        summary: "A configured generated header output path does not exist.",
    },
    RuleMetadata {
        code: "header-sync-missing-config",
        default_severity: Severity::Warning,
        title: "Header sync config missing",
        summary: "A configured header generation workflow references a missing config file.",
    },
    RuleMetadata {
        code: "header-sync-untracked-header",
        default_severity: Severity::Warning,
        title: "Header sync output not audited",
        summary: "A configured generated header is not part of the audited public header set.",
    },
    RuleMetadata {
        code: "header-sync-stale",
        default_severity: Severity::Warning,
        title: "Header sync appears stale",
        summary: "A configured generated header appears older than its Rust or generator inputs.",
    },
    RuleMetadata {
        code: "baseline-drift",
        default_severity: Severity::Warning,
        title: "Baseline drift detected",
        summary: "The current audited ABI surface differs from the selected baseline snapshot.",
    },
];

pub fn all_rule_metadata() -> &'static [RuleMetadata] {
    RULES
}

pub fn rule_metadata(code: &str) -> Option<&'static RuleMetadata> {
    RULES.iter().find(|rule| rule.code == code)
}
