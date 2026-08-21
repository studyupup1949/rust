//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later
use common::compile_alfa_src;
use common::compile_alfa_srcs;
use pretty_assertions::assert_eq;

mod common;

// Integration tests for minimal examples.
//
// This primarily handles sections 4.4 (comments) and 4.5 (namespaces)
// of alfa-for-xacml-v1.0-csd01

// (reminder we can use include_str! for reading from files)

/// No input
#[test]
fn no_alfa() {
    let x = compile_alfa_srcs(vec![]);
    // compile to xacml.
    // expect no top policies, since none were defined.
    assert_eq!(x.len(), 0);
}

/// An empty alfa file.
#[test]
fn empty_alfa() {
    let x = compile_alfa_src("");
    // expect no top policies, since none were defined.
    assert_eq!(x.len(), 0);
}

/// Many empty alfa files.
#[test]
fn empty_alfas() {
    let x = compile_alfa_srcs(vec![String::new(), String::new(), String::new()]);
    // expect no top policies, since none were defined.
    assert_eq!(x.len(), 0);
}

/// Comments
#[test]
fn line_comments() {
    // single line comment with no ending newline
    compile_alfa_src("// a single line comment");
    // single line comment with leading spaces
    compile_alfa_src("    // a single line comment");
    // single line comment with leading tabs and spaces
    compile_alfa_src("\t  // a single line comment");
    // single line comment, with a newline
    compile_alfa_src("// a single line comment\n");
    // multiple single line comments, with interspersed whitespace.
    compile_alfa_src(
        r"// a single line comment with blank line after

// another line",
    );
}

/// Comments
#[test]
fn block_comments() {
    // block comment with no final newline
    compile_alfa_src("/* a block comment */");
    // block comment, with a newline
    compile_alfa_src("/* a single line comment */\n");
    // block comment extending multiple lines
    compile_alfa_src(
        r"/* a single line comment with blank line after

*/",
    );
    // multiple block comments
    compile_alfa_src(
        r"/* comment one */
  /* comment two, indented */

/* comment three*/",
    );
}

/// Mixed comments
#[test]
fn mixed_comments() {
    // block comment with no final newline
    compile_alfa_src(
        r"/* a block comment */
// line comment

/* more block */   // and line comment!
/* // foo */
",
    );
}

/// Unclosed block comment
#[test]
#[should_panic(expected = "compile failed")]
fn unclosed_block_comment_1() {
    compile_alfa_src("/* a block comment *");
}
#[test]
#[should_panic(expected = "compile failed")]
fn unclosed_block_comment_2() {
    compile_alfa_src("/* a block comment /");
}
#[test]
#[should_panic(expected = "compile failed")]
fn unclosed_block_comment_3() {
    compile_alfa_src("a block comment */");
}

/// Empty namespace
#[test]
fn empty_namespace() {
    compile_alfa_src("namespace foo {}");
    // no space beteen name and def
    compile_alfa_src("namespace foo{}");
    // multiple namespaces on one line
    compile_alfa_src("namespace foo {} namespace bar{}");
}

/// Namespaces and Comments
#[test]
fn namespace_and_comments() {
    compile_alfa_src("// single comment\nnamespace foo {}");
    // no space beteen name and def
    compile_alfa_src("/* block prefix */ namespace foo{}");
    // multiple namespaces on one line
    compile_alfa_src("namespace foo {} /* in between */ namespace bar{}");
}

/// Nested Namespaces
#[test]
fn nested_namespace() {
    // namespaces inside of namespaces with varying spacing, on one line.
    compile_alfa_src("namespace foo { namespace bar { } }");
    compile_alfa_src("namespace foo { namespace bar { } namespace baz{}}");
    compile_alfa_src("namespace foo { namespace bar { /* comment */ } }");
    // multiple lines
    compile_alfa_src(
        r"
namespace foo {
  namespace bar {
    namespace baz {
}}
}
",
    );
}

/// Namespace with unmatched brace
#[test]
#[should_panic(expected = "compile failed")]
fn nested_namespace_unmatched_braces() {
    // Missing one trailing brace to close the foo namespace
    compile_alfa_src("namespace foo { namespace bar { namespace baz { } }");
}

/// Namespace with extra brace
#[test]
#[should_panic(expected = "compile failed")]
fn nested_namespace_extra_braces() {
    // Missing one trailing brace to close the foo namespace
    compile_alfa_src("namespace foo { namespace bar { namespace baz { } } } }");
}

/// Dotted Namespace Identifiers
#[test]
fn dotted_namespace() {
    // Namespaces can be collapsed with dotted names
    compile_alfa_src("namespace foo.bar.baz { namespace qux { } }");
}

/// Valid Namespace Identifiers
#[test]
fn valid_namespace_ids() {
    // Namespaces can have numbers (non-leading)
    compile_alfa_src("namespace foo1 {}");
    // and underscores (leading)
    compile_alfa_src("namespace _foo {}");
    // and underscores (or non-leading)
    compile_alfa_src("namespace foo_bar {}");
    // and mixed
    compile_alfa_src("namespace __foo___bar__baz_33 {}");
}

/// Invalid Namespace Identifiers
#[test]
#[should_panic(expected = "compile failed")]
fn invalid_numeric_namespace() {
    // Cannot use numbers as the first char
    compile_alfa_src("namespace 1foo { }");
}
#[test]
#[should_panic(expected = "compile failed")]
fn invalid_dash_namespace() {
    // Cannot use dashes in a namespace
    compile_alfa_src("namespace foo-bar { }");
}
#[test]
#[should_panic(expected = "compile failed")]
fn invalid_symbol_namespace() {
    // Other symbols prohibited
    compile_alfa_src("namespace !foo { }");
}
