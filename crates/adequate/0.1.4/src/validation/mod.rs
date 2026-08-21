use std::collections::HashMap;
use std::result::Result;

use crate::Message;
use crate::message::MESSAGES;

type ValidationResult = Result<(), Message>;

pub type Validator = dyn Fn(&String) -> ValidationResult;
pub type OptionalValidator = dyn Fn(&Option<String>) -> ValidationResult;

pub mod contain;
pub mod length;

fn make_error(key: &str, args: Vec<String>) -> ValidationResult {
    let m: HashMap<&str, &str> = MESSAGES.iter().cloned().collect();
    Err(Message {
        text: m.get(key).unwrap_or(&""),
        args,
    })
}

fn make_result(
    has_err: bool,
    key: &str,
    args: Vec<String>,
) -> ValidationResult {
    if has_err {
        make_error(key, args)
    } else {
        Ok(())
    }
}
