use super::*;
use crate::user_state::UserStatePaths;
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
    assert!(
        p.last_used_at.is_none(),
        "newly-constructed profiles must have last_used_at = None"
    );
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
    assert!(
        store.profiles[0].last_used_at.is_none(),
        "missing field must default to None"
    );
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

    store.profiles.push(make_profile("never_used")); // None

    let mut ancient = make_profile("ancient");
    ancient.last_used_at = Some(Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap());
    store.profiles.push(ancient);

    let sorted = store.sorted_by_recent();
    assert_eq!(sorted[0].name, "ancient");
    assert_eq!(sorted[1].name, "never_used");
}

#[test]
fn missing_profiles_do_not_create_state_root() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("state");
    let paths = UserStatePaths::from_root(root.clone()).unwrap();

    assert!(ProfileStore::load_from(&paths).unwrap().profiles.is_empty());
    assert!(!root.exists());
}

#[test]
fn save_and_load_use_the_explicit_state_root() {
    let temp = tempfile::tempdir().unwrap();
    let paths = UserStatePaths::from_root(temp.path().join("state")).unwrap();
    let mut store = ProfileStore::default();
    store.profiles.push(make_profile("p"));
    store.save_to(&paths).unwrap();

    let loaded = ProfileStore::load_from(&paths).unwrap();
    assert_eq!(loaded.profiles.len(), 1);
    assert_eq!(loaded.profiles[0].name, "p");
}

#[test]
fn touch_marks_and_persists_profile_in_the_explicit_state_root() {
    let temp = tempfile::tempdir().unwrap();
    let paths = UserStatePaths::from_root(temp.path().join("state")).unwrap();
    let mut store = ProfileStore::default();
    store.profiles.push(make_profile("p"));
    let before = Utc::now();
    assert!(store.touch_in(&paths, "p").unwrap());
    let after = Utc::now();

    let loaded = ProfileStore::load_from(&paths).unwrap();
    let ts = loaded.profiles[0].last_used_at.expect("touched");
    assert!(
        ts >= before && ts <= after,
        "touched timestamp should fall within the call window"
    );
}
