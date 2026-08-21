use super::*;
use crate::audit::result::AuditSummary;

fn make_record() -> HistoryRecord {
    HistoryRecord::new(
        std::path::Path::new("/before"),
        std::path::Path::new("/after"),
        None,
        &AuditSummary { total: 3, ok: 2, pending: 1, ..Default::default() },
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
