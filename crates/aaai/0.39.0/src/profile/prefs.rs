//! User preferences — persisted to `~/.aaai/prefs.yaml`.
//!
//! Currently stores the GUI theme selection.  Future preferences
//! (font size, language override, etc.) can be added here without
//! breaking the format because unknown YAML keys are ignored by serde.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// GUI colour theme.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// iced built-in light palette (default).
    #[default]
    Light,
    /// iced built-in dark palette.
    Dark,
    /// System preference (not yet implemented — falls back to Light).
    System,
    /// High-contrast light theme (snora-design preset, ≥8:1 status contrast).
    #[serde(rename = "high_contrast_light")]
    HighContrastLight,
    /// High-contrast dark theme (snora-design preset, ≥8:1 status contrast).
    #[serde(rename = "high_contrast_dark")]
    HighContrastDark,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Light  => write!(f, "Light"),
            Theme::Dark   => write!(f, "Dark"),
            Theme::System            => write!(f, "System"),
            Theme::HighContrastLight => write!(f, "High Contrast Light"),
            Theme::HighContrastDark  => write!(f, "High Contrast Dark"),
        }
    }
}

impl Theme {
    /// All user-selectable themes, in display order (RFC 093).
    ///
    /// `System` is excluded until OS dark-mode detection is available
    /// (RFC 093 §5.1 — hiding avoids a visibly broken picker option).
    /// RFC 094 appends `HighContrastLight` and `HighContrastDark` here.
    /// Returns true for the two high-contrast variants.
    pub fn is_high_contrast(self) -> bool {
        matches!(self, Theme::HighContrastLight | Theme::HighContrastDark)
    }

    pub fn choices() -> &'static [Theme] {
        &[Theme::Light, Theme::Dark, Theme::HighContrastLight, Theme::HighContrastDark]
    }
}

/// Persisted user preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPrefs {
    /// Selected GUI theme.
    #[serde(default)]
    pub theme: Theme,

    /// Locale code (e.g. "en", "ja"). Empty string = follow system / fallback.
    /// RFC 036 — previously tracked only in the GUI session; now persisted.
    #[serde(default)]
    pub language: String,

    /// Directory names silently excluded from every audit.
    /// Converted to `<name>/**` glob patterns and prepended to the
    /// `IgnoreRules` before any per-project `.aaaiignore` patterns.
    /// RFC 036 — configurable via the Settings dialog.
    #[serde(default = "default_ignored_dirs")]
    pub global_ignored_dirs: Vec<String>,
}

fn default_ignored_dirs() -> Vec<String> {
    vec![
        ".git".into(),
        "target".into(),
        "node_modules".into(),
        ".DS_Store".into(),
    ]
}

impl UserPrefs {
    fn path() -> anyhow::Result<PathBuf> {
        let base = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
            .join(".aaai");
        std::fs::create_dir_all(&base)?;
        Ok(base.join("prefs.yaml"))
    }

    /// Load from `~/.aaai/prefs.yaml`.  Returns defaults if the file is absent.
    pub fn load() -> Self {
        match Self::path().and_then(|p| {
            if !p.exists() { return Ok(Self::default()); }
            let text = std::fs::read_to_string(&p)?;
            serde_yaml::from_str(&text).map_err(|e| anyhow::anyhow!(e))
        }) {
            Ok(prefs) => prefs,
            Err(e) => {
                log::warn!("Could not load prefs: {e}");
                Self::default()
            }
        }
    }

    /// Save to `~/.aaai/prefs.yaml`.
    pub fn save(&self) {
        if let Err(e) = Self::path().and_then(|p| {
            let yaml = serde_yaml::to_string(self).map_err(|e| anyhow::anyhow!(e))?;
            std::fs::write(&p, yaml)?;
            Ok(())
        }) {
            log::warn!("Could not save prefs: {e}");
        }
    }
}

#[cfg(test)]
mod tests;
