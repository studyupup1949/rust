//! Audit profiles — named before/after/definition combos saved to
//! `~/.aaai/profiles.yaml`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// One saved audit project configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditProfile {
    /// Display name for this profile.
    pub name: String,
    pub before: String,
    pub after: String,
    pub definition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_file: Option<String>,
}

/// Root document for `~/.aaai/profiles.yaml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileStore {
    #[serde(default)]
    pub profiles: Vec<AuditProfile>,
}

impl ProfileStore {
    /// Load from `~/.aaai/profiles.yaml`, returning empty store if absent.
    pub fn load() -> anyhow::Result<Self> {
        let path = profile_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)?;
        Ok(serde_yaml::from_str(&text)?)
    }

    /// Save to `~/.aaai/profiles.yaml`.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = profile_path()?;
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(&path, yaml)?;
        Ok(())
    }

    pub fn add(&mut self, profile: AuditProfile) {
        // Replace existing profile with the same name.
        if let Some(pos) = self.profiles.iter().position(|p| p.name == profile.name) {
            self.profiles[pos] = profile;
        } else {
            self.profiles.push(profile);
        }
    }

    pub fn remove(&mut self, name: &str) {
        self.profiles.retain(|p| p.name != name);
    }
}

fn profile_path() -> anyhow::Result<PathBuf> {
    let base = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
        .join(".aaai");
    std::fs::create_dir_all(&base)?;
    Ok(base.join("profiles.yaml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_remove() {
        let mut store = ProfileStore::default();
        store.add(AuditProfile {
            name: "prod".into(), before: "/before".into(),
            after: "/after".into(), definition: None, ignore_file: None,
        });
        assert_eq!(store.profiles.len(), 1);
        store.remove("prod");
        assert!(store.profiles.is_empty());
    }

    #[test]
    fn add_replaces_same_name() {
        let mut store = ProfileStore::default();
        store.add(AuditProfile {
            name: "p".into(), before: "/a".into(),
            after: "/b".into(), definition: None, ignore_file: None,
        });
        store.add(AuditProfile {
            name: "p".into(), before: "/x".into(),
            after: "/y".into(), definition: None, ignore_file: None,
        });
        assert_eq!(store.profiles.len(), 1);
        assert_eq!(store.profiles[0].before, "/x");
    }
}
