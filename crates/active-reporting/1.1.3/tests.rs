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

//> HEAD -> SYSTEMSTD
use systemstd::System;

//> HEAD -> ISSUING
use issuing::Issue;


//^
//^ TESTS
//^

//> TESTS -> USAGE
#[test]
fn usage() -> () {
    let mut root = Root::default();
    let passing = |report: Report<"">| { // stays the same
        System::warning(Issue {
            name: "hello",
            ..
        }, &*report);
    };
    passing(root.to());
    let upgrading = |report: Report<"Category">| { // adds a category
        System::warning(Issue {
            name: "hello 2",
            ..
        }, &*report);
    };
    upgrading(root.to());
    System::warning(Issue {
        name: "outside",
        ..
    }, &*root);
}