use std::{fmt, str::Chars};

use unicode_normalization::UnicodeNormalization;

use super::AddrSpec;

#[inline]
pub const fn is_ascii_control_and_not_htab(chr: char) -> bool {
    chr.is_ascii_control() && chr != '\t'
}

#[inline]
pub const fn is_ascii_control_or_space(chr: char) -> bool {
    chr.is_ascii_control() || chr == ' '
}

#[inline]
pub const fn is_not_atext(chr: char) -> bool {
    chr.is_ascii_control()
        || matches!(
            chr,
            ' ' | '"' | '(' | ')' | ',' | ':' | '<' | '>' | '@' | '[' | ']' | '\\'
        )
}

#[inline]
pub const fn is_not_dtext(chr: char) -> bool {
    chr.is_ascii_control() || matches!(chr, ' ' | '[' | ']' | '\\')
}

/// A error that can occur when parsing or creating an address specification.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ParseError(pub(super) &'static str, pub(super) usize);

impl ParseError {
    /// Returns a static error message.
    #[inline]
    pub fn message(&self) -> &'static str {
        self.0
    }

    /// Returns the byte index where the error occurred.
    #[inline]
    pub fn index(&self) -> usize {
        self.1
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "parse error at index {}: {}",
            self.message(),
            self.index()
        )
    }
}

pub struct Parser<'a> {
    input: &'a str,
    iterator: Chars<'a>,
}

impl<'a> Parser<'a> {
    #[inline]
    pub fn new(input: &'a str) -> Parser<'a> {
        Parser {
            input,
            iterator: input.chars(),
        }
    }

    #[inline]
    pub fn parse(mut self) -> Result<AddrSpec, ParseError> {
        #[cfg(feature = "white-spaces")]
        self.parse_cfws()?;
        let local_part = self.parse_local_part()?;
        #[cfg(feature = "white-spaces")]
        self.parse_cfws()?;
        self.skip_at()?;
        #[cfg(feature = "white-spaces")]
        self.parse_cfws()?;
        let (domain, literal) = self.parse_domain()?;
        #[cfg(feature = "white-spaces")]
        self.parse_cfws()?;
        self.check_end("expected end of address")?;
        Ok(AddrSpec {
            local_part,
            domain: (domain, literal),
        })
    }

    #[cfg(feature = "white-spaces")]
    #[inline]
    fn parse_cfws(&mut self) -> Result<(), ParseError> {
        self.skip_fws();
        #[cfg(feature = "comments")]
        while self.eat_chr('(') {
            self.parse_comment()?;
            self.skip_fws();
        }
        Ok(())
    }

    #[cfg(feature = "white-spaces")]
    #[inline]
    fn skip_fws(&mut self) {
        self.skip_ws();
        if !self.eat_str("\r\n") {
            return;
        }
        self.skip_ws();
    }

    #[cfg(feature = "white-spaces")]
    #[inline]
    fn skip_ws(&mut self) {
        loop {
            if !self.eat_slice([' ', '\t']) {
                break;
            }
        }
    }

    #[cfg(feature = "white-spaces")]
    #[inline]
    fn eat_slice<const N: usize>(&mut self, pattern: [char; N]) -> bool {
        if self.iterator.as_str().starts_with(pattern) {
            self.iterator.next();
            return true;
        }
        false
    }

    #[cfg(feature = "white-spaces")]
    #[inline]
    fn eat_str(&mut self, pattern: &str) -> bool {
        if let Some(input) = self.iterator.as_str().strip_prefix(pattern) {
            self.iterator = input.chars();
            return true;
        }
        false
    }

    #[inline]
    fn eat_chr(&mut self, pattern: char) -> bool {
        if self.iterator.as_str().starts_with(pattern) {
            self.iterator.next();
            return true;
        }
        false
    }

    #[cfg(feature = "comments")]
    #[inline]
    fn parse_comment(&mut self) -> Result<(), ParseError> {
        #[cfg(feature = "white-spaces")]
        self.skip_fws();

        let mut nest_level = 1usize;
        while let Some(chr) = self.iterator.next() {
            match chr {
                '\\' => {
                    self.parse_quoted_pair()?;
                }
                '(' => {
                    nest_level += 1;
                }
                ')' if nest_level == 1 => {
                    return Ok(());
                }
                ')' => {
                    nest_level -= 1;
                }
                chr if is_ascii_control_or_space(chr) => {
                    return Err(self.error("invalid character in comment", -1))
                }
                _ => (),
            }

            #[cfg(feature = "white-spaces")]
            self.skip_fws();
        }

        Err(self.error("expected ')' for comment", 0))
    }

    #[inline]
    fn parse_quoted_pair(&mut self) -> Result<char, ParseError> {
        match self.iterator.next() {
            Some(chr) if !is_ascii_control_and_not_htab(chr) => Ok(chr),
            Some(_) => Err(self.error("invalid character in quoted pair", -1)),
            None => Err(self.error("unexpected end of quoted pair", 0)),
        }
    }

    #[inline]
    fn parse_local_part(&mut self) -> Result<String, ParseError> {
        if !self.eat_chr('"') {
            return Ok(self
                .parse_dot_atom(
                    "unquoted local part cannot be empty",
                    "empty label in local part",
                )?
                .nfc()
                .collect());
        }
        Ok(self
            .parse_quoted_string(
                "invalid character in quoted local part",
                "expected '\"' for quoted local part",
            )?
            .nfc()
            .collect())
    }

    #[inline]
    pub fn parse_dot_atom(
        &mut self,
        empty_error_text: &'static str,
        empty_label_error_text: &'static str,
    ) -> Result<&str, ParseError> {
        let input = self.iterator.as_str();
        let size = input.find(is_not_atext).unwrap_or(input.len());

        let dot_atom = &input[..size];
        if dot_atom.is_empty() {
            return Err(self.error(empty_error_text, 0));
        }
        if dot_atom.starts_with('.') {
            return Err(self.error(empty_label_error_text, 0));
        }
        if let Some(index) = dot_atom.find("..") {
            return Err(self.error(empty_label_error_text, index as isize));
        }
        if dot_atom.ends_with('.') {
            return Err(self.error(empty_label_error_text, (size - 1) as isize));
        }

        self.iterator = input[size..].chars();
        Ok(dot_atom)
    }

    #[inline]
    fn parse_quoted_string(
        &mut self,
        invalid_character_error_text: &'static str,
        expected_quote_error_text: &'static str,
    ) -> Result<String, ParseError> {
        #[cfg(feature = "white-spaces")]
        self.skip_fws();

        let mut quoted_string = unsafe { UnsafeVec::with_capacity(self.iterator.as_str().len()) };
        while let Some(chr) = self.iterator.next() {
            let chr = match chr {
                '"' => return Ok(quoted_string.as_mut().into()),
                '\\' => self.parse_quoted_pair()?,
                chr if is_ascii_control_or_space(chr) => {
                    return Err(self.error(invalid_character_error_text, -1))
                }
                chr => chr,
            };
            quoted_string.extend_char(chr);

            #[cfg(feature = "white-spaces")]
            self.skip_fws();
        }

        Err(self.error(expected_quote_error_text, 0))
    }

    #[inline]
    fn skip_at(&mut self) -> Result<(), ParseError> {
        if self.eat_chr('@') {
            return Ok(());
        }
        Err(self.error("expected '@'", 1))
    }

    #[inline]
    fn parse_domain(&mut self) -> Result<(String, bool), ParseError> {
        #[cfg(feature = "literals")]
        if self.eat_chr('[') {
            return Ok((self.parse_domain_literal()?.nfc().collect(), true));
        }
        Ok((
            self.parse_dot_atom(
                "non-literal domain cannot be empty",
                "empty label in domain",
            )?
            .nfc()
            .collect(),
            false,
        ))
    }

    #[cfg(all(feature = "literals", not(feature = "white-spaces")))]
    #[inline]
    fn parse_domain_literal(&mut self) -> Result<&str, ParseError> {
        let input = self.iterator.as_str();
        let size = input.find(is_not_dtext).unwrap_or(input.len());

        self.iterator = input[size..].chars();
        if !self.eat_chr(']') {
            return Err(self.error("expected ']' for domain literal", 0));
        }

        Ok(&input[..size])
    }

    #[cfg(all(feature = "literals", feature = "white-spaces"))]
    #[inline]
    fn parse_domain_literal(&mut self) -> Result<String, ParseError> {
        #[cfg(feature = "white-spaces")]
        self.skip_fws();

        let mut domain = unsafe { UnsafeVec::with_capacity(self.iterator.as_str().len()) };
        while let Some(chr) = self.iterator.next() {
            let chr = match chr {
                ']' => return Ok(domain.as_mut().into()),
                chr if is_not_dtext(chr) => {
                    return Err(self.error("invalid character in literal domain", -1))
                }
                chr => chr,
            };
            domain.extend_char(chr);

            #[cfg(feature = "white-spaces")]
            self.skip_fws();
        }

        Err(self.error("expected ']' for domain literal", 0))
    }

    #[inline]
    pub fn check_end(self, message: &'static str) -> Result<(), ParseError> {
        if self.iterator.as_str().is_empty() {
            return Ok(());
        }
        Err(self.error(message, 0))
    }

    #[inline(always)]
    fn error(&self, message: &'static str, offset: isize) -> ParseError {
        ParseError(
            message,
            (self.input.len() - self.iterator.as_str().len())
                .checked_add_signed(offset)
                .unwrap(),
        )
    }
}

/// All methods of this struct are unsafe. Even those implemented in traits. Use
/// with caution.
struct UnsafeVec<T> {
    vec: Vec<T>,
    len: usize,
}

impl<T> UnsafeVec<T> {
    #[inline]
    unsafe fn with_capacity(len: usize) -> Self {
        Self {
            vec: Vec::with_capacity(len),
            len: 0,
        }
    }

    #[inline]
    fn extend(&mut self, slice: &[T]) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                slice.as_ptr(),
                self.vec.as_mut_ptr().add(self.len),
                slice.len(),
            );
        }
        self.len += slice.len();
        debug_assert!(self.len <= self.vec.capacity());
    }
}

impl UnsafeVec<u8> {
    #[inline]
    fn extend_char(&mut self, chr: char) {
        self.extend(chr.encode_utf8(&mut [0; 4]).as_bytes())
    }
}

impl AsMut<str> for UnsafeVec<u8> {
    #[inline]
    fn as_mut(&mut self) -> &mut str {
        unsafe {
            self.vec.set_len(self.len);
            std::str::from_utf8_unchecked_mut(&mut self.vec)
        }
    }
}

#[cfg(test)]
mod tests {
    mod dot_atoms {
        use super::super::{ParseError, Parser};

        #[test]
        fn test_parse_local_part() {
            assert_eq!(&Parser::new("test").parse_local_part().unwrap(), "test")
        }

        #[test]
        fn test_parse_empty_local_part() {
            assert_eq!(
                Parser::new("").parse_local_part().unwrap_err(),
                ParseError("unquoted local part cannot be empty", 0)
            )
        }

        #[test]
        fn test_parse_local_part_with_empty_label_in_front() {
            assert_eq!(
                Parser::new(".test").parse_local_part().unwrap_err(),
                ParseError("empty label in local part", 0)
            )
        }

        #[test]
        fn test_parse_local_part_with_empty_label_in_middle() {
            assert_eq!(
                Parser::new("te..st").parse_local_part().unwrap_err(),
                ParseError("empty label in local part", 2)
            )
        }

        #[test]
        fn test_parse_local_part_with_empty_label_in_back() {
            assert_eq!(
                Parser::new("test.").parse_local_part().unwrap_err(),
                ParseError("empty label in local part", 4)
            )
        }

        #[test]
        fn test_parse_domain() {
            assert_eq!(
                Parser::new("test").parse_domain().unwrap(),
                ("test".to_string(), false)
            )
        }

        #[test]
        fn test_parse_empty_domain() {
            assert_eq!(
                Parser::new("").parse_domain().unwrap_err(),
                ParseError("non-literal domain cannot be empty", 0)
            )
        }

        #[test]
        fn test_parse_domain_with_empty_label_in_front() {
            assert_eq!(
                Parser::new(".test").parse_domain().unwrap_err(),
                ParseError("empty label in domain", 0)
            )
        }

        #[test]
        fn test_parse_domain_with_empty_label_in_middle() {
            assert_eq!(
                Parser::new("te..st").parse_domain().unwrap_err(),
                ParseError("empty label in domain", 2)
            )
        }

        #[test]
        fn test_parse_domain_with_empty_label_in_back() {
            assert_eq!(
                Parser::new("test.").parse_domain().unwrap_err(),
                ParseError("empty label in domain", 4)
            )
        }
    }

    #[cfg(feature = "literals")]
    mod literals {
        use super::super::{ParseError, Parser};

        #[test]
        fn test_parse_literal_domain() {
            assert_eq!(
                Parser::new("[test]").parse_domain().unwrap(),
                ("test".to_string(), true)
            )
        }

        #[test]
        fn test_parse_literal_domain_without_bracket() {
            assert_eq!(
                Parser::new("[test").parse_domain().unwrap_err(),
                ParseError("expected ']' for domain literal", 5)
            )
        }

        #[test]
        fn test_parse_empty_literal_domain() {
            assert_eq!(
                Parser::new("[]").parse_domain().unwrap(),
                ("".to_string(), true)
            )
        }

        #[test]
        fn test_parse_empty_literal_domain_without_bracket() {
            assert_eq!(
                Parser::new("[").parse_domain().unwrap_err(),
                ParseError("expected ']' for domain literal", 1)
            )
        }

        #[cfg(not(feature = "white-spaces"))]
        #[test]
        fn test_parse_literal_domain_with_white_spaces() {
            assert_eq!(
                Parser::new("[te st]").parse_domain().unwrap_err(),
                ParseError("expected ']' for domain literal", 3)
            )
        }

        #[cfg(feature = "white-spaces")]
        #[test]
        fn test_parse_literal_domain_with_white_spaces() {
            assert_eq!(
                Parser::new("[te st]").parse_domain().unwrap(),
                ("test".to_string(), true)
            )
        }

        #[cfg(feature = "white-spaces")]
        #[test]
        fn test_parse_literal_domain_with_fws_in_front() {
            assert_eq!(
                Parser::new("[\r\ntest]").parse_domain().unwrap(),
                ("test".to_string(), true)
            )
        }

        #[cfg(feature = "white-spaces")]
        #[test]
        fn test_parse_literal_domain_with_fws_in_middle() {
            assert_eq!(
                Parser::new("[te\r\nst]").parse_domain().unwrap(),
                ("test".to_string(), true)
            )
        }

        #[cfg(feature = "white-spaces")]
        #[test]
        fn test_parse_literal_domain_with_fws_in_back() {
            assert_eq!(
                Parser::new("[test\r\n]").parse_domain().unwrap(),
                ("test".to_string(), true)
            )
        }
    }
}
