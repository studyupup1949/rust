use super::*;
use crate::audit::result::AuditSummary;
use crate::user_state::UserStatePaths;

fn make_record() -> HistoryRecord {
    HistoryRecord::new(
        std::path::Path::new("/before"),
        std::path::Path::new("/after"),
        None,
        &AuditSummary {
            total: 3,
            ok: 2,
            pending: 1,
            ..Default::default()
        },
    )
}

#[test]
fn round_trip_json() {
    let r = make_record();
    let json = serde_json::to_string(&r).unwrap();
    let restored: HistoryRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.result, r.result);
    assert_eq!(restored.total, 3);
}

#[test]
fn missing_history_does_not_create_state_root() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("state");
    let paths = UserStatePaths::from_root(root.clone()).unwrap();

    assert!(load_all_in(&paths).unwrap().is_empty());
    assert!(!root.exists());
}

#[test]
fn append_and_load_use_the_explicit_state_root() {
    let temp = tempfile::tempdir().unwrap();
    let paths = UserStatePaths::from_root(temp.path().join("state")).unwrap();
    let first = make_record();
    let mut second = make_record();
    second.before = "/second".into();

    append_in(&paths, &first).unwrap();
    append_in(&paths, &second).unwrap();

    let records = load_all_in(&paths).unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].before, "/second");
    assert_eq!(records[1].before, "/before");
}

#[test]
fn prune_keeps_the_three_newest_records() {
    let temp = tempfile::tempdir().unwrap();
    let paths = UserStatePaths::from_root(temp.path().join("state")).unwrap();
    for index in 0..5 {
        let mut record = make_record();
        record.before = format!("/{index}");
        append_in(&paths, &record).unwrap();
    }

    assert_eq!(prune_in(&paths, 3).unwrap(), 2);
    let records = load_all_in(&paths).unwrap();
    let before: Vec<_> = records
        .iter()
        .map(|record| record.before.as_str())
        .collect();
    assert_eq!(before, ["/4", "/3", "/2"]);
}
