use super::*;

#[test]
fn round_trip_yaml() {
    let cfg = ProjectConfig {
        version: "1".into(),
        default_definition: Some("audit/audit.yaml".into()),
        approver_name: Some("alice".into()),
        mask_secrets: true,
        custom_mask_patterns: vec!["PATTERN_[A-Z]+".into()],
        ..Default::default()
    };
    let yaml = serde_yaml::to_string(&cfg).unwrap();
    let restored: ProjectConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(restored.approver_name.as_deref(), Some("alice"));
    assert!(restored.mask_secrets);
    assert_eq!(restored.custom_mask_patterns.len(), 1);
}

#[test]
fn load_nonexistent_returns_none() {
    let result = ProjectConfig::load(Path::new("/nonexistent/.aaai.yaml")).unwrap();
    assert!(result.is_none());
}
