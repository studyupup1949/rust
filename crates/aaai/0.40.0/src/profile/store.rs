//! Audit profiles — named before/after/definition combos saved to
//! OS config directory — `aaai/profiles.yaml`.

use chrono::{DateTime, Utc};
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
    /// RFC 023 §3.2: last-used timestamp for "Recent projects" ordering.
    /// `#[serde(default)]` keeps legacy profile files
    /// (without this field) loading cleanly — they appear as `None`,
    /// which `sorted_by_recent` treats as the oldest entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Root document for `aaai/profiles.yaml` in the OS config directory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileStore {
    #[serde(default)]
    pub profiles: Vec<AuditProfile>,
}

impl ProfileStore {
    /// Load from the OS config directory, returning an empty store if absent.
    pub fn load() -> anyhow::Result<Self> {
        let path = profile_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)?;
        Ok(serde_yaml::from_str(&text)?)
    }

    /// Save to the OS config directory.
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

    /// RFC 023 §3.2: stamp the profile's `last_used_at` to now and persist.
    ///
    /// Looks up by name (the same key `add()` uses for replace-on-match).
    /// Returns `Ok(true)` if a profile with that name was found and
    /// updated, `Ok(false)` if no such profile exists (the store is left
    /// unchanged). Errors only on I/O during save.
    pub fn touch(&mut self, name: &str) -> anyhow::Result<bool> {
        if let Some(p) = self.profiles.iter_mut().find(|p| p.name == name) {
            p.last_used_at = Some(Utc::now());
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// RFC 023 §3.2: profiles sorted by `last_used_at` descending.
    /// Profiles without a `last_used_at` (legacy entries) come last,
    /// in their original insertion order relative to each other.
    pub fn sorted_by_recent(&self) -> Vec<&AuditProfile> {
        let mut v: Vec<&AuditProfile> = self.profiles.iter().collect();
        // `Option<DateTime<Utc>>` orders `None < Some(_)`, so reverse-cmp
        // pushes `None` to the end naturally.
        v.sort_by(|a, b| b.last_used_at.cmp(&a.last_used_at));
        v
    }
}

fn profile_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine OS config directory"))?
        .join("aaai");
    std::fs::create_dir_all(&base)?;
    Ok(base.join("profiles.yaml"))
}

#[cfg(test)]
mod tests;
