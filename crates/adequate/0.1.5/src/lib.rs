#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate strfmt;

mod error;
pub use error::Error;

mod feedback;
pub use feedback::Feedback;

mod message;
pub use message::Message;
pub use message::msgfmt;

pub mod validation;

/// validate! macro validates given fields and its inputs.
///
/// ## Example
///
/// ```rust
/// # #[macro_use]
/// # extern crate adequate;
///
/// # use adequate::{Error, Feedback, Message};
/// # use adequate::validation::length;
///
/// # fn main() {
///     let name = "lorem ipsum".to_string();
///     let description = Some("lorem ipsum dolor sit amet".to_string());
///
///     let result = validate! {
///         "name" => name => [length::max(3)]
///     };
///     assert!(result.is_err());
///
///     let Error(out) = result.unwrap_err();
///     assert_eq!(vec![
///         Feedback {
///             field: "name",
///             messages: vec![
///                 Message {
///                   id: "validation.length.max",
///                   text: None,
///                   args: vec!["3".to_string()]
///                 }
///             ]
///         }
///     ], out);
///
///     let result = validate! {
///         "name" => name => [
///             length::max(64)
///         ],
///         "description" => description => [
///             length::max_if_present(255)
///         ]
///     };
///     assert!(result.is_ok());
/// # }
/// ```
#[macro_export]
macro_rules! validate {
    ( $( $n:expr => $v:expr => [ $( $c:expr ),* ] ),* ) => {{
        let errors = [$(
            Feedback {
                field: $n,
                messages: [ $( $c(&$v) ),* ]
                    .iter()
                    .cloned()
                    .filter_map(|c| c.err())
                    .collect::<Vec<_>>()
            }
        ),*]
            .iter()
            .cloned()
            .filter(|f| f.is_negative())
            .collect::<Vec<_>>();
        if !errors.is_empty() {
            Err(Error(errors))
        } else {
            Ok(())
        }
    }};
}

#[cfg(test)]
mod test {
    use super::*;
    use super::validation::Validator;

    #[test]
    fn test_message() {
        let m = Message {
            id: "validation.test",
            text: Some("lorem ipsum"),
            args: Vec::new(),
        };
        assert_eq!(m.to_string(), "lorem ipsum");

        let m = Message {
            id: "validation.test",
            text: Some("lorem {0}"),
            args: vec!["ipsum".to_string()],
        };
        assert_eq!(m.to_string(), "lorem ipsum");
    }

    #[test]
    fn test_message_without_text() {
        let m = Message {
            id: "validation.test",

            ..Default::default()
        };
        assert_eq!("validation.test", m.to_string());

        let m = Message {
            id: "validation.test",
            text: Some("lorem {0}"),
            args: vec!["ipsum".to_string()],
        };
        assert_eq!(m.to_string(), "lorem ipsum");
    }

    #[test]
    #[should_panic]
    fn test_message_panic_with_non_numeric_tmpl_ident() {
        let m = Message {
            id: "validation.test",
            text: Some("lorem ipsum {}"),
            args: vec!["dolor".to_string()],
        };
        m.to_string();
    }

    #[test]
    #[should_panic]
    fn test_message_panic_with_missing_ident() {
        let m = Message {
            id: "validation.test",
            text: Some("lorem ipsum"),
            args: vec!["dolor".to_string()],
        };
        m.to_string();
    }

    #[test]
    #[should_panic]
    fn test_message_panic_with_missing_arg() {
        let m = Message {
            id: "validation.test",
            text: Some("lorem ipsum {0} {1}"),
            args: vec!["dolor".to_string()],
        };
        m.to_string();
    }

    #[test]
    fn test_feedback_with_positive_result() {
        let f = Feedback {
            field: "dummy",
            messages: vec![],
        };
        assert!(!f.is_negative());
    }

    #[test]
    fn test_feedback_with_negative_result() {
        let m = Message {
            id: "validation.test",
            text: Some("lorem ipsum {0}"),
            args: vec!["dolor".to_string()],
        };
        let f = Feedback {
            field: "dummy",
            messages: vec![m],
        };
        assert!(f.is_negative());
    }

    #[test]
    fn test_failure() {
        let dummy = "".to_string();
        let validation = || -> Box<Validator> {
            Box::new(move |_: &String| {
                Err(Message {
                    id: "dummy",
                    text: Some("Error"),
                    args: vec![],
                })
            })
        };

        let result = validate! {
            "input" => dummy => [validation()]
        };
        assert!(result.is_err());
    }

    #[test]
    fn test_success() {
        let dummy = "".to_string();
        let validation =
            || -> Box<Validator> { Box::new(move |_: &String| Ok(())) };

        let result = validate! {
            "input" => dummy => [validation()]
        };
        assert!(result.is_ok());
    }
}
