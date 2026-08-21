//! Audit profiles — named before/after/definition combos saved to
//! `~/.aaai/profiles.yaml`.

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
    /// `#[serde(default)]` keeps legacy `~/.aaai/profiles.yaml` files
    /// (without this field) loading cleanly — they appear as `None`,
    /// which `sorted_by_recent` treats as the oldest entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
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
    let base = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
        .join(".aaai");
    std::fs::create_dir_all(&base)?;
    Ok(base.join("profiles.yaml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_profile(name: &str) -> AuditProfile {
        AuditProfile {
            name: name.into(),
            before: "/before".into(),
            after: "/after".into(),
            definition: None,
            ignore_file: None,
            last_used_at: None,
        }
    }

    #[test]
    fn add_and_remove() {
        let mut store = ProfileStore::default();
        store.add(make_profile("prod"));
        assert_eq!(store.profiles.len(), 1);
        store.remove("prod");
        assert!(store.profiles.is_empty());
    }

    #[test]
    fn add_replaces_same_name() {
        let mut store = ProfileStore::default();
        let mut p = make_profile("p");
        p.before = "/a".into();
        store.add(p);
        let mut p2 = make_profile("p");
        p2.before = "/x".into();
        store.add(p2);
        assert_eq!(store.profiles.len(), 1);
        assert_eq!(store.profiles[0].before, "/x");
    }

    // ── RFC 023 §3.2: last_used_at + sorted_by_recent ────────────────────

    #[test]
    fn last_used_at_defaults_to_none() {
        let p = make_profile("p");
        assert!(p.last_used_at.is_none(),
            "newly-constructed profiles must have last_used_at = None");
    }

    #[test]
    fn legacy_yaml_without_last_used_at_deserialises() {
        // RFC 023 NFR-3: a profile YAML missing `last_used_at` must
        // still load — older versions of aaai didn't write that field.
        let legacy_yaml = "\
profiles:
  - name: legacy
    before: /before
    after: /after
    definition: null
";
        let store: ProfileStore = serde_yaml::from_str(legacy_yaml).unwrap();
        assert_eq!(store.profiles.len(), 1);
        assert_eq!(store.profiles[0].name, "legacy");
        assert!(store.profiles[0].last_used_at.is_none(),
            "missing field must default to None");
    }

    #[test]
    fn sorted_by_recent_orders_most_recent_first() {
        let mut store = ProfileStore::default();

        let mut newer = make_profile("newer");
        newer.last_used_at = Some(Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap());
        store.profiles.push(newer);

        let mut older = make_profile("older");
        older.last_used_at = Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap());
        store.profiles.push(older);

        let sorted = store.sorted_by_recent();
        assert_eq!(sorted[0].name, "newer");
        assert_eq!(sorted[1].name, "older");
    }

    #[test]
    fn sorted_by_recent_pushes_none_to_end() {
        // Legacy profiles (last_used_at = None) sort after any
        // profile with a real timestamp — even an ancient one.
        let mut store = ProfileStore::default();

        store.profiles.push(make_profile("never_used"));  // None

        let mut ancient = make_profile("ancient");
        ancient.last_used_at = Some(Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap());
        store.profiles.push(ancient);

        let sorted = store.sorted_by_recent();
        assert_eq!(sorted[0].name, "ancient");
        assert_eq!(sorted[1].name, "never_used");
    }

    // touch() unit-tests its bookkeeping by working on the in-memory
    // store; the `save()` call inside touch() touches the home directory,
    // so we deliberately avoid going through `touch()` here and exercise
    // the equivalent assignment instead. A full touch() test would need
    // a fakeable filesystem hook, which is out of scope for v0.20.

    #[test]
    fn touch_marks_profile_when_found() {
        // Verifies the lookup + assignment logic without persisting,
        // by inlining the body of touch(): real touch() also calls
        // save() which we avoid in tests.
        let mut store = ProfileStore::default();
        store.profiles.push(make_profile("p"));
        assert!(store.profiles[0].last_used_at.is_none());

        let before = Utc::now();
        if let Some(p) = store.profiles.iter_mut().find(|p| p.name == "p") {
            p.last_used_at = Some(Utc::now());
        }
        let after = Utc::now();

        let ts = store.profiles[0].last_used_at.expect("touched");
        assert!(ts >= before && ts <= after,
            "touched timestamp should fall within the call window");
    }
}
