use std::collections::HashMap;
use std::fmt;

use serde::Serialize;
use strfmt::strfmt;

/// Message struct holds the validation error message keys and its args for
/// interpolation.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Message {
    pub id: &'static str,
    pub text: Option<&'static str>,
    pub args: Vec<String>,
}

/// An utility function for translation by using the optional text attribute.
///
/// ## Example
///
/// ```rust
/// # #[macro_use]
/// # extern crate adequate;
///
/// # use adequate::{Message, msgfmt};
/// # use adequate::validation::length;
///
/// # fn main() {
///     let msg = Message {
///       id: "lorem.ipsum",
///       text: Some("Lorem {0} dolor {1} amet"),
///       args: vec!["ipsum".to_string(), "sit".to_string()],
///     };
///     let out = msgfmt(&msg);
///     assert_eq!("Lorem ipsum dolor sit amet", out);
///
///     // it's used also for fmt::Display::fmt()
///     assert_eq!(msg.to_string(), out);
/// # }
pub fn msgfmt(m: &Message) -> String {
    match &m.text {
        None => m.id.to_string(),
        Some(txt) => {
            let mut args = HashMap::new();
            for (i, a) in m.args.iter().enumerate() {
                args.insert(i.to_string(), a.to_string());
            }
            // panic if given txt value doesn't match
            let out = strfmt(txt, &args).expect("message format is invalid");
            if !args.is_empty() && txt == &out {
                panic!("message does not have expected number of identifiers");
            }
            out
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = msgfmt(self);
        write!(f, "{}", out)
    }
}

lazy_static! {
    pub static ref MESSAGES: [(&'static str, &'static str); 7] = [
        ("max", "validation.length.max"), // args {0}
        ("min", "validation.length.min"), // args {0}
        ("within", "validation.length.within"), // args {0}, {1}
        ("contains", "validation.contain.contains"), // args {0}
        ("contains_any", "validation.contain.contains_any"), // args {0}
        ("contains_only", "validation.contain.contains_only"), // args {0}
        ("not_contain", "validation.contain.not_contain"), // args {1}
    ];
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fmt_without_text() {
        let m = Message {
            id: "lorem.ipsum",
            text: None,
            args: vec!["dolor sit amet".to_string()],
        };
        assert_eq!("lorem.ipsum", format!("{}", m));
    }

    #[test]
    fn test_fmt() {
        let m = Message {
            id: "lorem.ipsum",
            text: Some("lorem ipsum {0}"),
            args: vec!["dolor sit amet".to_string()],
        };
        assert_eq!("lorem ipsum dolor sit amet".to_string(), format!("{}", m));
    }

    #[test]
    fn test_eq() {
        let a: Message = Default::default();
        assert!(a.eq(&a));

        let b = Message {
            text: Some("lorem ipsum {0}"),

            ..Default::default()
        };
        assert!(!a.eq(&b));

        let c = Message {
            id: "validation.id",

            ..Default::default()
        };
        assert!(!a.eq(&c));

        let d = Message {
            args: vec![
                "dolor".to_string(),
                "sit".to_string(),
                "amet".to_string(),
            ],

            ..Default::default()
        };
        assert!(!a.eq(&d));
    }
}
