use assert_cmd::Command;
use file_diff::diff;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn valid_input_valid_output() -> TestResult {
    let mut cmd = Command::cargo_bin("actfv").unwrap();
    cmd.args(vec![
        "tests/resources/sample-aws-credentials",
        "../target/output.tfvars",
        "adfs",
    ])
    .assert()
    .success();
    assert!(diff(
        "../target/output.tfvars",
        "tests/resources/valid.tfvars"
    ));
    Ok(())
}

#[test]
fn shows_usage() {
    let mut cmd = Command::cargo_bin("actfv").unwrap();
    cmd.assert()
        .failure()
        .stderr("Usage: actfv <aws_credentials_file> <output_tfvars_file> <profile>\n");
}
