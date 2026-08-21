//! `.aaaiignore` — gitignore-style path exclusion for the diff engine.
//!
//! Rules are plain glob patterns, one per line.  Lines beginning with `#`
//! are comments.  Blank lines are ignored.  A leading `!` negates a rule.
//!
//! # Example `.aaaiignore`
//!
//! ```text
//! # Generated files
//! target/**
//! *.lock
//! .DS_Store
//!
//! # Never ignore these
//! !Cargo.lock
//! ```

use std::path::Path;

/// A compiled set of ignore rules loaded from an `.aaaiignore` file.
#[derive(Debug, Clone, Default)]
pub struct IgnoreRules {
    /// Each entry is `(negated, compiled_pattern)`.
    rules: Vec<(bool, glob::Pattern)>,
}

impl IgnoreRules {
    /// Load rules from the given file path.
    /// Returns an empty ruleset when the file doesn't exist.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", path.display()))?;
        Self::from_str(&text)
    }

    /// Parse rules from a string (one pattern per line).
    pub fn from_str(text: &str) -> anyhow::Result<Self> {
        let mut rules = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (negated, pattern_str) = if let Some(rest) = line.strip_prefix('!') {
                (true, rest.trim())
            } else {
                (false, line)
            };
            match glob::Pattern::new(pattern_str) {
                Ok(pat) => rules.push((negated, pat)),
                Err(e) => {
                    log::warn!(".aaaiignore: invalid pattern {:?} — {e}", pattern_str);
                }
            }
        }
        Ok(Self { rules })
    }

    /// Return `true` when `path` should be excluded from the diff.
    ///
    /// Rules are evaluated in order; the last matching rule wins.
    /// A negation rule (`!pattern`) un-ignores a previously ignored path.
    pub fn is_ignored(&self, path: &str) -> bool {
        let mut ignored = false;
        for (negated, pat) in &self.rules {
            if pat.matches(path) {
                ignored = !negated;
            }
        }
        ignored
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;
