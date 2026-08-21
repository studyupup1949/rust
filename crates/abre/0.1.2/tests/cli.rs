use assert_cmd::Command;
use predicates::prelude::*;

fn abre() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("abre").unwrap()
}

// --- Collapse (default) ---

#[test]
fn collapse_paths() {
    abre()
        .write_stdin(
            "/home/user/proj/foo/src/main.rs\n\
             /home/user/proj/bar/src/main.rs\n\
             /home/user/proj/bar/src/lib.rs\n",
        )
        .assert()
        .success()
        .stdout(
            "/…/foo/src/main.rs\n\
             /…/bar/src/main.rs\n\
             /…/bar/src/lib.rs\n",
        );
}

#[test]
fn collapse_no_shared_prefix() {
    abre()
        .write_stdin("foo/bar\nbaz/qux\n")
        .assert()
        .success()
        .stdout("foo/bar\nbaz/qux\n");
}

#[test]
fn collapse_single_line_unchanged() {
    abre()
        .write_stdin("/home/user/proj/foo/main.rs\n")
        .assert()
        .success()
        .stdout("/home/user/proj/foo/main.rs\n");
}

#[test]
fn collapse_identical_lines_unchanged() {
    abre()
        .write_stdin("a/b/c\na/b/c\n")
        .assert()
        .success()
        .stdout("a/b/c\na/b/c\n");
}

#[test]
fn collapse_mid_path() {
    abre()
        .write_stdin(
            "/org/frontend/issues/42\n\
             /org/frontend/pulls/15\n\
             /org/backend/actions/runs/99\n\
             /team/infra/merge_requests/7\n",
        )
        .assert()
        .success()
        .stdout(
            "/org/frontend/issues/42\n\
             /org/frontend/pulls/15\n\
             /org/backend/…/99\n\
             /team/…/7\n",
        );
}

#[test]
fn collapse_empty_input() {
    abre().write_stdin("").assert().success().stdout("");
}

// --- Suffix ---

#[test]
fn suffix_paths() {
    abre()
        .args(["--suffix"])
        .write_stdin(
            "/home/user/proj/foo/src/main.rs\n\
             /home/user/proj/bar/src/main.rs\n\
             /home/user/proj/bar/src/lib.rs\n",
        )
        .assert()
        .success()
        .stdout(
            "/…/foo/src/main.rs\n\
             /…/bar/src/main.rs\n\
             /…/lib.rs\n",
        );
}

#[test]
fn suffix_all_unique_leaves() {
    abre()
        .args(["--suffix"])
        .write_stdin("a/b/x\na/b/y\na/b/z\n")
        .assert()
        .success()
        .stdout("…/x\n…/y\n…/z\n");
}

// --- Truncate ---

#[test]
fn truncate_default_n() {
    abre()
        .args(["--truncate"])
        .write_stdin(
            "/home/user/proj/foo/src/main.rs\n\
             /home/user/proj/bar/src/main.rs\n",
        )
        .assert()
        .success()
        .stdout(
            "/h/u/p/foo/src/main.rs\n\
             /h/u/p/bar/src/main.rs\n",
        );
}

#[test]
fn truncate_custom_n() {
    abre()
        .args(["--truncate", "-n", "2"])
        .write_stdin(
            "/home/user/proj/foo/main.rs\n\
             /home/user/proj/bar/main.rs\n",
        )
        .assert()
        .success()
        .stdout(
            "/ho/us/pr/foo/main.rs\n\
             /ho/us/pr/bar/main.rs\n",
        );
}

// --- Separator ---

#[test]
fn dot_separator() {
    abre()
        .args(["-s", "."])
        .write_stdin("com.example.foo.Bar\ncom.example.bar.Baz\n")
        .assert()
        .success()
        .stdout("….foo.Bar\n….bar.Baz\n");
}

#[test]
fn dot_separator_truncate() {
    abre()
        .args(["-s", ".", "--truncate"])
        .write_stdin("com.example.foo.Bar\ncom.example.bar.Baz\n")
        .assert()
        .success()
        .stdout("c.e.foo.Bar\nc.e.bar.Baz\n");
}

// --- Ellipsis ---

#[test]
fn custom_ellipsis() {
    abre()
        .args(["--ellipsis", ".."])
        .write_stdin(
            "/home/user/proj/foo/main.rs\n\
             /home/user/proj/bar/main.rs\n",
        )
        .assert()
        .success()
        .stdout(
            "/../foo/main.rs\n\
             /../bar/main.rs\n",
        );
}

// --- Capture regex ---

#[test]
fn capture_regex() {
    abre()
        .args(["-c", r"https?://[^/]+(.*)"])
        .write_stdin(
            "https://example.com/a/b/c\n\
             https://example.com/a/b/d\n",
        )
        .assert()
        .success()
        .stdout(
            "https://example.com/…/c\n\
             https://example.com/…/d\n",
        );
}

#[test]
fn capture_no_match_passes_through() {
    abre()
        .args(["-c", r"https?://(.*)"])
        .write_stdin("not-a-url\nhttps://example.com/foo\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("not-a-url\n"));
}

// --- Presets ---

#[test]
fn preset_url_path() {
    abre()
        .args(["-p", "url-path"])
        .write_stdin(
            "https://github.com/org/repo/issues/1\n\
             https://github.com/org/repo/pulls/2\n",
        )
        .assert()
        .success()
        .stdout(
            "https://github.com/…/issues/1\n\
             https://github.com/…/pulls/2\n",
        );
}

#[test]
fn preset_docker() {
    abre()
        .args(["-p", "docker"])
        .write_stdin("registry.io/org/app:v1\nregistry.io/org/api:v2\n")
        .assert()
        .success()
        .stdout("…/app:v1\n…/api:v2\n");
}

#[test]
fn preset_unknown_fails() {
    abre()
        .args(["-p", "nope"])
        .write_stdin("x\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown preset"));
}

// --- JSON mode ---

#[test]
fn json_modify_in_place() {
    abre()
        .args(["--json", "-k", "path"])
        .write_stdin(
            r#"{"id":1,"path":"/a/b/c/x"}
{"id":2,"path":"/a/b/c/y"}
"#,
        )
        .assert()
        .success()
        .stdout(
            predicate::str::contains(r#""path":"/…/x""#)
                .and(predicate::str::contains(r#""path":"/…/y""#)),
        );
}

#[test]
fn json_add_key() {
    abre()
        .args(["--json", "-k", "path", "--add-key", "short"])
        .write_stdin(
            r#"{"id":1,"path":"/a/b/c/x"}
{"id":2,"path":"/a/b/c/y"}
"#,
        )
        .assert()
        .success()
        .stdout(
            predicate::str::contains(r#""short":"/…/x""#)
                .and(predicate::str::contains(r#""path":"/a/b/c/x""#)),
        );
}

#[test]
fn json_keep_original() {
    abre()
        .args(["--json", "-k", "path", "--keep-original", "orig"])
        .write_stdin(
            r#"{"id":1,"path":"/a/b/c/x"}
{"id":2,"path":"/a/b/c/y"}
"#,
        )
        .assert()
        .success()
        .stdout(
            predicate::str::contains(r#""path":"/…/x""#)
                .and(predicate::str::contains(r#""orig":"/a/b/c/x""#)),
        );
}

#[test]
fn json_with_preset() {
    abre()
        .args(["--json", "-k", "url", "-p", "url-path"])
        .write_stdin(
            r#"{"url":"https://github.com/org/repo/issues/1"}
{"url":"https://github.com/org/repo/pulls/2"}
"#,
        )
        .assert()
        .success()
        .stdout(
            predicate::str::contains("issues/1")
                .and(predicate::str::contains("pulls/2"))
                .and(predicate::str::contains("https://github.com")),
        );
}

#[test]
fn json_missing_key_fails() {
    abre()
        .args(["--json", "-k", "nope"])
        .write_stdin(r#"{"foo":"bar"}"#)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn json_without_k_fails() {
    abre()
        .args(["--json"])
        .write_stdin(r#"{"foo":"bar"}"#)
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires -k"));
}
