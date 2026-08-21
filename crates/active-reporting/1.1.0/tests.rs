//^
//^ HEAD
//^

//> HEAD -> FEATURES
#![feature(default_field_values)]

//> HEAD -> ACTIVE_REPORTING
use active_reporting::{
    Report,
    Root
};

//> HEAD -> SYSTEMIO
use systemio::{
    SystemIO,
    Update
};

//> HEAD -> SYSTEMSTD
use systemstd::System;

//> HEAD -> ISSUING
use issuing::{
    Issue,
    Severity
};


//^
//^ TESTS
//^

//> TESTS -> USAGE
#[test]
fn usage() -> () {
    let mut root = Root::default();
    let passing = |report: Report<"">| { // stays the same
        System::problem(Issue {
            name: "hello",
            severity: Severity::Warning,
            ..
        }, &*report).sync();
    };
    passing(root.to());
    let upgrading = |report: Report<"Category">| { // adds a category
        System::problem(Issue {
            name: "hello 2",
            severity: Severity::Warning,
            ..
        }, &*report).sync();
    };
    upgrading(root.to());
    System::problem(Issue {
        name: "outside",
        severity: Severity::Warning,
        ..
    }, &*root).sync();
    System::clear().sync();
}