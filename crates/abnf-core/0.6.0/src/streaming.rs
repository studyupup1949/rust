//! ABNF Core Rules (RFC5234 B.1.)

use std::ops::{RangeFrom, RangeTo};

use nom::{
    character::streaming::satisfy,
    combinator::{opt, recognize},
    error::ParseError,
    multi::many0_count,
    sequence::{pair, terminated},
    AsChar, Err as OutCome, IResult, InputIter, InputLength, Needed, Offset, Slice,
};

use crate::{
    is_alpha, is_bit, is_char, is_cr, is_ctl, is_digit, is_dquote, is_hexdig, is_htab, is_lf,
    is_sp, is_wsp,
};

/// ALPHA = %x41-5A / %x61-7A ; A-Z / a-z
pub fn alpha<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_alpha)(input)
}

/// BIT = "0" / "1"
pub fn bit<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_bit)(input)
}

/// CHAR = %x01-7F ; any 7-bit US-ASCII character, excluding NUL
pub fn char<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
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
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
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
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    pair(satisfy(is_cr), satisfy(is_lf))(input)
}

/// Newline, with and without "\r".
pub fn crlf_relaxed<I, E>(input: I) -> IResult<I, (Option<char>, char), E>
where
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    pair(opt(satisfy(is_cr)), satisfy(is_lf))(input)
}

/// CTL = %x00-1F / %x7F ; controls
pub fn ctl<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_ctl)(input)
}

/// DIGIT = %x30-39 ; 0-9
pub fn digit<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
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
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_dquote)(input)
}

/// HEXDIG = DIGIT / "A" / "B" / "C" / "D" / "E" / "F"
pub fn hexdig<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
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
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
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
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
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
        + InputLength
        + InputIter
        + Slice<RangeTo<usize>>
        + Slice<RangeFrom<usize>>,
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
        None => Err(OutCome::Incomplete(Needed::new(1))),
        Some((&b, tail)) => Ok((tail, b)),
    }
}

/// SP = %x20
pub fn sp<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_sp)(input)
}

/// VCHAR = %x21-7E ; visible (printing) characters
pub fn vchar<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(is_char)(input)
}

/// White space
///
/// WSP = SP / HTAB
pub fn wsp<I, E>(input: I) -> IResult<I, char, E>
where
    I: InputLength + InputIter + Slice<RangeFrom<usize>> + Clone,
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
    fn test_cr() {
        assert!(cr::<_, VerboseError<_>>("\n").is_err());
        assert_eq!(cr::<_, VerboseError<_>>("\r"), Ok(("", '\r')));

        assert!(cr::<_, VerboseError<_>>(&b"\n"[..]).is_err());
        assert_eq!(cr::<_, VerboseError<_>>(&b"\r"[..]), Ok((&b""[..], '\r')));
    }
}
