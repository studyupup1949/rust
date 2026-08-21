//! Built-in secret detection patterns.
//!
//! These patterns are intentionally conservative (few false positives) rather
//! than exhaustive.  Users can add custom patterns via the project config.

/// A named secret pattern.
pub struct SecretPattern {
    pub name: &'static str,
    /// The regex pattern string.  The matching portion is replaced with the mask.
    pub pattern: &'static str,
    /// Optional capture group index that contains the secret value.
    /// If None, the entire match is masked.
    pub value_group: Option<usize>,
}

/// Built-in patterns for common secret formats.
pub static BUILTIN_PATTERNS: &[SecretPattern] = &[
    SecretPattern {
        name: "Generic API key assignment",
        pattern: r#"(?i)(api[_\-]?key|api[_\-]?secret|auth[_\-]?token)\s*[:=]\s*["']?([A-Za-z0-9\-_/+]{16,})"#,
        value_group: Some(2),
    },
    SecretPattern {
        name: "Password assignment",
        pattern: r#"(?i)(password|passwd|pwd|secret)\s*[:=]\s*["']?([^\s"']{8,})"#,
        value_group: Some(2),
    },
    SecretPattern {
        name: "AWS access key",
        pattern: r"(AKIA[0-9A-Z]{16})",
        value_group: Some(1),
    },
    SecretPattern {
        name: "AWS secret key assignment",
        pattern: r#"(?i)aws[_\-]?secret[_\-]?access[_\-]?key\s*[:=]\s*["']?([A-Za-z0-9/+=]{40})"#,
        value_group: Some(1),
    },
    SecretPattern {
        name: "Generic Bearer token",
        pattern: r"(?i)Bearer\s+([A-Za-z0-9\-._~+/]+=*)",
        value_group: Some(1),
    },
    SecretPattern {
        name: "Private key header",
        pattern: r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----",
        value_group: None,
    },
    SecretPattern {
        name: "Connection string with password",
        pattern: r"://([^:@\s]+):([^@\s]+)@",
        value_group: Some(2),
    },
    SecretPattern {
        name: "GitHub token",
        pattern: r"(gh[pousr]_[A-Za-z0-9]{36,})",
        value_group: Some(1),
    },
    SecretPattern {
        name: "Slack token",
        pattern: r"(xox[baprs]-[0-9A-Za-z\-]{10,})",
        value_group: Some(1),
    },
];
