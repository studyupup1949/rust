use crate::{IResult, Input};
use nom::{
    AsChar, Parser,
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_till, take_till1},
    character::complete::{char, digit0, digit1, one_of, satisfy},
    combinator::{map, map_res, opt, recognize, value},
    multi::many0,
    sequence::{delimited, preceded, terminated},
};

fn special_char(b: u8) -> bool {
    "()<>[]{}/%".contains(b as char)
}

fn word_sep(b: u8) -> bool {
    " \t\r\n".contains(b as char)
}

fn name(i: Input) -> IResult<Input> {
    alt((
        tag("["),
        tag("]"),
        take_till1(|b| word_sep(b) || special_char(b)),
    ))
    .parse(i)
}

fn literal_name(i: Input) -> IResult<Input> {
    preceded(char('/'), take_till(|b| word_sep(b) || special_char(b))).parse(i)
}

fn string(i: Input) -> IResult<Vec<u8>> {
    delimited(char('('), delimited_literal, char(')')).parse(i)
}

fn integer(i: Input) -> IResult<i32> {
    map_res(recognize((opt(one_of("+-")), digit1)), |s| {
        std::str::from_utf8(s).unwrap().parse()
    })
    .parse(i)
}

fn float(i: Input) -> IResult<f32> {
    map_res(
        recognize((
            opt(one_of("+-")),
            digit0,
            char('.'),
            digit0,
            opt((one_of("eE"), opt(one_of("+-")), digit1)),
        )),
        |s| std::str::from_utf8(s).unwrap().parse::<f32>(),
    )
    .parse(i)
}

fn delimited_literal(i: Input) -> IResult<Vec<u8>> {
    let mut level = 0;
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(&b) = i.get(pos) {
        match b {
            b')' => {
                if level == 0 {
                    break;
                }
                level -= 1;
                out.push(b);
                pos += 1;
            }
            b'(' => {
                level += 1;
                out.push(b);
                pos += 1;
            }
            b'\\' => {
                if let Some(&c) = i.get(pos + 1) {
                    let r = match c {
                        b'n' => b'\n',
                        b'r' => b'\r',
                        b't' => b'\t',
                        b'b' => 8,
                        b'f' => 12,
                        b @ b'\n' | b @ b'\r' => {
                            match (b, i.get(pos + 2)) {
                                (b'\n', Some(b'\r')) | (b'\r', Some(b'\n')) => pos += 3,
                                _ => pos += 2,
                            }
                            continue;
                        }
                        c => c,
                    };
                    out.push(r);
                    pos += 2;
                } else {
                    break;
                }
            }
            _ => {
                out.push(b);
                pos += 1;
            }
        }
    }
    Ok((&i[pos..], out))
}

fn procedure(i: Input) -> IResult<Vec<Token>> {
    delimited(
        tag("{"),
        many0(preceded(spaces, token)),
        preceded(spaces, tag("}")),
    )
    .parse(i)
}

pub fn spaces(input: Input) -> IResult<()> {
    value((), opt(is_a(" \t\r\n"))).parse(input)
}

pub fn comment(input: Input) -> IResult<()> {
    value((), preceded(char('%'), (opt(is_not("\r\n")), spaces))).parse(input)
}

#[derive(PartialEq)]
pub enum Token<'a> {
    Int(i32),
    Real(f32),
    Literal(&'a [u8]),
    Name(&'a [u8]),
    String(Vec<u8>),
    Procedure(Vec<Token<'a>>),
}

impl<'a> std::fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Int(i) => i.fmt(f),
            Token::Real(r) => r.fmt(f),
            Token::Literal(s) => write!(f, "/{}", String::from_utf8_lossy(&s)),
            Token::Name(s) => write!(f, "{}", String::from_utf8_lossy(&s)),
            Token::String(data) => write!(f, "({:?})", String::from_utf8_lossy(data)),
            Token::Procedure(vec) => f.debug_set().entries(vec).finish(),
        }
    }
}

pub fn token(i: Input) -> IResult<Token> {
    terminated(
        alt((
            map(float, |f| Token::Real(f.into())),
            map(integer, |i| Token::Int(i)),
            map(literal_name, |s| Token::Literal(s)),
            map(procedure, |v| Token::Procedure(v)),
            map(string, |v| Token::String(v)),
            map(name, |s| Token::Name(s)),
        )),
        spaces,
    )
    .parse(i)
}

pub fn hex_string(input: Input) -> IResult<Vec<u8>> {
    fn hex_digit(input: Input) -> IResult<u8> {
        map(satisfy(|c| c.is_hex_digit()), |c| {
            c.to_digit(16).unwrap() as u8
        })
        .parse(input)
    }

    fn hex_char(input: Input) -> IResult<u8> {
        map(
            (
                terminated(hex_digit, spaces),
                opt(terminated(hex_digit, spaces)),
            ),
            |(a, b)| a << 4 | b.unwrap_or(0),
        )
        .parse(input)
    }

    many0(hex_char).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        assert_eq!(
            literal_name(&b"/FontBBox{-180 -293 1090 1010}readonly def"[..]),
            Ok((&b"{-180 -293 1090 1010}readonly def"[..], &b"FontBBox"[..]))
        );
        assert_eq!(
            literal_name(&b"/.notdef "[..]),
            Ok((&b" "[..], &b".notdef"[..]))
        );
    }

    #[test]
    fn test_procedure() {
        assert_eq!(
            procedure("{-180 -293 1090 1010}readonly ".as_bytes())
                .unwrap()
                .1,
            vec![
                Token::Int(-180),
                Token::Int(-293),
                Token::Int(1090),
                Token::Int(1010)
            ]
        );
        assert_eq!(
            procedure("{1 index exch /.notdef put} ".as_bytes())
                .unwrap()
                .1,
            vec![
                Token::Int(1),
                Token::Name("index".as_bytes()),
                Token::Name("exch".as_bytes()),
                Token::Literal(".notdef".as_bytes()),
                Token::Name("put".as_bytes()),
            ]
        );
    }
}
