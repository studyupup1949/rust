#![allow(non_snake_case)]

//!
//! Parsing of ABNF Core Rules
//!
//! See https://tools.ietf.org/html/rfc5234#appendix-B.1
//!

pub mod complete;
pub mod streaming;

pub fn is_ALPHA(c: char) -> bool {
    c.is_ascii_alphabetic()
}

pub fn is_BIT(c: char) -> bool {
    c == '0' || c == '1'
}

pub fn is_CHAR(c: char) -> bool {
    matches!(c, '\x01'..='\x7F')
}

pub fn is_CR(c: char) -> bool {
    c == '\r'
}

// CRLF

pub fn is_CTL(c: char) -> bool {
    matches!(c, '\x00'..='\x1F' | '\x7F')
}

pub fn is_DIGIT(c: char) -> bool {
    c.is_ascii_digit()
}

pub fn is_DQUOTE(c: char) -> bool {
    c == '"'
}

pub fn is_HEXDIG(c: char) -> bool {
    c.is_ascii_hexdigit()
}

// HTAB

// LF

// LWSP

// OCTET

// SP

// VCHAR

// WSP

pub fn is_VCHAR(c: char) -> bool {
    matches!(c, '\x21'..='\x7E')
}

pub fn is_WSP(c: char) -> bool {
    matches!(c, '\x20' | '\x09')
}
