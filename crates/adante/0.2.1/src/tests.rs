use crate::{ArgumentType, Arguments, Error};

#[derive(Debug, Clone, Copy)]
pub enum TestErrorType {
    Syntax,
    FlagVal,
    NoFlagVal,
    NotRecognized,
}
impl Error for TestErrorType {
    fn handle(&self) {
        println!("{:?}", self);
        std::process::exit(1);
    }
    fn as_str(&self) -> &str {
        match self {
            Self::FlagVal => "No associated value for given flag",
            Self::NoFlagVal => "Need associated value for given flag",
            Self::Syntax => "Improper syntax usage",
            Self::NotRecognized => "Action or flag is not recognized",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TestFlagType {
    Help,
    Verbose,
    Print,
    TestFail, // NOTE: For testing only, use instead of std::process::exit
}
impl ArgumentType for TestFlagType {
    fn from_str<TestErrorType>(key: &str, error: TestErrorType)
                               -> Result<Self, TestErrorType> {
        match key {
            "-h" | "--help" => Ok(Self::Help),
            "-v" | "--verbose" => Ok(Self::Verbose),
            "-p" | "--print" => Ok(Self::Print),
            _ => Err(error),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TestActionType {
    Add,
    Remove,
    Edit,
    TestFail, // NOTE: For testing only, use instead of std::process::exit
}

impl ArgumentType for TestActionType {
    fn from_str<TestErrorType>(key: &str, error: TestErrorType) -> Result<Self, TestErrorType> {
        match key {
            "add" | "a" => Ok(Self::Add),
            "remove" | "r" => Ok(Self::Remove),
            "edit" | "e" => Ok(Self::Edit),
            _ => Err(error),
        }
    }
}

// "Simulates" running a program with arguments, collected by std::env::args::collect()
// NOTE: File path is omitted, would cause error as of 01-11
fn simulate(env_args: Vec<&str>) -> Result<Arguments<TestFlagType, TestActionType>, TestErrorType> {
    let _env_args: Arguments<TestFlagType, TestActionType> =
        return match Arguments::parse(env_args, TestErrorType::Syntax) {
            Ok(a) => Ok(a),
            Err(e) => Err(e),
        };
}

#[test]
fn parse_flag_key_from_str() {
    let env_args = match simulate(vec!["-v"]) {
        Ok(a) => a, Err(_) => Arguments::new()
    };
    assert_eq!(env_args.flags[0].key, TestFlagType::Verbose)
}

#[test]
fn parse_flag_val_from_str() {
    let env_args = match simulate(vec!["-h=test"]) {
        Ok(a) => a, Err(_) => Arguments::new()
    };
    assert_eq!(env_args.flags[0].value, Some("test".to_string()));
}

#[test]
fn no_misinterpret_flag_as_action() {
    let env_args = match simulate(vec!["-v"]) {
        Ok(a) => a, Err(_) => Arguments::new()
    };
    assert_eq!(env_args.actions.len(), 0);
}

#[test]
fn parse_noval_flag() {
    let env_args = match simulate(vec!["-v"]) {
        Ok(a) => a, Err(_) => Arguments::new()
    };
    assert_eq!(env_args.flags[0].key, TestFlagType::Verbose);
    assert_eq!(env_args.flags[0].value, None);
    assert_eq!(env_args.actions.len(), 0);
}

// FIXME: FAILS
#[test]
fn parse_val_flag() {
    let env_args = match simulate(vec!["-h=test"]) {
        Ok(a) => a, Err(_) => Arguments::new()
    };
    assert_eq!(env_args.flags[0].key, TestFlagType::Help);
    assert_eq!(env_args.flags[0].value, Some("test".to_string()));
    assert_eq!(env_args.actions.len(), 0);
}

#[test]
fn flag_from_str() {
    let ref_string = "-v";
    let result = match TestFlagType::from_str(ref_string, TestErrorType::Syntax) {
        Ok(t) => t,
        Err(_) => TestFlagType::TestFail,
    };
    assert_eq!(result, TestFlagType::Verbose)
}

#[test]
fn action_from_str() {
    let ref_string = "add";
    let result = match TestActionType::from_str(ref_string, TestErrorType::Syntax) {
        Ok(t) => t,
        Err(_) => TestActionType::TestFail,
    };
    assert_eq!(result, TestActionType::Add)
}

#[test]
fn parse_action_from_str() {
    let env_args = match simulate(vec!["add"]) {
        Ok(a) => a, Err(_) => Arguments::new()
    };
    assert_eq!(env_args.actions[0], TestActionType::Add)
}

#[test]
fn no_misinterpret_action_as_flag() {
    let env_args = match simulate(vec!["add"]) {
        Ok(a) => a, Err(_) => Arguments::new()
    };
    assert_eq!(env_args.flags.len(), 0)
}

#[test]
fn parse_action() {
    let env_args = match simulate(vec!["add"]) {
        Ok(a) => a, Err(_) => Arguments::new()
    };
    assert_eq!(env_args.actions[0], TestActionType::Add);
    assert_eq!(env_args.actions.len(), 1);
    assert_eq!(env_args.flags.len(), 0);
}
