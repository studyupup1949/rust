use super::support::{aaai, aaai_with_program};

#[test]
fn history_stats_reads_only_allowed_fixture() {
    let mut command = aaai();
    command.seed_history(5);
    let output = command.args(["history", "--stats"]).run_output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Total runs   : 5"));
    assert!(stdout.contains("Pass rate    : 60.0%  (3/5)"));
}

#[test]
fn history_prune_reduces_five_records_to_the_newest_three() {
    let mut command = aaai();
    command.seed_history(5);
    let output = command
        .args(["history", "--prune", "3"])
        .run_output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("removed 2")
    );

    let records = command.history_records();
    let retained: Vec<_> = records
        .iter()
        .map(|record| record["before"].as_str().unwrap())
        .collect();
    assert_eq!(
        retained,
        [
            "/allowed/before-2",
            "/allowed/before-3",
            "/allowed/before-4"
        ]
    );
}

#[test]
fn history_count_limits_output_to_three_allowed_records() {
    let mut command = aaai();
    command.seed_history(5);
    let output = command
        .args(["history", "--count", "3"])
        .run_output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("Before:").count(), 3);
    assert!(stdout.contains("/allowed/before-4"));
    assert!(!stdout.contains("/allowed/before-1"));
}

#[test]
fn history_json_output_is_an_allowed_fixture_array() {
    let mut command = aaai();
    command.seed_history(4);
    let output = command
        .args(["history", "--json-output", "--count", "4"])
        .run_output()
        .unwrap();
    assert!(output.status.success());
    let records: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(records.len(), 4);
    assert_eq!(records[0]["before"], "/allowed/before-3");
}

#[test]
fn empty_history_and_empty_stats_succeed_without_creating_state() {
    for arguments in [["history"].as_slice(), ["history", "--stats"].as_slice()] {
        let mut command = aaai();
        let output = command.args(arguments).run_output().unwrap();
        assert!(output.status.success());
        assert!(!command.state_root().exists());
    }
}

#[test]
fn status_api_rejects_canary_on_stdout() {
    let mut command = aaai();
    let path = command.synthetic_path_with_canary();
    let error = command
        .args(["config", "--init", "--dir"])
        .arg(path)
        .run_status()
        .unwrap_err();
    assert!(error.has_failure("stdout-disclosure"));
    assert!(!error.has_failure("fallback-mutation"));
    assert!(!error.to_string().contains("Created:"));
}

#[test]
fn output_api_rejects_canary_on_stdout() {
    let mut command = aaai();
    let path = command.synthetic_path_with_canary();
    let error = command
        .args(["config", "--init", "--dir"])
        .arg(path)
        .run_output()
        .unwrap_err();
    assert!(error.has_failure("stdout-disclosure"));
    assert!(!error.has_failure("fallback-mutation"));
    assert!(!error.to_string().contains("Created:"));
}

#[test]
fn status_api_rejects_canary_on_stderr() {
    let mut command = aaai();
    let canary = command.canary_id().to_owned();
    let error = command.arg(canary).run_status().unwrap_err();
    assert!(error.has_failure("stderr-disclosure"));
    assert!(!error.has_failure("fallback-mutation"));
    assert!(!error.to_string().contains("Usage:"));
}

#[test]
fn output_api_rejects_canary_on_stderr() {
    let mut command = aaai();
    let canary = command.canary_id().to_owned();
    let error = command.arg(canary).run_output().unwrap_err();
    assert!(error.has_failure("stderr-disclosure"));
    assert!(!error.has_failure("fallback-mutation"));
    assert!(!error.to_string().contains("Usage:"));
}

#[test]
fn fallback_mutation_is_reported_with_only_a_relative_path() {
    let mut command = aaai();
    command.mutate_fallback_for_test();
    let error = command.arg("--help").run_output().unwrap_err();
    let rendered = error.to_string();
    let expected_path = std::path::Path::new("fallback")
        .join("home")
        .join(".config")
        .join("aaai")
        .join("sentinel");
    assert!(error.has_failure("fallback-mutation"));
    assert!(rendered.contains(expected_path.to_string_lossy().as_ref()));
    assert!(!rendered.contains(command.state_root().parent().unwrap().to_str().unwrap()));
}

#[test]
fn successful_and_nonzero_children_keep_fallback_stores_unchanged() {
    let mut success = aaai();
    assert!(success.arg("--help").run_status().unwrap().success());

    let mut nonzero = aaai();
    assert!(
        !nonzero
            .arg("synthetic-invalid-subcommand")
            .run_status()
            .unwrap()
            .success()
    );
}

#[test]
fn spawn_failures_are_reported_without_operator_paths() {
    let mut command = aaai_with_program("aaai-rfc096-deliberately-missing-binary");
    let error = command.run_output().unwrap_err();
    assert!(error.has_failure("spawn-failure"));
    assert_eq!(error.to_string(), "isolated command failed: spawn-failure");
}

#[test]
fn an_unexecuted_command_keeps_fallback_stores_unchanged() {
    drop(aaai());
}
