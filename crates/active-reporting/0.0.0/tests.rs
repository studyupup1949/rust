//^
//^ HEAD
//^

//> HEAD -> SUPER
use super::{
    Report,
    Same,
    Name
};


//^
//^ TESTS
//^

//> TESTS -> USAGE
#[test]
fn usage() -> () {
    let mut report = Report::new("Main");
    let passing = |report: Report<Same<'_>>| {
        report.issue("hello");
    };
    passing(report.to());
    let upgrading = |report: Report<Name<'_, "Category">>| {
        report.issue("hello");
    };
    upgrading(report.to());
    report.issue("outside");
}