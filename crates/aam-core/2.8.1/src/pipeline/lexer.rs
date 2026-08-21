//! Lexer stage: tokenizes raw AAML text into a stream of tokens.
//!
//! The Lexer scans through raw text and produces a `Vec<Token>` with positional
//! information preserved for error diagnostics.

use crate::error::{AamlError, ErrorDiagnostics};

/// A single token produced by the Lexer.
///
/// Each token carries its line and column number for error reporting and its
/// byte span inside the original source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
    pub text: std::borrow::Cow<'a, str>,
    pub start: usize,
    pub end: usize,
    pub source: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(
        kind: TokenKind,
        line: usize,
        column: usize,
        text: impl Into<std::borrow::Cow<'a, str>>,
        start: usize,
        end: usize,
        source: &'a str,
    ) -> Self {
        Self {
            kind,
            line,
            column,
            text: text.into(),
            start,
            end,
            source,
        }
    }
}

/// The type of token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// Identifier or unquoted value (e.g., `host`, `localhost`)
    Identifier,
    /// The `=` operator in assignments
    Assign,
    /// String literal (quoted with `"` or `'`)
    String,
    /// Number literal (integer or float)
    Number,
    /// Boolean literal (`true` or `false`)
    Boolean,
    /// Opening brace `{`
    LeftBrace,
    /// Closing brace `}`
    RightBrace,
    /// Opening bracket `[`
    LeftBracket,
    /// Closing bracket `]`
    RightBracket,
    /// Comma separator `,`
    Comma,
    /// The `@` directive prefix
    At,
    /// End of line / newline
    Newline,
    /// Comment (including the `#`)
    Comment,
}

/// Trait for lexical analysis stage.
pub trait Lexer: Send + Sync {
    /// Tokenizes raw AAML content and returns a stream of tokens with line/column info.
    ///
    /// # Errors
    /// Returns `AamlError::LexError` if the input contains invalid tokens or
    /// unclosed delimiters.
    fn tokenize<'a>(&self, content: &'a str) -> Result<Vec<Token<'a>>, AamlError>;
}

/// Default implementation of the Lexer stage.
pub struct DefaultLexer;

impl DefaultLexer {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Checks if a character is whitespace (excluding newlines)
    const fn is_whitespace(c: char) -> bool {
        c == ' ' || c == '\t' || c == '\r'
    }

    /// Checks if a character can start an identifier
    fn is_id_start(c: char) -> bool {
        c.is_alphabetic() || c == '_' || c == '@' || c == '#' || c == '/'
    }

    /// Checks if a character can continue an identifier
    fn is_id_cont(c: char) -> bool {
        c.is_alphanumeric()
            || c == '_'
            || c == ':'
            || c == '.'
            || c == '*'
            || c == '#'
            || c == '-'
            || c == '/'
            || c == '<'
            || c == '>'
    }

    // Keep legacy quirk: `#` starts a comment only when followed by whitespace.
    fn is_comment_start(chars: &std::iter::Peekable<std::str::Chars>) -> bool {
        chars.clone().nth(1).is_some_and(char::is_whitespace)
    }

    /// Checks if a character is a digit
    const fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    /// Checks if a character can be part of a number
    const fn is_number_part(c: char) -> bool {
        c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E'
    }
}

impl Default for DefaultLexer {
    fn default() -> Self {
        Self::new()
    }
}

impl Lexer for DefaultLexer {
    #[allow(clippy::too_many_lines)]
    fn tokenize<'a>(&self, content: &'a str) -> Result<Vec<Token<'a>>, AamlError> {
        let mut tokens = Vec::new();
        let mut line = 1;
        let mut column = 1;
        let mut byte_offset = 0usize;
        let mut chars = content.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                '\n' => {
                    Self::handle_newline(
                        &mut tokens,
                        &mut chars,
                        &mut line,
                        &mut column,
                        &mut byte_offset,
                        content,
                    );
                }
                c if Self::is_whitespace(c) => {
                    chars.next();
                    column += 1;
                    byte_offset += c.len_utf8();
                }
                '#' => {
                    if Self::is_comment_start(&chars) {
                        Self::handle_comment(
                            &mut tokens,
                            &mut chars,
                            line,
                            &mut column,
                            &mut byte_offset,
                            content,
                        );
                    } else {
                        Self::handle_identifier(
                            &mut tokens,
                            &mut chars,
                            line,
                            &mut column,
                            &mut byte_offset,
                            content,
                        );
                    }
                }
                '=' => Self::push_single_token(
                    &mut tokens,
                    TokenKind::Assign,
                    line,
                    column,
                    "=",
                    &mut chars,
                    &mut column,
                    &mut byte_offset,
                    content,
                ),
                '{' => Self::push_single_token(
                    &mut tokens,
                    TokenKind::LeftBrace,
                    line,
                    column,
                    "{",
                    &mut chars,
                    &mut column,
                    &mut byte_offset,
                    content,
                ),
                '}' => Self::push_single_token(
                    &mut tokens,
                    TokenKind::RightBrace,
                    line,
                    column,
                    "}",
                    &mut chars,
                    &mut column,
                    &mut byte_offset,
                    content,
                ),
                '[' => Self::push_single_token(
                    &mut tokens,
                    TokenKind::LeftBracket,
                    line,
                    column,
                    "[",
                    &mut chars,
                    &mut column,
                    &mut byte_offset,
                    content,
                ),
                ']' => Self::push_single_token(
                    &mut tokens,
                    TokenKind::RightBracket,
                    line,
                    column,
                    "]",
                    &mut chars,
                    &mut column,
                    &mut byte_offset,
                    content,
                ),
                ',' => Self::push_single_token(
                    &mut tokens,
                    TokenKind::Comma,
                    line,
                    column,
                    ",",
                    &mut chars,
                    &mut column,
                    &mut byte_offset,
                    content,
                ),
                '@' => Self::push_single_token(
                    &mut tokens,
                    TokenKind::At,
                    line,
                    column,
                    "@",
                    &mut chars,
                    &mut column,
                    &mut byte_offset,
                    content,
                ),
                '"' | '\'' => {
                    Self::handle_string(
                        &mut tokens,
                        &mut chars,
                        ch,
                        line,
                        &mut column,
                        &mut line,
                        &mut byte_offset,
                        content,
                    );
                }
                _ if Self::is_digit(ch)
                    || (ch == '-' && chars.clone().nth(1).is_some_and(Self::is_digit)) =>
                {
                    Self::handle_number(
                        &mut tokens,
                        &mut chars,
                        ch,
                        line,
                        &mut column,
                        &mut byte_offset,
                        content,
                    );
                }
                _ if Self::is_id_start(ch) => {
                    Self::handle_identifier(
                        &mut tokens,
                        &mut chars,
                        line,
                        &mut column,
                        &mut byte_offset,
                        content,
                    );
                }
                _ => {
                    return Err(AamlError::LexError {
                        line,
                        column,
                        character: ch.to_string(),
                        diagnostics: Some(Box::new(ErrorDiagnostics::new(
                            "Invalid character in input",
                            format!("Unexpected character '{ch}' at {line}:{column}"),
                            "Check for typos or unsupported characters",
                        ))),
                    });
                }
            }
        }

        // Add final newline if not present
        if tokens.is_empty() || tokens.last().is_none_or(|t| t.kind != TokenKind::Newline) {
            tokens.push(Token::new(
                TokenKind::Newline,
                line,
                column,
                "\n".to_string(),
                byte_offset,
                byte_offset + 1,
                content,
            ));
        }

        Ok(tokens)
    }
}

impl DefaultLexer {
    fn handle_newline<'a>(
        tokens: &mut Vec<Token<'a>>,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        line: &mut usize,
        column: &mut usize,
        byte_offset: &mut usize,
        source: &'a str,
    ) {
        let start = *byte_offset;
        tokens.push(Token::new(
            TokenKind::Newline,
            *line,
            *column,
            "\n".to_string(),
            start,
            start + 1,
            source,
        ));
        chars.next();
        *line += 1;
        *column = 1;
        *byte_offset += 1;
    }

    fn handle_comment<'a>(
        tokens: &mut Vec<Token<'a>>,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        line: usize,
        column: &mut usize,
        byte_offset: &mut usize,
        source: &'a str,
    ) {
        let col = *column;
        let start = *byte_offset;
        let mut text = String::new();
        while let Some(&c) = chars.peek() {
            if c == '\n' {
                break;
            }
            text.push(c);
            chars.next();
            *column += 1;
            *byte_offset += c.len_utf8();
        }
        tokens.push(Token::new(
            TokenKind::Comment,
            line,
            col,
            text,
            start,
            *byte_offset,
            source,
        ));
    }

    #[allow(clippy::too_many_arguments)]
    fn push_single_token<'a>(
        tokens: &mut Vec<Token<'a>>,
        kind: TokenKind,
        line: usize,
        column: usize,
        text: &str,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        col_ref: &mut usize,
        byte_offset: &mut usize,
        source: &'a str,
    ) {
        let start = *byte_offset;
        let end = start + text.len();
        tokens.push(Token::new(
            kind,
            line,
            column,
            text.to_string(),
            start,
            end,
            source,
        ));
        chars.next();
        *col_ref += 1;
        *byte_offset = end;
    }

    const fn update_string_scan_state(
        c: char,
        quote: char,
        escaped: &mut bool,
        line: &mut usize,
        column: &mut usize,
    ) -> bool {
        if *escaped {
            *escaped = false;
            return false;
        }

        if c == '\\' {
            *escaped = true;
            return false;
        }

        if c == quote {
            return true;
        }

        if c == '\n' {
            *line += 1;
            *column = 1;
        }

        false
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_string<'a>(
        tokens: &mut Vec<Token<'a>>,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        quote: char,
        mut line: usize,
        column: &mut usize,
        line_ref: &mut usize,
        byte_offset: &mut usize,
        source: &'a str,
    ) {
        let col = *column;
        let start = *byte_offset;
        chars.next();
        *column += 1;
        *byte_offset += quote.len_utf8();
        let mut text = String::from(quote);
        let mut escaped = false;

        while let Some(&c) = chars.peek() {
            text.push(c);
            chars.next();
            *column += 1;
            *byte_offset += c.len_utf8();

            if Self::update_string_scan_state(c, quote, &mut escaped, &mut line, column) {
                break;
            }
        }

        tokens.push(Token::new(
            TokenKind::String,
            line,
            col,
            text,
            start,
            *byte_offset,
            source,
        ));
        *line_ref = line;
    }

    fn handle_number<'a>(
        tokens: &mut Vec<Token<'a>>,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        first_ch: char,
        line: usize,
        column: &mut usize,
        byte_offset: &mut usize,
        source: &'a str,
    ) {
        let col = *column;
        let start = *byte_offset;
        let mut text = String::new();

        if first_ch == '-' {
            text.push('-');
            chars.next();
            *column += 1;
            *byte_offset += '-'.len_utf8();
        }

        while let Some(&c) = chars.peek() {
            if Self::is_number_part(c) {
                text.push(c);
                chars.next();
                *column += 1;
                *byte_offset += c.len_utf8();
            } else {
                break;
            }
        }

        let kind = if text == "true" || text == "false" {
            TokenKind::Boolean
        } else {
            TokenKind::Number
        };

        tokens.push(Token::new(
            kind,
            line,
            col,
            text,
            start,
            *byte_offset,
            source,
        ));
    }

    fn handle_identifier<'a>(
        tokens: &mut Vec<Token<'a>>,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        line: usize,
        column: &mut usize,
        byte_offset: &mut usize,
        source: &'a str,
    ) {
        let col = *column;
        let start = *byte_offset;
        let mut text = String::new();

        while let Some(&c) = chars.peek() {
            if Self::is_id_cont(c) {
                text.push(c);
                chars.next();
                *column += 1;
                *byte_offset += c.len_utf8();
            } else {
                break;
            }
        }

        let kind = match text.as_str() {
            "true" | "false" => TokenKind::Boolean,
            _ => TokenKind::Identifier,
        };

        tokens.push(Token::new(
            kind,
            line,
            col,
            text,
            start,
            *byte_offset,
            source,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_assignment() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("host = localhost").unwrap();

        assert_eq!(tokens.len(), 4); // host, =, localhost, newline
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::Assign);
        assert_eq!(tokens[2].kind, TokenKind::Identifier);
        assert_eq!(tokens[3].kind, TokenKind::Newline);
    }

    #[test]
    fn test_quoted_string() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("name = \"John Doe\"").unwrap();

        assert!(tokens.iter().any(|t| t.kind == TokenKind::String));
    }

    #[test]
    fn test_number_literal() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("port = 8080").unwrap();

        assert!(tokens.iter().any(|t| t.kind == TokenKind::Number));
    }

    #[test]
    fn test_boolean_literal() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("enabled = true").unwrap();

        assert!(tokens.iter().any(|t| t.kind == TokenKind::Boolean));
    }

    #[test]
    fn test_braces_and_brackets() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("obj = { key = val }").unwrap();

        assert!(tokens.iter().any(|t| t.kind == TokenKind::LeftBrace));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::RightBrace));
    }

    #[test]
    fn test_directive() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("@import base.aam").unwrap();

        assert_eq!(tokens[0].kind, TokenKind::At);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
    }
}
