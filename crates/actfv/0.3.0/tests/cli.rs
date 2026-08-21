use assert_cmd::Command;
use file_diff::diff;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn valid_input_valid_output() -> TestResult {
    let mut cmd = Command::cargo_bin("actfv").unwrap();
    cmd.args(vec![
        "tests/resources/sample-aws-credentials",
        "../target/output.tfvars",
        "--profile",
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
fn valid_input_with_region_name_override_valid_output() -> TestResult {
    let mut cmd = Command::cargo_bin("actfv").unwrap();
    cmd.args(vec![
        "tests/resources/sample-aws-credentials",
        "../target/output_region_name_override.tfvars",
        "--profile",
        "adfs",
        "--region-name-override",
        "aws_region",
    ])
    .assert()
    .success();
    assert!(diff(
        "../target/output_region_name_override.tfvars",
        "tests/resources/valid_region_name_override.tfvars"
    ));
    Ok(())
}

#[test]
fn shows_usage() {
    let mut cmd = Command::cargo_bin("actfv").unwrap();
    cmd.assert().failure();
    // .stderr("Usage: actfv <aws_credentials_file> <output_tfvars_file> <profile>\n");
}
