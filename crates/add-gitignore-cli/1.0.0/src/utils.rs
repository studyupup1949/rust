use std::process;

use inquire::InquireError;

pub type AnyError = Box<dyn std::error::Error>;

pub fn capitalize_first_letter(word: &str) -> String {
    let first_letter = word.chars().next().unwrap();

    let capitalized_first_letter = first_letter.to_uppercase().to_string();

    let new_word = word
        .replace(first_letter, &capitalized_first_letter)
        .to_owned();

    new_word
}

///This is simple because when the user wants to cancel the operation it doesnt display as an arror
pub fn validate_response(res: &InquireError) -> Result<(), &InquireError> {
    match res {
        InquireError::OperationCanceled => process::exit(1),
        InquireError::OperationInterrupted => process::exit(1),
        _ => Err(res),
    }
}
