use super::*;

#[test]
fn round_trip_yaml() {
    let prefs = UserPrefs { theme: Theme::Dark, ..Default::default() };
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
    assert!(!p.global_ignored_dirs.is_empty(), "default dirs should be applied");
    assert_eq!(p.language, "", "absent language should be empty string");
}
