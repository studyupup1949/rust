use assert_cmd::Command;
use predicates::prelude::*;
use std::error::Error;

const PRG: &str = "acro";
const EMPTY: &str = "tests/inputs/empty.csv";
const DEFAULT_COLUMNS_FILE: &str = "tests/inputs/def.csv";
const DIFFERENT_COLUMNS_FILE: &str = "tests/inputs/diff.csv";
const WITH_HEADER_FILE: &str = "tests/inputs/with_header.csv";
const SEMICOLON_FILE: &str = "tests/inputs/semicolon.csv";

const NATO_RESULT: &str = " NATO: North Atlantic Treaty Organization\n";
const HEADER: &str = " acronym: definition";
const NATO_ACR: &str = "NAto";

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn usage() -> TestResult {
    for flag in &["-h", "--help"] {
        Command::cargo_bin(PRG)?
            .arg(flag)
            .assert()
            .stdout(predicate::str::contains("Usage"));
    }
    Ok(())
}

#[test]
fn empty() -> TestResult {
    run_acro(&[NATO_ACR, "-f", EMPTY], String::from(""))
}

#[test]
fn no_results() -> TestResult {
    run_acro(&["noresult", "-f", DEFAULT_COLUMNS_FILE], String::from(""))
}

#[test]
fn default_columns() -> TestResult {
    run_acro(
        &[NATO_ACR, "-f", DEFAULT_COLUMNS_FILE],
        String::from(NATO_RESULT),
    )
}

#[test]
fn partial_match() -> TestResult {
    run_acro(
        &["NAT", "-f", DEFAULT_COLUMNS_FILE],
        String::from(NATO_RESULT),
    )
}

#[test]
fn partial_match_2() -> TestResult {
    run_acro(
        &["AT", "-f", DEFAULT_COLUMNS_FILE],
        String::from(NATO_RESULT),
    )
}

#[test]
fn different_columns() -> TestResult {
    run_acro(
        &[NATO_ACR, "-f", DIFFERENT_COLUMNS_FILE, "-a", "2", "-d", "3"],
        String::from(NATO_RESULT),
    )
}

#[test]
fn all_arguments() -> TestResult {
    run_acro(
        &[NATO_ACR, "-f", DEFAULT_COLUMNS_FILE, "-a", "1", "-d", "2"],
        String::from(NATO_RESULT),
    )
}

#[test]
fn test_with_header() -> TestResult {
    run_acro(
        &["a", "-f", WITH_HEADER_FILE, "-H"],
        String::from(NATO_RESULT),
    )
}

#[test]
fn test_with_header_but_command_without() -> TestResult {
    run_acro(
        &["a", "-f", WITH_HEADER_FILE],
        String::from(format!("{}\n{}", HEADER, NATO_RESULT)),
    )
}

#[test]
fn test_with_semicolons() -> TestResult {
    run_acro(
        &["NATO", "-f", SEMICOLON_FILE, "-D", ";"],
        String::from(NATO_RESULT),
    )
}

fn run_acro(args: &[&str], expected: String) -> TestResult {
    Command::cargo_bin(PRG)?
        .args(args)
        .assert()
        .success()
        .stdout(expected);
    Ok(())
}

