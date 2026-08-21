//! ABNF Core Rules (RFC5234 B.1.)

use std::ops::{RangeFrom, RangeTo};

use nom::{
    character::complete::satisfy,
    combinator::{opt, recognize},
    error::{ErrorKind, ParseError},
    multi::many0_count,
    sequence::{pair, terminated},
    AsChar, Err as OutCome, IResult, InputIter, InputLength, Offset, Slice,
};

use crate::{
    is_alpha, is_bit, is_char, is_cr, is_ctl, is_digit, is_dquote, is_hexdig, is_htab, is_lf,
    is_sp, is_wsp,
};

/// ALPHA = %x41-5A / %x61-7A ; A-Z / a-z
pub fn alpha<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_alpha)(input)
}

/// BIT = "0" / "1"
pub fn bit<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_bit)(input)
}

/// CHAR = %x01-7F ; any 7-bit US-ASCII character, excluding NUL
pub fn char<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_char)(input)
}

/// Carriage return
///
/// CR = %x0D
pub fn cr<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_cr)(input)
}

/// Internet standard newline
///
/// CRLF = CR LF
///
/// Note: this variant will strictly expect "\r\n".
/// Use [crlf_relaxed](fn.crlf_relaxed.html) to accept "\r\n" as well as only "\n".
pub fn crlf<I, E>(input: I) -> IResult<I, (char, char), E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    pair(satisfy(is_cr), satisfy(is_lf))(input)
}

/// Newline, with and without "\r".
pub fn crlf_relaxed<I, E>(input: I) -> IResult<I, (Option<char>, char), E>
where
    I: InputIter + Slice<RangeFrom<usize>> + Clone,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    pair(opt(satisfy(is_cr)), satisfy(is_lf))(input)
}

/// CTL = %x00-1F / %x7F ; controls
pub fn ctl<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_ctl)(input)
}

/// DIGIT = %x30-39 ; 0-9
pub fn digit<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_digit)(input)
}

/// Double Quote
///
/// DQUOTE = %x22
pub fn dquote<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_dquote)(input)
}

/// HEXDIG = DIGIT / "A" / "B" / "C" / "D" / "E" / "F"
pub fn hexdig<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_hexdig)(input)
}

/// Horizontal tab
///
/// HTAB = %x09
pub fn htab<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_htab)(input)
}

/// Linefeed
///
/// LF = %x0A
pub fn lf<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_lf)(input)
}

/// Use of this linear-white-space rule permits lines containing only white
/// space that are no longer legal in mail headers and have caused
/// interoperability problems in other contexts.
///
/// Do not use when defining mail headers and use with caution in other contexts.
///
/// LWSP = *(WSP / CRLF WSP)
pub fn lwsp<I, E>(input: I) -> IResult<I, I, E>
where
    I: Clone
        + Offset
        + PartialEq
        + InputIter
        + Slice<RangeTo<usize>>
        + Slice<RangeFrom<usize>>
        + InputLength,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    // code as equivalent avoid branching LWSP = *([CRLF] WSP)
    recognize(many0_count(terminated(opt(crlf), wsp)))(input)
}

/// OCTET = %x00-FF ; 8 bits of data
pub fn octet<E>(input: &[u8]) -> IResult<&[u8], u8, E>
where
    for<'a> E: ParseError<&'a [u8]>,
{
    match input.split_first() {
        None => Err(OutCome::Error(E::from_error_kind(
            input,
            ErrorKind::Complete,
        ))),
        Some((&b, tail)) => Ok((tail, b)),
    }
}

/// SP = %x20
pub fn sp<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_sp)(input)
}

/// VCHAR = %x21-7E ; visible (printing) characters
pub fn vchar<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_char)(input)
}

/// WSP = SP / HTAB ; white space
pub fn wsp<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputIter + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_wsp)(input)
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use super::*;

    #[test]
    fn test_alpha() {
        assert!(alpha::<_, VerboseError<&str>>("").is_err());

        assert!(alpha::<_, VerboseError<&str>>("`").is_err());
        assert_eq!(alpha::<_, VerboseError<&str>>("a"), Ok(("", 'a')));
        assert_eq!(alpha::<_, VerboseError<&str>>("z"), Ok(("", 'z')));
        assert!(alpha::<_, VerboseError<&str>>("{").is_err());

        assert!(alpha::<_, VerboseError<&str>>("@").is_err());
        assert_eq!(alpha::<_, VerboseError<&str>>("A"), Ok(("", 'A')));
        assert_eq!(alpha::<_, VerboseError<&str>>("Z"), Ok(("", 'Z')));
        assert!(alpha::<_, VerboseError<&str>>("[").is_err());
    }

    #[test]
    fn test_bit() {
        assert!(bit::<_, VerboseError<&str>>("").is_err());

        assert!(bit::<_, VerboseError<&str>>("/").is_err());
        assert_eq!(bit::<_, VerboseError<&str>>("0"), Ok(("", '0')));
        assert_eq!(bit::<_, VerboseError<&str>>("1"), Ok(("", '1')));
        assert!(bit::<_, VerboseError<&str>>("2").is_err());
    }

    #[test]
    fn test_char() {
        assert!(char::<_, VerboseError<&str>>("").is_err());

        assert!(char::<_, VerboseError<&str>>("\x00").is_err());
        assert_eq!(char::<_, VerboseError<&str>>("\x01"), Ok(("", '\x01')));
        assert_eq!(char::<_, VerboseError<&str>>("\x7f"), Ok(("", '\x7f')));
        assert!(char::<_, VerboseError<&str>>("\u{80}").is_err());
    }

    #[test]
    fn test_cr() {
        assert!(cr::<_, VerboseError<&str>>("").is_err());

        assert!(cr::<_, VerboseError<&str>>("\x0c").is_err());
        assert_eq!(cr::<_, VerboseError<&str>>("\r"), Ok(("", '\r')));
        assert!(cr::<_, VerboseError<&str>>("\x0e").is_err());
    }

    #[test]
    fn test_crlf() {
        assert!(crlf::<_, VerboseError<&str>>("").is_err());

        assert!(crlf::<_, VerboseError<&str>>("\x0c").is_err());
        assert!(crlf::<_, VerboseError<&str>>("\r").is_err());
        assert!(crlf::<_, VerboseError<&str>>("\x0e").is_err());

        assert!(crlf::<_, VerboseError<&str>>("\x09").is_err());
        assert!(crlf::<_, VerboseError<&str>>("\n").is_err());
        assert!(crlf::<_, VerboseError<&str>>("\x0b").is_err());

        assert_eq!(
            crlf::<_, VerboseError<&str>>("\r\n"),
            Ok(("", ('\r', '\n')))
        );
    }

    #[test]
    fn test_crlf_relaxed() {
        assert!(crlf_relaxed::<_, VerboseError<&str>>("").is_err());

        assert!(crlf_relaxed::<_, VerboseError<&str>>("\x0c").is_err());
        assert!(crlf_relaxed::<_, VerboseError<&str>>("\r").is_err());
        assert!(crlf_relaxed::<_, VerboseError<&str>>("\x0e").is_err());

        assert!(crlf_relaxed::<_, VerboseError<&str>>("\x09").is_err());
        assert_eq!(
            crlf_relaxed::<_, VerboseError<&str>>("\n"),
            Ok(("", (None, '\n')))
        );
        assert!(crlf_relaxed::<_, VerboseError<&str>>("\x0b").is_err());

        assert_eq!(
            crlf_relaxed::<_, VerboseError<&str>>("\r\n"),
            Ok(("", ((Some('\r'), '\n'))))
        );
    }

    #[test]
    fn test_ctl() {
        assert!(ctl::<_, VerboseError<&str>>("").is_err());

        assert!(ctl::<_, VerboseError<&str>>("\x00").is_ok());
        assert!(ctl::<_, VerboseError<&str>>("\x1f").is_ok());
        assert!(ctl::<_, VerboseError<&str>>("\x20").is_err());
        assert!(ctl::<_, VerboseError<&str>>("\x7f").is_ok());
        assert!(ctl::<_, VerboseError<&str>>("\u{80}").is_err());
    }

    #[test]
    fn test_digit() {
        assert!(digit::<_, VerboseError<&str>>("").is_err());

        assert!(digit::<_, VerboseError<&str>>("/").is_err());
        assert_eq!(digit::<_, VerboseError<&str>>("0"), Ok(("", '0')));
        assert_eq!(digit::<_, VerboseError<&str>>("9"), Ok(("", '9')));
        assert!(digit::<_, VerboseError<&str>>(":").is_err());
    }

    // DQUOTE

    #[test]
    fn test_hexdig() {
        assert!(hexdig::<_, VerboseError<&str>>("").is_err());

        assert!(hexdig::<_, VerboseError<&str>>("/").is_err());
        assert_eq!(hexdig::<_, VerboseError<&str>>("0"), Ok(("", '0')));
        assert_eq!(hexdig::<_, VerboseError<&str>>("9"), Ok(("", '9')));
        assert!(hexdig::<_, VerboseError<&str>>(":").is_err());

        assert!(hexdig::<_, VerboseError<&str>>("`").is_err());
        assert_eq!(hexdig::<_, VerboseError<&str>>("a"), Ok(("", 'a')));
        assert_eq!(hexdig::<_, VerboseError<&str>>("f"), Ok(("", 'f')));
        assert!(hexdig::<_, VerboseError<&str>>("g").is_err());

        assert!(hexdig::<_, VerboseError<&str>>("@").is_err());
        assert_eq!(hexdig::<_, VerboseError<&str>>("A"), Ok(("", 'A')));
        assert_eq!(hexdig::<_, VerboseError<&str>>("F"), Ok(("", 'F')));
        assert!(hexdig::<_, VerboseError<&str>>("G").is_err());
    }

    // HTAB

    // LF

    // LWSP

    // OCTET

    // SP

    // VCHAR

    // WSP
}
