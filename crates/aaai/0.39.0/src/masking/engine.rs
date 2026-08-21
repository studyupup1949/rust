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
mod tests;
