use std::collections::HashMap;
use std::result::Result;

use crate::Message;
use crate::message::MESSAGES;

type ValidationResult = Result<(), Message>;

pub type Validator = dyn Fn(&String) -> ValidationResult;
pub type OptionalValidator = dyn Fn(&Option<String>) -> ValidationResult;

pub mod contain;
pub mod length;

fn make_error<T>(key: &str, args: Vec<T>) -> ValidationResult
where
    T: ToString,
{
    let m: HashMap<&'static str, &'static str> =
        MESSAGES.iter().cloned().collect();
    Err(Message {
        id: m.get(key).unwrap_or(&""),
        text: None,
        args: args.iter().map(|a| a.to_string()).collect(),
    })
}

fn make_result<T>(has_err: bool, key: &str, args: Vec<T>) -> ValidationResult
where
    T: ToString,
{
    if has_err {
        make_error(key, args)
    } else {
        Ok(())
    }
}
