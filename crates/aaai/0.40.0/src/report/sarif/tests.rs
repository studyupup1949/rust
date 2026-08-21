use super::*;
use crate::audit::result::{AuditResult, FileAuditResult, AuditStatus};
use crate::diff::entry::{DiffEntry, DiffType};

fn dummy_diff(path: &str, diff_type: DiffType) -> DiffEntry {
    DiffEntry {
        path: path.to_string(), diff_type, is_dir: false,
        before_text: None, after_text: None,
        is_binary: false,
        before_size: None, after_size: None,
        before_sha256: None, after_sha256: None,
        stats: None, error_detail: None,
    }
}

#[test]
fn sarif_output_is_valid_json_with_schema() {
    let results = vec![
        FileAuditResult {
            diff: dummy_diff("fail.txt", DiffType::Modified),
            entry: None,
            status: AuditStatus::Failed,
            detail: Some("strategy failed".into()),
            warnings: Vec::new(),
        },
        FileAuditResult {
            diff: dummy_diff("ok.txt", DiffType::Added),
            entry: None,
            status: AuditStatus::Ok,
            detail: None,
            warnings: Vec::new(),
        },
    ];
    let audit_result = AuditResult::new(results);
    let sarif = build_sarif(
        &audit_result,
        Path::new("/before"),
        Path::new("/after"),
    );

    assert_eq!(sarif["version"], "2.1.0");
    let run_results = &sarif["runs"][0]["results"];
    assert_eq!(run_results.as_array().unwrap().len(), 1,
        "only Failed/Pending/Error go into SARIF results");
}

#[test]
fn pending_maps_to_warning() {
    let results = vec![FileAuditResult {
        diff: dummy_diff("p.txt", DiffType::Added),
        entry: None,
        status: AuditStatus::Pending,
        detail: Some("no rule".into()),
        warnings: Vec::new(),
    }];
    let sarif = build_sarif(
        &AuditResult::new(results),
        Path::new("/b"), Path::new("/a"),
    );
    assert_eq!(sarif["runs"][0]["results"][0]["level"], "warning");
}
