use super::*;
use crate::user_state::UserStatePaths;

#[test]
fn round_trip_yaml() {
    let prefs = UserPrefs {
        theme: Theme::Dark,
        ..Default::default()
    };
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

// RFC 036 ────────────────────────────────────────────────────────

#[test]
fn new_fields_round_trip() {
    let p = UserPrefs {
        language: "ja".into(),
        global_ignored_dirs: vec![".git".into(), "target".into()],
        ..Default::default()
    };
    let yaml = serde_yaml::to_string(&p).unwrap();
    let r: UserPrefs = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(r.language, "ja");
    assert_eq!(r.global_ignored_dirs, vec![".git", "target"]);
}

#[test]
fn missing_fields_get_defaults() {
    // Simulate an old prefs.yaml that predates RFC 036 fields.
    let yaml = "theme: light\n";
    let p: UserPrefs = serde_yaml::from_str(yaml).unwrap();
    assert!(
        !p.global_ignored_dirs.is_empty(),
        "default dirs should be applied"
    );
    assert_eq!(p.language, "", "absent language should be empty string");
}

#[test]
fn missing_prefs_do_not_create_state_root() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("state");
    let paths = UserStatePaths::from_root(root.clone()).unwrap();

    assert_eq!(UserPrefs::load_from(&paths).unwrap().theme, Theme::Light);
    assert!(!root.exists());
}

#[test]
fn save_and_load_use_the_explicit_state_root() {
    let temp = tempfile::tempdir().unwrap();
    let paths = UserStatePaths::from_root(temp.path().join("state")).unwrap();
    let prefs = UserPrefs {
        theme: Theme::Dark,
        language: "ja".into(),
        ..Default::default()
    };

    prefs.save_to(&paths).unwrap();
    let loaded = UserPrefs::load_from(&paths).unwrap();
    assert_eq!(loaded.theme, Theme::Dark);
    assert_eq!(loaded.language, "ja");
}
