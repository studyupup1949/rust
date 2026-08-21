//!
//! Parsing of ABNF Core Rules
//!
//! See <https://tools.ietf.org/html/rfc5234#appendix-B.1>
//!

pub mod complete;
pub mod streaming;

use nom::AsChar;

/// A-Z / a-z
///
/// ALPHA = %x41-5A / %x61-7A
pub fn is_alpha(c: impl AsChar) -> bool {
    matches!(c.as_char(), '\x41'..='\x5A' | '\x61'..='\x7A')
}

/// BIT = "0" / "1"
/// BIT = %x30 / %x31
pub fn is_bit(c: impl AsChar) -> bool {
    matches!(c.as_char(), '\x30'..='\x31')
}

/// Any 7-bit US-ASCII character, excluding NUL
///
/// CHAR = %x01-7F
pub fn is_char(c: impl AsChar) -> bool {
    matches!(c.as_char(), '\x01'..='\x7F')
}

/// Carriage return
///
/// CR = %x0D
pub fn is_cr(c: impl AsChar) -> bool {
    matches!(c.as_char(), '\x0D')
}

// CRLF
// Not implemented as predicate.

/// Controls
///
/// CTL = %x00-1F / %x7F
pub fn is_ctl(c: impl AsChar) -> bool {
    matches!(c.as_char(), '\x00'..='\x1F' | '\x7F')
}

/// 0-9
///
/// DIGIT = %x30-39
pub fn is_digit(c: impl AsChar) -> bool {
    matches!(c.as_char(), '\x30'..='\x39')
}

/// Double Quote (")
///
/// DQUOTE = %x22
pub fn is_dquote(c: impl AsChar) -> bool {
    matches!(c.as_char(), '\x22')
}

/// HEXDIG = DIGIT / "A" / "B" / "C" / "D" / "E" / "F"
///
/// Note: ABNF strings are case-insensitive so `a` / ... / `f` are allowed, too.
/// Issue: <https://github.com/duesee/abnf-core/issues/12>
pub fn is_hexdig(c: impl AsChar) -> bool {
    matches!(c.as_char(), '0'..='9' | 'a'..='f' | 'A'..='F')
}

/// Horizontal tab
///
/// HTAB = %x09
pub fn is_htab(c: impl AsChar) -> bool {
    matches!(c.as_char(), '\x09')
}

/// Linefeed
///
/// LF = %x0A
pub fn is_lf(c: impl AsChar) -> bool {
    matches!(c.as_char(), '\x0A')
}

// LWSP
// Not implemented as predicate.

/// 8 bits of data
///
/// OCTET = %x00-FF
pub fn is_octet(_: u8) -> bool {
    true
}

/// Space
///
/// SP = %x20
pub fn is_sp(c: impl AsChar) -> bool {
    matches!(c.as_char(), '\x20')
}

/// Visible (printing) characters
///
/// VCHAR = %x21-7E
pub fn is_vchar(c: impl AsChar) -> bool {
    matches!(c.as_char(), '\x21'..='\x7E')
}

/// White space
///
/// WSP = SP / HTAB
pub fn is_wsp(c: impl AsChar) -> bool {
    matches!(c.as_char(), '\x20' | '\x09')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_alpha() {
        assert!(is_alpha(b'a'));
        assert!(is_alpha('A'));
        assert!(is_alpha('z'));
        assert!(is_alpha('Z'));
        assert!(!is_alpha('0'));
        assert!(!is_alpha('9'));
        assert!(!is_alpha('#'));
    }

    #[test]
    fn test_is_bit() {
        assert!(is_bit(b'0'));
        assert!(is_bit('1'));
        assert!(!is_bit('2'));
        assert!(!is_bit('a'));
        assert!(!is_bit('A'));
        assert!(!is_bit('z'));
        assert!(!is_bit('Z'));
        assert!(!is_bit('#'));
    }

    #[test]
    fn test_is_char() {
        assert!(is_char(b'a'));
        assert!(is_char('A'));
        assert!(is_char('z'));
        assert!(is_char('Z'));
        assert!(is_char('\x7F'));
        assert!(!is_char('\x00'));
    }

    #[test]
    fn test_is_cr() {
        assert!(is_cr(b'\r'));
        assert!(!is_cr('\n'));
        assert!(!is_cr('a'));
        assert!(!is_cr('A'));
        assert!(!is_cr('z'));
        assert!(!is_cr('Z'));
    }

    #[test]
    fn test_is_ctl() {
        assert!(is_ctl(b'\x00'));
        assert!(is_ctl('\x1F'));
        assert!(is_ctl('\x7F'));
        assert!(!is_ctl('a'));
        assert!(!is_ctl('A'));
        assert!(!is_ctl('z'));
        assert!(!is_ctl('Z'));
    }

    #[test]
    fn test_is_digit() {
        assert!(is_digit(b'0'));
        assert!(is_digit('9'));
        assert!(!is_digit('a'));
        assert!(!is_digit('A'));
        assert!(!is_digit('z'));
        assert!(!is_digit('Z'));
    }

    #[test]
    fn test_is_dquote() {
        assert!(is_dquote(b'"'));
        assert!(!is_dquote('\''));
        assert!(!is_dquote('a'));
        assert!(!is_dquote('A'));
        assert!(!is_dquote('z'));
        assert!(!is_dquote('Z'));
    }

    #[test]
    fn test_is_hexdig() {
        assert!(is_hexdig(b'0'));
        assert!(is_hexdig('9'));
        assert!(is_hexdig('a'));
        assert!(is_hexdig('f'));
        assert!(is_hexdig('A'));
        assert!(is_hexdig('F'));
        assert!(!is_hexdig('z'));
        assert!(!is_hexdig('Z'));
    }

    #[test]
    fn test_is_htab() {
        assert!(is_htab(b'\t'));
    }

    #[test]
    fn test_is_lf() {
        assert!(is_lf(b'\n'));
        assert!(!is_lf('\r'));
        assert!(!is_lf('a'));
        assert!(!is_lf('A'));
        assert!(!is_lf('z'));
        assert!(!is_lf('Z'));
    }

    #[test]
    fn test_is_octet() {
        assert!(is_octet(0x00));
        assert!(is_octet(0x7F));
        assert!(is_octet(0xFF));
    }

    #[test]
    fn test_is_sp() {
        assert!(is_sp(b' '));
        assert!(!is_sp('\t'));
        assert!(!is_sp('a'));
        assert!(!is_sp('A'));
        assert!(!is_sp('z'));
        assert!(!is_sp('Z'));
    }

    #[test]
    fn test_is_vchar() {
        assert!(is_vchar(b'!'));
        assert!(is_vchar('~'));
        assert!(!is_vchar('\x20'));
        assert!(!is_vchar('\x7F'));
    }

    #[test]
    fn test_is_wsp() {
        assert!(is_wsp(b' '));
        assert!(is_wsp('\t'));
        assert!(!is_wsp('a'));
        assert!(!is_wsp('A'));
        assert!(!is_wsp('z'));
        assert!(!is_wsp('Z'));
    }
}
