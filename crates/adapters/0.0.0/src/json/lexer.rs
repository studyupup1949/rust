//! Native JSON lexer — tokenizes a JSON string without external dependencies.
//!
//! Implements strict parsing of literal primitives, object/array punctuation, and unicode escape sequences.

use crate::error::JsonError;

/// All lexical tokens produced by the JSON scanner.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Opening curly brace `{`
    LBrace,
    /// Closing curly brace `}`
    RBrace,
    /// Opening square bracket `[`
    LBracket,
    /// Closing square bracket `]`
    RBracket,
    /// Field delimiter colon `:`
    Colon,
    /// Item delimiter comma `,`
    Comma,
    /// Standard JSON string value.
    Str(String),
    /// Numeric JSON representation.
    ///
    /// The second element indicates if the token contains a decimal point or scientific exponent form.
    Number(f64, bool),
    /// Boolean state flag.
    Bool(bool),
    /// Primitive null reference.
    Null,
    /// Sentinel indicating end of input payload.
    Eof,
}

/// Stateful lexer scanner working over a borrowed string slice.
pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexical scanner instance from the input source.
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Consumes characters to yield the next valid syntactic [`Token`].
    ///
    /// # Errors
    ///
    /// Returns a [`JsonError`] if an illegal symbol or malformed structure is parsed.
    pub fn next_token(&mut self) -> Result<Token, JsonError> {
        self.skip_whitespace();
        match self.current_char() {
            None => Ok(Token::Eof),
            Some('{') => {
                self.advance();
                Ok(Token::LBrace)
            }
            Some('}') => {
                self.advance();
                Ok(Token::RBrace)
            }
            Some('[') => {
                self.advance();
                Ok(Token::LBracket)
            }
            Some(']') => {
                self.advance();
                Ok(Token::RBracket)
            }
            Some(':') => {
                self.advance();
                Ok(Token::Colon)
            }
            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }
            Some('"') => {
                self.advance();
                let s = self.read_string()?;
                Ok(Token::Str(s))
            }
            Some('t') => {
                self.read_literal("true")?;
                Ok(Token::Bool(true))
            }
            Some('f') => {
                self.read_literal("false")?;
                Ok(Token::Bool(false))
            }
            Some('n') => {
                self.read_literal("null")?;
                Ok(Token::Null)
            }
            Some(c) if c == '-' || c.is_ascii_digit() => {
                let (n, is_float) = self.read_number()?;
                Ok(Token::Number(n, is_float))
            }
            Some(c) => Err(JsonError::at(
                format!("unexpected character '{c}'"),
                self.pos,
            )),
        }
    }

    /// Returns the next token without advancing the internal position pointer.
    pub fn peek_token(&mut self) -> Result<Token, JsonError> {
        let saved = self.pos;
        let tok = self.next_token()?;
        self.pos = saved;
        Ok(tok)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self) -> Result<String, JsonError> {
        let mut out = String::new();
        loop {
            match self.current_char() {
                None => return Err(JsonError::at("unterminated string", self.pos)),
                Some('"') => {
                    self.advance();
                    return Ok(out);
                }
                Some('\\') => {
                    self.advance();
                    match self.current_char() {
                        Some('"') => {
                            out.push('"');
                            self.advance();
                        }
                        Some('\\') => {
                            out.push('\\');
                            self.advance();
                        }
                        Some('/') => {
                            out.push('/');
                            self.advance();
                        }
                        Some('n') => {
                            out.push('\n');
                            self.advance();
                        }
                        Some('r') => {
                            out.push('\r');
                            self.advance();
                        }
                        Some('t') => {
                            out.push('\t');
                            self.advance();
                        }
                        Some('b') => {
                            out.push('\x08');
                            self.advance();
                        }
                        Some('f') => {
                            out.push('\x0C');
                            self.advance();
                        }
                        Some('u') => {
                            self.advance();
                            let hex = self.read_hex4()?;
                            if (0xD800..=0xDBFF).contains(&hex) {
                                if self.current_char() == Some('\\') {
                                    self.advance();
                                    if self.current_char() == Some('u') {
                                        self.advance();
                                        let low = self.read_hex4()?;
                                        if (0xDC00..=0xDFFF).contains(&low) {
                                            let code_point = 0x10000
                                                + ((hex - 0xD800) as u32) * 0x400
                                                + (low - 0xDC00) as u32;
                                            if let Some(ch) = char::from_u32(code_point) {
                                                out.push(ch);
                                            } else {
                                                return Err(JsonError::at(
                                                    "invalid unicode surrogate pair",
                                                    self.pos,
                                                ));
                                            }
                                            continue;
                                        }
                                    }
                                }
                                return Err(JsonError::at("unpaired high surrogate", self.pos));
                            } else if let Some(ch) = char::from_u32(hex as u32) {
                                out.push(ch);
                            } else {
                                return Err(JsonError::at("invalid unicode code point", self.pos));
                            }
                        }
                        Some(c) => {
                            return Err(JsonError::at(
                                format!("invalid escape sequence '\\{c}'"),
                                self.pos,
                            ));
                        }
                        None => return Err(JsonError::at("unexpected end in escape", self.pos)),
                    }
                }
                Some(c) => {
                    out.push(c);
                    self.advance();
                }
            }
        }
    }

    fn read_hex4(&mut self) -> Result<u16, JsonError> {
        let mut val: u32 = 0;
        for _ in 0..4 {
            match self.current_char() {
                Some(c) if c.is_ascii_hexdigit() => {
                    val = val * 16 + c.to_digit(16).unwrap();
                    self.advance();
                }
                _ => return Err(JsonError::at("expected 4 hex digits in \\uXXXX", self.pos)),
            }
        }
        Ok(val as u16)
    }

    fn read_number(&mut self) -> Result<(f64, bool), JsonError> {
        let start = self.pos;
        let mut is_float = false;
        if self.current_char() == Some('-') {
            self.advance();
        }
        if self.current_char() == Some('0') {
            self.advance();
        } else if self
            .current_char()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            while self
                .current_char()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                self.advance();
            }
        } else {
            return Err(JsonError::at("invalid number", self.pos));
        }
        if self.current_char() == Some('.') {
            is_float = true;
            self.advance();
            if !self
                .current_char()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return Err(JsonError::at(
                    "digit expected after decimal point",
                    self.pos,
                ));
            }
            while self
                .current_char()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                self.advance();
            }
        }
        if matches!(self.current_char(), Some('e') | Some('E')) {
            is_float = true;
            self.advance();
            if matches!(self.current_char(), Some('+') | Some('-')) {
                self.advance();
            }
            if !self
                .current_char()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return Err(JsonError::at("digit expected in exponent", self.pos));
            }
            while self
                .current_char()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                self.advance();
            }
        }
        let slice = &self.input[start..self.pos];
        let val = slice
            .parse::<f64>()
            .map_err(|_| JsonError::at(format!("invalid number '{slice}'"), start))?;
        Ok((val, is_float))
    }

    fn read_literal(&mut self, lit: &str) -> Result<(), JsonError> {
        let end = self.pos + lit.len();
        if self.input.get(self.pos..end) == Some(lit) {
            self.pos = end;
            Ok(())
        } else {
            Err(JsonError::at(format!("expected literal '{lit}'"), self.pos))
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(c) = self.current_char() {
            self.pos += c.len_utf8();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(input: &str) -> Vec<Token> {
        let mut lex = Lexer::new(input);
        let mut out = vec![];
        loop {
            let t = lex.next_token().unwrap();
            if t == Token::Eof {
                break;
            }
            out.push(t);
        }
        out
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(
            tokens("{}[],:"),
            vec![
                Token::LBrace,
                Token::RBrace,
                Token::LBracket,
                Token::RBracket,
                Token::Comma,
                Token::Colon,
            ]
        );
    }

    #[test]
    fn test_string() {
        assert_eq!(tokens(r#""hello""#), vec![Token::Str("hello".into())]);
    }

    #[test]
    fn test_string_escapes() {
        let t = tokens(r#""\n\t\\\"\/""#);
        assert_eq!(t, vec![Token::Str("\n\t\\\"/".into())]);
    }

    #[test]
    fn test_unicode_escape() {
        let t = tokens(r#""\u0041""#);
        assert_eq!(t, vec![Token::Str("A".into())]);
    }

    #[test]
    fn test_number_int() {
        assert_eq!(tokens("42"), vec![Token::Number(42.0, false)]);
    }

    #[test]
    fn test_number_negative() {
        assert_eq!(tokens("-7"), vec![Token::Number(-7.0, false)]);
    }

    #[test]
    fn test_number_float() {
        assert_eq!(tokens("1.23"), vec![Token::Number(1.23, true)]);
    }

    #[test]
    fn test_number_exponent() {
        let t = tokens("1.5e2");
        assert_eq!(t, vec![Token::Number(150.0, true)]);
    }

    #[test]
    fn test_bool_null() {
        assert_eq!(
            tokens("true false null"),
            vec![Token::Bool(true), Token::Bool(false), Token::Null,]
        );
    }

    #[test]
    fn test_error_unexpected_char() {
        let mut lex = Lexer::new("@");
        assert!(lex.next_token().is_err());
    }

    #[test]
    fn test_error_unterminated_string() {
        let mut lex = Lexer::new("\"abc");
        assert!(lex.next_token().is_err());
    }

    #[test]
    fn test_peek_does_not_advance() {
        let mut lex = Lexer::new("true");
        let p1 = lex.peek_token().unwrap();
        let p2 = lex.peek_token().unwrap();
        assert_eq!(p1, p2);
        let next = lex.next_token().unwrap();
        assert_eq!(next, Token::Bool(true));
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(tokens(r#""""#), vec![Token::Str(String::new())]);
    }
}
