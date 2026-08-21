use crate::validation::{make_error, make_result, OptionalValidator, Validator};

const ALPHANUMERIC_CHARS: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D',
    'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9',
];

const DIGITS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

const LOWER_LETTERS: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const UPPER_LETTERS: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

/// Returns a function that checks if the given &str contains another &str.
pub fn contains(part: &'static str) -> Box<Validator> {
    Box::new(move |s: &str| {
        make_result(!s.contains(part), "contains", vec![part.to_string()])
    })
}

/// Returns a function that checks if the given &str contains optional &str.
pub fn contains_if_given(part: Option<&'static str>) -> Box<Validator> {
    let p = part.unwrap_or_default();
    Box::new(move |s: &str| {
        make_result(
            !p.is_empty() && !s.contains(p),
            "contains",
            vec![p.to_string()],
        )
    })
}

/// Returns a function that checks if the given &str contains only chars from
/// the accepted characters.
pub fn contains_only(accepted: &'static [char]) -> Box<Validator> {
    Box::new(move |s: &str| {
        for c in s.chars() {
            if !accepted.contains(&c) {
                return make_error("not_contain", vec![c.to_string()]);
            }
        }
        Ok(())
    })
}

/// Returns a function that checks if the given &str contains only alphanumeric
/// characters.
pub fn contains_only_alphanumeric_chars() -> Box<Validator> {
    contains_only(ALPHANUMERIC_CHARS)
}

/// Returns a function that checks if the given &str contains any of accepted
/// characters.
pub fn contains_any(
    accepted: &'static [char],
    text: &'static str,
) -> Box<Validator> {
    Box::new(move |s: &str| {
        for c in s.chars() {
            if accepted.contains(&c) {
                return Ok(());
            }
        }
        make_error("contains_any", vec![text.to_string()])
    })
}

/// Returns a function that checks if the given &str contains any of DIGITS
/// (0-9).
pub fn contains_any_digits() -> Box<Validator> {
    contains_any(DIGITS, "0-9")
}

/// Returns a function that checks if the given &str contains any of
/// LOWER_LETTERS (a-z).
pub fn contains_any_lower_letters() -> Box<Validator> {
    contains_any(LOWER_LETTERS, "a-z")
}

/// Returns a function that checks if the given &str contains any of
/// UPPER_LETTERS (A-Z).
pub fn contains_any_upper_letters() -> Box<Validator> {
    contains_any(UPPER_LETTERS, "A-Z")
}

/// Returns a function that checks if the given &str does not contain
/// optional &str.
pub fn not_contain_if_given(part: Option<&'static str>) -> Box<Validator> {
    let p = part.unwrap_or_default();
    Box::new(move |s: &str| {
        make_result(
            !p.is_empty() && s.contains(p),
            "not_contain",
            vec![p.to_string()],
        )
    })
}

/// Returns a function that checks if the given optional &str contains another
/// string when the one exists.
pub fn contains_if_present(part: &'static str) -> Box<OptionalValidator> {
    Box::new(move |s: Option<&str>| match s {
        Some(v) => contains(part)(v),
        None => Ok(()),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    // contains
    #[test]
    fn test_contains_ok() {
        let f = contains("lorem");
        let result = f("lorem ipsum");
        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_err() {
        let f = contains("dolor sit amet");
        let result = f("lorem ipsum");
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_err_message() {
        let f = contains("dolor sit amet");
        let result = f("lorem ipsum");
        assert_eq!(
            result.map_err(|e| e.to_string()),
            Err("Must contain dolor sit amet".to_string())
        );
    }

    // contains_if_given
    #[test]
    fn test_contains_if_given_ok() {
        let part = "lorem ipsum";

        let f = contains_if_given(None);
        let result = f(part);
        assert!(result.is_ok());

        let f = contains_if_given(Some(""));
        let result = f(part);
        assert!(result.is_ok());

        let f = contains_if_given(Some("lorem"));
        let result = f(part);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_if_given_err() {
        let f = contains_if_given(Some("dolor sit amet"));
        let result = f("lorem ipsum");
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_if_given_err_message() {
        let f = contains_if_given(Some("dolor sit amet"));
        let result = f("lorem ipsum");
        assert_eq!(
            result.map_err(|e| e.to_string()),
            Err("Must contain dolor sit amet".to_string())
        );
    }

    // contains_only
    #[test]
    fn test_contains_only_ok() {
        let part = "lorem ipsum";

        let f =
            contains_only(&[' ', 'e', 'i', 'l', 'm', 'o', 'p', 'r', 's', 'u']);
        let result = f(part);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_only_err() {
        let part = "lorem ipsum";

        let f =
            contains_only(&['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j']);
        let result = f(part);
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_only_err_message() {
        let part = "lorem ipsum";

        let f =
            contains_only(&['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j']);
        let result = f(part);
        assert_eq!(
            result.map_err(|e| e.to_string()),
            Err("Must not contain l".to_string())
        );
    }

    // contains_only_alphanumeric_chars
    #[test]
    fn test_contains_only_alphanumeric_chars_ok() {
        let part = "lorem123";

        let f = contains_only_alphanumeric_chars();
        let result = f(part);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_only_alphanumeric_chars_err() {
        let part = "lorem ipsum";

        let f = contains_only_alphanumeric_chars();
        let result = f(part);
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_only_alphanumeric_chars_err_message() {
        let part = "lorem-";

        let f = contains_only_alphanumeric_chars();
        let result = f(part);
        assert_eq!(
            result.map_err(|e| e.to_string()),
            Err("Must not contain -".to_string())
        );
    }

    // contains_any
    #[test]
    fn test_contains_any_ok() {
        let part = "lorem ipsum";

        let f = contains_any(&['a', 'b', 'c', 'd', 'e'], "a..=e");
        let result = f(part);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_any_err() {
        let part = "lorem ipsum";

        let f = contains_any(&['a', 'b', 'c', 'd'], "a..=e");
        let result = f(part);
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_any_err_message() {
        let part = "lorem";

        let f = contains_any(&['a', 'b', 'c', 'd'], "a..=e");
        let result = f(part);
        assert_eq!(
            result.map_err(|e| e.to_string()),
            Err("Must contain any of a..=e".to_string())
        );
    }

    // contains_any_digits
    #[test]
    fn test_contains_any_digits_ok() {
        let part = "l0rem";

        let f = contains_any_digits();
        let result = f(part);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_any_digits_err() {
        let part = "lorem ipsum";

        let f = contains_any_digits();
        let result = f(part);
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_any_digits_err_message() {
        let part = "lorem";

        let f = contains_any_digits();
        let result = f(part);
        assert_eq!(
            result.map_err(|e| e.to_string()),
            Err("Must contain any of 0-9".to_string())
        );
    }

    // contains_any_lower_letters
    #[test]
    fn test_contains_any_lower_letters_ok() {
        let part = "L0ReM";

        let f = contains_any_lower_letters();
        let result = f(part);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_any_lower_letters_err() {
        let part = "LOREM IPSUM";

        let f = contains_any_lower_letters();
        let result = f(part);
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_any_lower_letters_err_message() {
        let part = "LOREM";

        let f = contains_any_lower_letters();
        let result = f(part);
        assert_eq!(
            result.map_err(|e| e.to_string()),
            Err("Must contain any of a-z".to_string())
        );
    }

    // contains_any_upper_letters
    #[test]
    fn test_contains_any_upper_letters_ok() {
        let part = "Lorem";

        let f = contains_any_upper_letters();
        let result = f(part);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_any_upper_letters_err() {
        let part = "lorem ipsum";

        let f = contains_any_upper_letters();
        let result = f(part);
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_any_upper_letters_err_message() {
        let part = "lorem ipsum dolor sit amet";

        let f = contains_any_upper_letters();
        let result = f(part);
        assert_eq!(
            result.map_err(|e| e.to_string()),
            Err("Must contain any of A-Z".to_string())
        );
    }

    // not_contain_if_given
    #[test]
    fn test_not_contain_if_given_ok() {
        let part = "lorem ipsum";

        let f = not_contain_if_given(None);
        let result = f(part);
        assert!(result.is_ok());

        let f = not_contain_if_given(Some(""));
        let result = f(part);
        assert!(result.is_ok());

        let f = not_contain_if_given(Some("dolor sit amet"));
        let result = f(part);
        assert!(result.is_ok());
    }

    #[test]
    fn test_not_contain_if_given_err() {
        let part = "lorem ipsum";

        let f = not_contain_if_given(Some("ipsum"));
        let result = f(part);
        assert!(result.is_err());
    }

    #[test]
    fn test_not_contain_if_given_message() {
        let f = not_contain_if_given(Some("dolor"));
        let result = f("dolor sit amet");
        assert_eq!(
            result.map_err(|e| e.to_string()),
            Err("Must not contain dolor".to_string())
        );
    }

    // contains_if_present
    #[test]
    fn test_contains_if_present_ok() {
        let f = contains_if_present("dolor sit amet");

        let result = f(Some("lorem ipsum dolor sit amet"));
        assert!(result.is_ok());

        let result = f(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_if_present_err() {
        let f = contains_if_present("dolor sit amet");

        let result = f(Some("lorem ipsum"));
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_if_present_err_message() {
        let f = contains_if_present("dolor sit amet");

        let result = f(Some("lorem ipsum"));
        assert_eq!(
            result.map_err(|e| e.to_string()),
            Err("Must contain dolor sit amet".to_string())
        );
    }
}
