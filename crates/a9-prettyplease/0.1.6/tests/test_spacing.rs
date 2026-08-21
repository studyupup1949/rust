#[test]
fn test_spacing_output() {
    let code = r#"
use std::collections::HashMap;
use std::io;
use serde::Deserialize;
use crate::config::Config;

fn main() {
    let x = 1;
    let y = 2;
    println!("{}", x + y);
    if x > 0 {
        let a = 3;
        let b = 4;
        println!("{}", a + b);
        do_something();
    }
}

struct Foo {
    x: i32,
}

impl Foo {
    fn new() -> Self {
        Foo { x: 0 }
    }
    fn get_x(&self) -> i32 {
        self.x
    }
    fn set_x(&mut self, x: i32) {
        self.x = x;
    }
}

fn helper() {
    println!("hello");
}
"#;
    let syntax_tree = syn::parse_file(code).unwrap();
    let formatted = a9_prettyplease::unparse(&syntax_tree);
    print!("{}", formatted);
}

// ---------------------------------------------------------------------------
// Tracing macro blank-line regression tests (Stmt::Macro variant)
// ---------------------------------------------------------------------------
//
// All tracing macros appear with a trailing semicolon, so syn parses them as
// `Stmt::Macro` — NOT `Stmt::Expr(Expr::Macro, _)`. The tests below guard
// against the Stmt::Macro blindspot where tracing attachment semantics were
// previously ignored.

/// `info!` is standalone: blank line before AND after.
#[test]
fn info_semicolon_is_standalone() {
    let code = r#"
fn main() {
    foo();
    info!("milestone reached");
    bar();
}
"#;
    let ast = syn::parse_file(code).expect("parse");
    let out = a9_prettyplease::unparse(&ast);
    // blank before info!
    assert!(
        out.contains("foo();\n\n    info!("),
        "expected blank before info!, got:\n{out}"
    );
    // blank after info!
    assert!(
        out.contains("info!(\"milestone reached\");\n\n    bar()"),
        "expected blank after info!, got:\n{out}"
    );
}

/// `warn!` is standalone: blank before AND after.
#[test]
fn warn_semicolon_is_standalone() {
    let code = r#"
fn main() {
    foo();
    warn!("something odd");
    bar();
}
"#;
    let ast = syn::parse_file(code).expect("parse");
    let out = a9_prettyplease::unparse(&ast);
    assert!(
        out.contains("foo();\n\n    warn!("),
        "expected blank before warn!, got:\n{out}"
    );
    assert!(
        out.contains("warn!(\"something odd\");\n\n    bar()"),
        "expected blank after warn!, got:\n{out}"
    );
}

/// `error!` is standalone: blank before AND after.
#[test]
fn error_semicolon_is_standalone() {
    let code = r#"
fn main() {
    foo();
    error!("fatal");
    bar();
}
"#;
    let ast = syn::parse_file(code).expect("parse");
    let out = a9_prettyplease::unparse(&ast);
    assert!(
        out.contains("foo();\n\n    error!("),
        "expected blank before error!, got:\n{out}"
    );
    assert!(
        out.contains("error!(\"fatal\");\n\n    bar()"),
        "expected blank after error!, got:\n{out}"
    );
}

/// `trace!` attaches FORWARD: blank before (detach from prev), no blank after.
#[test]
fn trace_semicolon_attaches_forward() {
    let code = r#"
fn main() {
    foo();
    trace!("about to connect");
    bar();
}
"#;
    let ast = syn::parse_file(code).expect("parse");
    let out = a9_prettyplease::unparse(&ast);
    // blank before trace (detach from prev)
    assert!(
        out.contains("foo();\n\n    trace!("),
        "expected blank before trace!, got:\n{out}"
    );
    // no blank after trace (attach to next)
    assert!(
        out.contains("trace!(\"about to connect\");\n    bar()"),
        "expected no blank after trace!, got:\n{out}"
    );
}

/// `debug!` attaches BACKWARD: no blank before, blank after (detach from next).
#[test]
fn debug_semicolon_attaches_backward() {
    let code = r#"
fn main() {
    foo();
    debug!("foo done");
    bar();
}
"#;
    let ast = syn::parse_file(code).expect("parse");
    let out = a9_prettyplease::unparse(&ast);
    // no blank before debug (attach to prev)
    assert!(
        out.contains("foo();\n    debug!("),
        "expected no blank before debug!, got:\n{out}"
    );
    // blank after debug (detach from next)
    assert!(
        out.contains("debug!(\"foo done\");\n\n    bar()"),
        "expected blank after debug!, got:\n{out}"
    );
}

/// Full pattern from diag_service.rs: Medium → info! → Medium cluster.
#[test]
fn diag_service_tail_pattern() {
    let code = r#"
fn main() {
    println!("GET_PUBLIC_KEY_OK pubkey={}", hex(&pubkey));
    info!("waiting 2s for holepunch upgrade...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    log_paths(&endpoint, provision_id, "provision").await;
    log_paths(&endpoint, gov_id, "governance").await;
    println!("DONE total={}ms", start.elapsed().as_millis());
    root_ctx.terminate().await;
    Ok(())
}
"#;
    let ast = syn::parse_file(code).expect("parse");
    let out = a9_prettyplease::unparse(&ast);
    // blank before info! (standalone: detach from println)
    assert!(
        out.contains("hex(&pubkey));\n\n    info!("),
        "expected blank before info!, got:\n{out}"
    );
    // blank after info! (standalone: detach from sleep)
    assert!(
        out.contains("info!(\"waiting 2s for holepunch upgrade...\");\n\n    tokio"),
        "expected blank after info!, got:\n{out}"
    );
    // Medium cluster: sleep + log_paths + log_paths + println + terminate are all Medium → no blanks between them
    assert!(
        out.contains("log_paths(&endpoint, provision_id, \"provision\").await;\n    log_paths"),
        "expected log_paths to cluster with no blank, got:\n{out}"
    );
}

// ---------------------------------------------------------------------------
// Heavy-static blank-line tests
//
// `static` items whose initialiser contains a closure or block body are
// "heavyweight" and must be separated by blank lines, just like `fn`/`impl`.
// Simple statics (literal / path inits) continue to cluster together.
// ---------------------------------------------------------------------------

/// Two simple statics should cluster (no blank between them).
#[test]
fn simple_statics_cluster() {
    let code = r#"
static FOO: &str = "foo";
static BAR: usize = 42;
"#;
    let out = a9_prettyplease::unparse(&syn::parse_file(code).unwrap());
    assert!(
        out.contains("\"foo\";\nstatic BAR"),
        "expected simple statics to cluster without blank, got:\n{out}"
    );
}

/// Two heavy statics (LazyLock::new closure) must have a blank line between them.
#[test]
fn heavy_statics_get_blank() {
    let code = r#"
use std::sync::LazyLock;
static FIRST: LazyLock<u32> = LazyLock::new(|| {
    let x = 1;
    x + 1
});
static SECOND: LazyLock<u32> = LazyLock::new(|| {
    let y = 2;
    y + 2
});
"#;
    let out = a9_prettyplease::unparse(&syn::parse_file(code).unwrap());
    assert!(
        out.contains("});\n\nstatic SECOND"),
        "expected blank line between heavy statics, got:\n{out}"
    );
}

/// A simple static followed by a heavy static also gets a blank.
#[test]
fn simple_then_heavy_static_gets_blank() {
    let code = r#"
use std::sync::LazyLock;
static SIMPLE: &str = "hello";
static HEAVY: LazyLock<u32> = LazyLock::new(|| {
    42
});
"#;
    let out = a9_prettyplease::unparse(&syn::parse_file(code).unwrap());
    assert!(
        out.contains("\"hello\";\n\nstatic HEAVY"),
        "expected blank line between simple and heavy static, got:\n{out}"
    );
}
