pub const DEFAULT_CHECKER: &str = r#"use std::env;
use std::fs;
use std::process::exit;

const OK: i32 = 0;
const WRONG_ANSWER: i32 = 1;
const _PRESENTATION_ERROR: i32 = 2;
const FAIL: i32 = 3;

fn die(code: i32, msg: &str) -> ! {
    eprintln!("{msg}");
    exit(code);
}

fn check(output: String, answer: String) {
    if output.trim_end() == answer.trim_end() {
        die(
            OK,
            &format!("OK: `{}` and `{}`", output.trim_end(), answer.trim_end()),
        )
    } else {
        die(
            WRONG_ANSWER,
            &format!(
                "Wrong answer: `{}` and `{}`",
                output.trim_end(),
                answer.trim_end()
            ),
        )
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        die(FAIL, "Checker takes 4 arguments");
    }

    let _input_path = &args[1];
    let output_path = &args[2];
    let answer_path = &args[3];

    let output =
        fs::read_to_string(output_path).unwrap_or_else(|_| die(WRONG_ANSWER, "Cannot read output"));
    let answer =
        fs::read_to_string(answer_path).unwrap_or_else(|_| die(FAIL, "Cannot read answer"));

    check(output, answer);
}
"#;

pub const BASE_CONFIG: &str = r#"name = "<name>"
# owner_id = <owner id>
contest_id = <contest id>
problem_index = <problem's index in contest>
type = "default" # interactive/run_twice/run_twice_interactive/run_twice_first_interactive/run_twice_second_interactive
merge_subgroups = false
time_limit_ms = 1000
memory_limit_mb = 64
checker_path = "checker"
tests_path = "tests"

[[subgroups]]
type = "sample"
tests = [0]
score = 0
depends_on = []

[[subgroups]]
type = "main"
tests = [1]
score = 0
# score_per_test = 0
depends_on = [0]
"#;
