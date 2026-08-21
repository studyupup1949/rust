//! Secret masking engine.
//!
//! Applies regex-based patterns to strings and replaces matched secrets with
//! `***MASKED***`.  Used by the report generator and CLI output when
//! `--mask-secrets` is active.

use regex::Regex;

use super::patterns::{BUILTIN_PATTERNS, SecretPattern};

const MASK: &str = "***MASKED***";

/// A compiled set of masking rules.
pub struct MaskingEngine {
    rules: Vec<(Regex, Option<usize>)>,
}

impl MaskingEngine {
    /// Build an engine from the built-in patterns only.
    pub fn builtin() -> Self {
        Self::from_patterns(BUILTIN_PATTERNS, &[])
    }

    /// Build an engine from built-in patterns plus custom regex strings.
    pub fn with_custom(custom: &[String]) -> Self {
        Self::from_patterns(BUILTIN_PATTERNS, custom)
    }

    fn from_patterns(builtin: &[SecretPattern], custom: &[String]) -> Self {
        let mut rules = Vec::new();
        for sp in builtin {
            match Regex::new(sp.pattern) {
                Ok(re) => rules.push((re, sp.value_group)),
                Err(e) => log::warn!("Built-in mask pattern {:?} failed to compile: {e}", sp.name),
            }
        }
        for pat in custom {
            match Regex::new(pat) {
                Ok(re) => rules.push((re, None)),
                Err(e) => log::warn!("Custom mask pattern {:?} failed to compile: {e}", pat),
            }
        }
        Self { rules }
    }

    /// Apply all masking rules to `text`, returning the masked version.
    pub fn mask(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (re, group) in &self.rules {
            result = mask_with_regex(&result, re, *group);
        }
        result
    }

    /// Mask only if secrets are present; return `None` if no change.
    pub fn mask_if_needed(&self, text: &str) -> Option<String> {
        let masked = self.mask(text);
        if masked == text { None } else { Some(masked) }
    }
}

fn mask_with_regex(text: &str, re: &Regex, group: Option<usize>) -> String {
    match group {
        None => re.replace_all(text, MASK).to_string(),
        Some(g) => {
            let mut result = text.to_string();
            // Process matches in reverse order so offsets remain valid.
            let captures: Vec<_> = re.captures_iter(text).collect();
            for cap in captures.iter().rev() {
                if let Some(m) = cap.get(g) {
                    result.replace_range(m.start()..m.end(), MASK);
                }
            }
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> MaskingEngine { MaskingEngine::builtin() }

    #[test]
    fn masks_api_key_assignment() {
        let text = r#"api_key = "sk-abcdefghijklmnop1234567890""#;
        let masked = engine().mask(text);
        assert!(masked.contains(MASK), "expected mask in: {masked}");
        assert!(!masked.contains("sk-abcdefghijklmnop"), "key should be masked");
    }

    #[test]
    fn masks_aws_access_key() {
        let text = "AKIAIOSFODNN7EXAMPLE";
        let masked = engine().mask(text);
        assert!(masked.contains(MASK));
    }

    #[test]
    fn safe_text_unchanged() {
        let text = "port = 8080";
        let masked = engine().mask(text);
        assert_eq!(masked, text);
    }

    #[test]
    fn mask_if_needed_returns_none_for_safe_text() {
        assert!(engine().mask_if_needed("ordinary text").is_none());
    }

    #[test]
    fn custom_pattern_applied() {
        let engine = MaskingEngine::with_custom(&["SECRET_VALUE".to_string()]);
        let masked = engine.mask("here is SECRET_VALUE exposed");
        assert!(masked.contains(MASK));
    }

    #[test]
    fn private_key_header_masked() {
        let text = "-----BEGIN RSA PRIVATE KEY-----\nABC123\n-----END RSA PRIVATE KEY-----";
        let masked = engine().mask(text);
        assert!(masked.contains(MASK));
    }

    #[test]
    fn github_token_masked() {
        let text = "token = ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef123456";
        let masked = engine().mask(text);
        assert!(masked.contains(MASK));
    }
}
// (tests already included in the module above)
