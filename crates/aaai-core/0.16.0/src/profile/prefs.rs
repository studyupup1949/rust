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
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Light  => write!(f, "Light"),
            Theme::Dark   => write!(f, "Dark"),
            Theme::System => write!(f, "System"),
        }
    }
}

/// Persisted user preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPrefs {
    /// Selected GUI theme.
    #[serde(default)]
    pub theme: Theme,
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
mod tests {
    use super::*;

    #[test]
    fn round_trip_yaml() {
        let prefs = UserPrefs { theme: Theme::Dark };
        let yaml = serde_yaml::to_string(&prefs).unwrap();
        let restored: UserPrefs = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(restored.theme, Theme::Dark);
    }

    #[test]
    fn default_is_light() {
        assert_eq!(UserPrefs::default().theme, Theme::Light);
    }

    #[test]
    fn display_names() {
        assert_eq!(Theme::Light.to_string(), "Light");
        assert_eq!(Theme::Dark.to_string(), "Dark");
    }
}
