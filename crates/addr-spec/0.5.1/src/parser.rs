use std::{fmt, iter::Peekable, str::CharIndices};

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

    /// Returns the index where the error occurred.
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
    iterator: Peekable<CharIndices<'a>>,
}

impl<'a> Parser<'a> {
    #[inline]
    pub fn new(input: &'a str) -> Parser<'a> {
        Parser {
            input,
            iterator: input.char_indices().peekable(),
        }
    }

    #[inline]
    pub fn parse(&mut self) -> Result<AddrSpec, ParseError> {
        #[cfg(feature = "white-spaces")]
        self.skip_cfws()?;
        let local_part = self.parse_local_part()?;
        #[cfg(feature = "white-spaces")]
        self.skip_cfws()?;
        self.skip_at()?;
        #[cfg(feature = "white-spaces")]
        self.skip_cfws()?;
        let (domain, literal) = self.parse_domain()?;
        #[cfg(feature = "white-spaces")]
        self.skip_cfws()?;
        self.check_end()?;
        Ok(AddrSpec {
            local_part,
            domain: (domain, literal),
        })
    }

    #[cfg(feature = "white-spaces")]
    #[inline]
    fn skip_cfws(&mut self) -> Result<(), ParseError> {
        self.skip_fws()?;
        #[cfg(feature = "comments")]
        while self.eat('(') {
            self.skip_comment()?;
            self.skip_fws()?;
        }
        Ok(())
    }

    #[cfg(feature = "white-spaces")]
    #[inline]
    fn skip_fws(&mut self) -> Result<(), ParseError> {
        self.skip_ws();
        if !self.eat('\r') {
            return Ok(());
        }
        if !self.eat('\n') {
            return Err(ParseError("expected newline", self.char_offset()));
        }
        self.skip_ws();
        Ok(())
    }

    #[cfg(feature = "white-spaces")]
    #[inline]
    fn skip_ws(&mut self) {
        while let Some((_, chr)) = self.iterator.peek() {
            if !matches!(chr, ' ' | '\t') {
                break;
            }
            self.iterator.next();
        }
    }

    #[inline]
    fn eat(&mut self, chr: char) -> bool {
        match self.iterator.peek() {
            Some(&(_, next_chr)) if next_chr == chr => {
                self.iterator.next();
                true
            }
            _ => false,
        }
    }

    #[cfg(feature = "comments")]
    #[inline]
    fn skip_comment(&mut self) -> Result<(), ParseError> {
        #[cfg(feature = "white-spaces")]
        self.skip_fws()?;

        let mut nest_level = 1usize;
        while let Some((index, chr)) = self.iterator.next() {
            match chr {
                '\\' => {
                    self.scan_quoted_pair(
                        "invalid character in comment",
                        "expected closing parenthesis",
                    )?;
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
                    return Err(ParseError("invalid character in comment", index))
                }
                _ => (),
            }

            #[cfg(feature = "white-spaces")]
            self.skip_fws()?;
        }

        Err(ParseError("expected ')' for comment", self.char_offset()))
    }

    fn scan_quoted_pair(
        &mut self,
        invalid_character_error_text: &'static str,
        expected_character_error_text: &'static str,
    ) -> Result<char, ParseError> {
        match self.iterator.next() {
            Some((_, chr)) if !is_ascii_control_and_not_htab(chr) => Ok(chr),
            Some((index, _)) => Err(ParseError(invalid_character_error_text, index)),
            None => Err(ParseError(
                expected_character_error_text,
                self.char_offset(),
            )),
        }
    }

    #[inline]
    fn parse_local_part(&mut self) -> Result<String, ParseError> {
        if self.iterator.peek().is_none() {
            return Err(ParseError("missing local part", self.char_offset()));
        }
        if !self.eat('"') {
            return Ok(self
                .parse_dot_atom("empty local part", "empty label in local part")?
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
    fn parse_dot_atom(
        &mut self,
        empty_error_text: &'static str,
        empty_label_error_text: &'static str,
    ) -> Result<&str, ParseError> {
        let first = self.byte_offset();
        let last = loop {
            let (index, chr) = match self.iterator.peek() {
                Some(&(index, chr)) => (index, chr),
                None => break self.input.len(),
            };
            if is_not_atext(chr) {
                break index;
            }
            self.iterator.next();
        };

        let dot_atom = &self.input[first..last];
        if dot_atom.is_empty() {
            return Err(ParseError(
                empty_error_text,
                self.char_offset() - dot_atom.chars().count(),
            ));
        }
        if dot_atom.starts_with('.') {
            return Err(ParseError(
                empty_label_error_text,
                self.char_offset() - dot_atom.chars().count(),
            ));
        }
        if let Some(index) = dot_atom.find("..") {
            return Err(ParseError(
                empty_label_error_text,
                self.char_offset() - dot_atom[index..].chars().count(),
            ));
        }
        if dot_atom.ends_with('.') {
            return Err(ParseError(empty_label_error_text, self.char_offset()));
        }

        Ok(dot_atom)
    }

    #[inline]
    fn parse_quoted_string(
        &mut self,
        invalid_character_error_text: &'static str,
        expected_quote_error_text: &'static str,
    ) -> Result<String, ParseError> {
        #[cfg(feature = "white-spaces")]
        self.skip_fws()?;

        let mut quoted_string =
            unsafe { UnsafeVec::with_capacity(self.input.len() - self.byte_offset()) };
        while let Some((index, chr)) = self.iterator.next() {
            let chr = match chr {
                '"' => return Ok(quoted_string.as_mut().into()),
                '\\' => {
                    self.scan_quoted_pair(invalid_character_error_text, expected_quote_error_text)?
                }
                chr if is_ascii_control_or_space(chr) => {
                    return Err(ParseError(invalid_character_error_text, index))
                }
                chr => chr,
            };
            quoted_string.extend_char(chr);

            #[cfg(feature = "white-spaces")]
            self.skip_fws()?;
        }

        Err(ParseError(expected_quote_error_text, self.char_offset()))
    }

    #[inline]
    fn skip_at(&mut self) -> Result<(), ParseError> {
        if self.eat('@') {
            return Ok(());
        }
        Err(ParseError("expected '@'", self.char_offset()))
    }

    #[inline]
    fn parse_domain(&mut self) -> Result<(String, bool), ParseError> {
        if self.iterator.peek().is_none() {
            return Err(ParseError("missing domain", self.char_offset()));
        }
        #[cfg(feature = "literals")]
        if self.eat('[') {
            return Ok((self.parse_domain_literal()?.nfc().collect(), true));
        }
        Ok((
            self.parse_dot_atom("empty domain", "empty label in domain")?
                .nfc()
                .collect(),
            false,
        ))
    }

    #[cfg(all(feature = "literals", not(feature = "white-spaces")))]
    #[inline]
    fn parse_domain_literal(&mut self) -> Result<&str, ParseError> {
        let first = self.byte_offset();
        let last = self
            .iterator
            .find(|&(_, chr)| is_not_dtext(chr))
            .map(|(index, _)| index)
            .unwrap_or(self.input.len());

        if !self.input[last..].starts_with(']') {
            return Err(ParseError(
                "expected ']' for domain literal",
                self.char_offset(),
            ));
        }

        Ok(&self.input[first..last])
    }

    #[cfg(all(feature = "literals", feature = "white-spaces"))]
    #[inline]
    fn parse_domain_literal(&mut self) -> Result<String, ParseError> {
        #[cfg(feature = "white-spaces")]
        self.skip_fws()?;

        let mut domain = unsafe { UnsafeVec::with_capacity(self.input.len() - self.byte_offset()) };
        while let Some((index, chr)) = self.iterator.next() {
            let chr = match chr {
                ']' => return Ok(domain.as_mut().into()),
                chr if is_not_dtext(chr) => {
                    return Err(ParseError("invalid character in literal domain", index))
                }
                chr => chr,
            };
            domain.extend_char(chr);

            #[cfg(feature = "white-spaces")]
            self.skip_fws()?;
        }

        Err(ParseError(
            "expected ']' for domain literal",
            self.char_offset(),
        ))
    }

    #[inline]
    fn check_end(&mut self) -> Result<(), ParseError> {
        if self.iterator.peek().is_none() {
            return Ok(());
        }
        Err(ParseError("expected end of address", self.char_offset()))
    }

    // Note this method is only called on error, so even though counting UTF-8
    // is expensive, we let it slide.
    #[inline(always)]
    fn char_offset(&self) -> usize {
        self.input.chars().count() - self.iterator.clone().count()
    }

    #[inline]
    fn byte_offset(&mut self) -> usize {
        self.iterator
            .peek()
            .map(|&(index, _)| index)
            .unwrap_or(self.input.len())
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
