use crate::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, one_of},
    combinator::recognize,
    error::{ErrorKind, ParseError},
    multi::many0,
    sequence::tuple,
    Err, IResult,
};

/// ALPHA = %x41-5A / %x61-7A ; A-Z / a-z
pub fn ALPHA<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    one(input, is_ALPHA)
}

/// BIT = "0" / "1"
pub fn BIT<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    one_of("01")(input)
}

/// CHAR = %x01-7F ; any 7-bit US-ASCII character, excluding NUL
pub fn CHAR<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    one(input, is_CHAR)
}

/// CR = %x0D ; carriage return
pub fn CR<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    char('\r')(input)
}

pub fn crlf_strict<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &str, E> {
    tag("\r\n")(input)
}

pub fn crlf_relaxed<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &str, E> {
    line_ending(input)
}

/// CRLF = CR LF ; Internet standard newline
///
/// Note: this variant will strictly expect "\r\n".
/// Use [crlf_relaxed](fn.crlf_relaxed.html) to accept "\r\n" as well as only "\n".
pub fn CRLF<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &str, E> {
    crlf_strict(input)
}

/// CTL = %x00-1F / %x7F ; controls
pub fn CTL<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    one(input, is_CTL)
}

/// DIGIT = %x30-39 ; 0-9
pub fn DIGIT<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    one(input, is_DIGIT)
}

/// DQUOTE = %x22 ; " (Double Quote)
pub fn DQUOTE<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    char('"')(input)
}

/// HEXDIG = DIGIT / "A" / "B" / "C" / "D" / "E" / "F"
pub fn HEXDIG<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    one(input, is_HEXDIG)
}

/// HTAB = %x09 ; horizontal tab
pub fn HTAB<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    char('\t')(input)
}

/// LF = %x0A ; linefeed
pub fn LF<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    char('\n')(input)
}

/// LWSP = *(WSP / CRLF WSP)
///         ; Use of this linear-white-space rule
///         ;  permits lines containing only white
///         ;  space that are no longer legal in
///         ;  mail headers and have caused
///         ;  interoperability problems in other
///         ;  contexts.
///         ; Do not use when defining mail
///         ;  headers and use with caution in
///         ;  other contexts.
pub fn LWSP<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &str, E> {
    recognize(many0(alt((recognize(WSP), recognize(tuple((CRLF, WSP)))))))(input)
}

/// OCTET = %x00-FF ; 8 bits of data
pub fn OCTET(input: &[u8]) -> IResult<&[u8], &[u8]> {
    if input.is_empty() {
        Err(Err::Error((input, nom::error::ErrorKind::Char)))
    } else {
        Ok((&input[1..], &input[0..1]))
    }
}

/// SP = %x20
pub fn SP<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    char(' ')(input)
}

/// VCHAR = %x21-7E ; visible (printing) characters
pub fn VCHAR<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    one(input, is_VCHAR)
}

/// WSP = SP / HTAB ; white space
pub fn WSP<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    alt((SP, HTAB))(input)
}

fn one<'a, E, F>(input: &'a str, predicate: F) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str>,
    F: Fn(char) -> bool,
{
    let mut chars = input.chars();

    match chars.next() {
        Some(first) => {
            if predicate(first) {
                Ok((chars.as_str(), first))
            } else {
                Err(Err::Error(ParseError::from_error_kind(
                    input,
                    ErrorKind::IsNot,
                )))
            }
        }
        None => Err(Err::Error(ParseError::from_error_kind(
            input,
            ErrorKind::Eof,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::VerboseError;

    #[test]
    fn test_ALPHA() {
        assert!(ALPHA::<VerboseError<&str>>("").is_err());

        assert!(ALPHA::<VerboseError<&str>>("`").is_err());
        assert_eq!(ALPHA::<VerboseError<&str>>("a"), Ok(("", 'a')));
        assert_eq!(ALPHA::<VerboseError<&str>>("z"), Ok(("", 'z')));
        assert!(ALPHA::<VerboseError<&str>>("{").is_err());

        assert!(ALPHA::<VerboseError<&str>>("@").is_err());
        assert_eq!(ALPHA::<VerboseError<&str>>("A"), Ok(("", 'A')));
        assert_eq!(ALPHA::<VerboseError<&str>>("Z"), Ok(("", 'Z')));
        assert!(ALPHA::<VerboseError<&str>>("[").is_err());
    }

    #[test]
    fn test_BIT() {
        assert!(BIT::<VerboseError<&str>>("").is_err());

        assert!(BIT::<VerboseError<&str>>("/").is_err());
        assert_eq!(BIT::<VerboseError<&str>>("0"), Ok(("", '0')));
        assert_eq!(BIT::<VerboseError<&str>>("1"), Ok(("", '1')));
        assert!(BIT::<VerboseError<&str>>("2").is_err());
    }

    #[test]
    fn test_CHAR() {
        assert!(CHAR::<VerboseError<&str>>("").is_err());

        assert!(CHAR::<VerboseError<&str>>("\x00").is_err());
        assert_eq!(CHAR::<VerboseError<&str>>("\x01"), Ok(("", '\x01')));
        assert_eq!(CHAR::<VerboseError<&str>>("\x7f"), Ok(("", '\x7f')));
        assert!(CHAR::<VerboseError<&str>>("\u{80}").is_err());
    }

    #[test]
    fn test_CR() {
        assert!(CR::<VerboseError<&str>>("").is_err());

        assert!(CR::<VerboseError<&str>>("\x0c").is_err());
        assert_eq!(CR::<VerboseError<&str>>("\r"), Ok(("", '\r')));
        assert!(CR::<VerboseError<&str>>("\x0e").is_err());
    }

    #[test]
    fn test_crlf_strict() {
        assert!(crlf_strict::<VerboseError<&str>>("").is_err());

        assert!(crlf_strict::<VerboseError<&str>>("\x0c").is_err());
        assert!(crlf_strict::<VerboseError<&str>>("\r").is_err());
        assert!(crlf_strict::<VerboseError<&str>>("\x0e").is_err());

        assert!(crlf_strict::<VerboseError<&str>>("\x09").is_err());
        assert!(crlf_strict::<VerboseError<&str>>("\n").is_err());
        assert!(crlf_strict::<VerboseError<&str>>("\x0b").is_err());

        assert_eq!(crlf_strict::<VerboseError<&str>>("\r\n"), Ok(("", "\r\n")));
    }

    #[test]
    fn test_crlf_relaxed() {
        assert!(crlf_relaxed::<VerboseError<&str>>("").is_err());

        assert!(crlf_relaxed::<VerboseError<&str>>("\x0c").is_err());
        assert!(crlf_relaxed::<VerboseError<&str>>("\r").is_err());
        assert!(crlf_relaxed::<VerboseError<&str>>("\x0e").is_err());

        assert!(crlf_relaxed::<VerboseError<&str>>("\x09").is_err());
        assert_eq!(crlf_relaxed::<VerboseError<&str>>("\n"), Ok(("", "\n")));
        assert!(crlf_relaxed::<VerboseError<&str>>("\x0b").is_err());

        assert_eq!(crlf_relaxed::<VerboseError<&str>>("\r\n"), Ok(("", "\r\n")));
    }

    #[test]
    fn test_CTL() {
        assert!(CTL::<VerboseError<&str>>("").is_err());

        assert!(CTL::<VerboseError<&str>>("\x00").is_ok());
        assert!(CTL::<VerboseError<&str>>("\x1f").is_ok());
        assert!(CTL::<VerboseError<&str>>("\x20").is_err());
        assert!(CTL::<VerboseError<&str>>("\x7f").is_ok());
        assert!(CTL::<VerboseError<&str>>("\u{80}").is_err());
    }

    #[test]
    fn test_DIGIT() {
        assert!(DIGIT::<VerboseError<&str>>("").is_err());

        assert!(DIGIT::<VerboseError<&str>>("/").is_err());
        assert_eq!(DIGIT::<VerboseError<&str>>("0"), Ok(("", '0')));
        assert_eq!(DIGIT::<VerboseError<&str>>("9"), Ok(("", '9')));
        assert!(DIGIT::<VerboseError<&str>>(":").is_err());
    }

    // DQUOTE

    #[test]
    fn test_HEXDIG() {
        assert!(HEXDIG::<VerboseError<&str>>("").is_err());

        assert!(HEXDIG::<VerboseError<&str>>("/").is_err());
        assert_eq!(HEXDIG::<VerboseError<&str>>("0"), Ok(("", '0')));
        assert_eq!(HEXDIG::<VerboseError<&str>>("9"), Ok(("", '9')));
        assert!(HEXDIG::<VerboseError<&str>>(":").is_err());

        assert!(HEXDIG::<VerboseError<&str>>("`").is_err());
        assert_eq!(HEXDIG::<VerboseError<&str>>("a"), Ok(("", 'a')));
        assert_eq!(HEXDIG::<VerboseError<&str>>("f"), Ok(("", 'f')));
        assert!(HEXDIG::<VerboseError<&str>>("g").is_err());

        assert!(HEXDIG::<VerboseError<&str>>("@").is_err());
        assert_eq!(HEXDIG::<VerboseError<&str>>("A"), Ok(("", 'A')));
        assert_eq!(HEXDIG::<VerboseError<&str>>("F"), Ok(("", 'F')));
        assert!(HEXDIG::<VerboseError<&str>>("G").is_err());
    }

    // HTAB

    // LF

    // LWSP

    // OCTET

    // SP

    // VCHAR

    // WSP
}
