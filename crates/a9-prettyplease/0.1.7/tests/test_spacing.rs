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

#[test]
fn blank_line_before_cfg_use() {
    // After sorting: unconditionals first, then cfg-gated.
    // a9-prettyplease should add a blank line before the first #[cfg] use.
    let src = "use a::A;\nuse b::B;\n#[cfg(feature = \"x\")]\nuse c::C;\n";
    let file = syn::parse_file(src).unwrap();
    let out = a9_prettyplease::unparse(&file);
    eprintln!("output:\n{out}");
    // There should be a blank line before #[cfg]
    assert!(out.contains("\n\n#[cfg"), "expected blank line before cfg use:\n{out}");
}
