//! Lexer stage: tokenizes raw AAML text into a stream of tokens.
//!
//! The Lexer scans through raw text and produces a `Vec<Token>` with positional
//! information preserved for error diagnostics.

use crate::error::{AamlError, ErrorDiagnostics};

/// A single token produced by the Lexer.
///
/// Each token carries its line and column number for error reporting.
#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
    pub text: std::borrow::Cow<'a, str>,
}

impl<'a> Token<'a> {
    pub fn new(
        kind: TokenKind,
        line: usize,
        column: usize,
        text: impl Into<std::borrow::Cow<'a, str>>,
    ) -> Self {
        Self {
            kind,
            line,
            column,
            text: text.into(),
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
    pub fn new() -> Self {
        Self
    }

    /// Checks if a character is whitespace (excluding newlines)
    fn is_whitespace(c: char) -> bool {
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
    fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    /// Checks if a character can be part of a number
    fn is_number_part(c: char) -> bool {
        c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E'
    }
}

impl Default for DefaultLexer {
    fn default() -> Self {
        Self::new()
    }
}

impl Lexer for DefaultLexer {
    fn tokenize<'a>(&self, content: &'a str) -> Result<Vec<Token<'a>>, AamlError> {
        let mut tokens = Vec::new();
        let mut line = 1;
        let mut column = 1;
        let mut chars = content.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                '\n' => {
                    self.handle_newline(&mut tokens, &mut chars, &mut line, &mut column);
                }
                c if Self::is_whitespace(c) => {
                    chars.next();
                    column += 1;
                }
                '#' => {
                    if Self::is_comment_start(&chars) {
                        self.handle_comment(&mut tokens, &mut chars, line, &mut column);
                    } else {
                        self.handle_identifier(&mut tokens, &mut chars, line, &mut column);
                    }
                }
                '=' => self.push_single_token(
                    &mut tokens,
                    TokenKind::Assign,
                    line,
                    column,
                    "=",
                    &mut chars,
                    &mut column,
                ),
                '{' => self.push_single_token(
                    &mut tokens,
                    TokenKind::LeftBrace,
                    line,
                    column,
                    "{",
                    &mut chars,
                    &mut column,
                ),
                '}' => self.push_single_token(
                    &mut tokens,
                    TokenKind::RightBrace,
                    line,
                    column,
                    "}",
                    &mut chars,
                    &mut column,
                ),
                '[' => self.push_single_token(
                    &mut tokens,
                    TokenKind::LeftBracket,
                    line,
                    column,
                    "[",
                    &mut chars,
                    &mut column,
                ),
                ']' => self.push_single_token(
                    &mut tokens,
                    TokenKind::RightBracket,
                    line,
                    column,
                    "]",
                    &mut chars,
                    &mut column,
                ),
                ',' => self.push_single_token(
                    &mut tokens,
                    TokenKind::Comma,
                    line,
                    column,
                    ",",
                    &mut chars,
                    &mut column,
                ),
                '@' => self.push_single_token(
                    &mut tokens,
                    TokenKind::At,
                    line,
                    column,
                    "@",
                    &mut chars,
                    &mut column,
                ),
                '"' | '\'' => {
                    self.handle_string(&mut tokens, &mut chars, ch, line, &mut column, &mut line)?;
                }
                _ if Self::is_digit(ch)
                    || (ch == '-' && chars.clone().nth(1).is_some_and(Self::is_digit)) =>
                {
                    self.handle_number(&mut tokens, &mut chars, ch, line, &mut column);
                }
                _ if Self::is_id_start(ch) => {
                    self.handle_identifier(&mut tokens, &mut chars, line, &mut column);
                }
                _ => {
                    return Err(AamlError::LexError {
                        line,
                        column,
                        character: ch.to_string(),
                        diagnostics: Some(ErrorDiagnostics::new(
                            "Invalid character in input",
                            format!("Unexpected character '{}' at {}:{}", ch, line, column),
                            "Check for typos or unsupported characters",
                        )),
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
            ));
        }

        Ok(tokens)
    }
}

impl DefaultLexer {
    fn handle_newline(
        &self,
        tokens: &mut Vec<Token>,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        line: &mut usize,
        column: &mut usize,
    ) {
        tokens.push(Token::new(
            TokenKind::Newline,
            *line,
            *column,
            "\n".to_string(),
        ));
        chars.next();
        *line += 1;
        *column = 1;
    }

    fn handle_comment(
        &self,
        tokens: &mut Vec<Token>,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        line: usize,
        column: &mut usize,
    ) {
        let col = *column;
        let mut text = String::new();
        while let Some(&c) = chars.peek() {
            if c == '\n' {
                break;
            }
            text.push(c);
            chars.next();
            *column += 1;
        }
        tokens.push(Token::new(TokenKind::Comment, line, col, text));
    }

    fn push_single_token(
        &self,
        tokens: &mut Vec<Token>,
        kind: TokenKind,
        line: usize,
        column: usize,
        text: &str,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        col_ref: &mut usize,
    ) {
        tokens.push(Token::new(kind, line, column, text.to_string()));
        chars.next();
        *col_ref += 1;
    }
    fn update_string_scan_state(
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

    fn handle_string(
        &self,
        tokens: &mut Vec<Token>,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        quote: char,
        mut line: usize,
        column: &mut usize,
        line_ref: &mut usize,
    ) -> Result<(), AamlError> {
        let col = *column;
        chars.next();
        *column += 1;
        let mut text = String::from(quote);
        let mut escaped = false;

        while let Some(&c) = chars.peek() {
            text.push(c);
            chars.next();
            *column += 1;

            if Self::update_string_scan_state(c, quote, &mut escaped, &mut line, column) {
                break;
            }
        }

        tokens.push(Token::new(TokenKind::String, line, col, text));
        *line_ref = line;
        Ok(())
    }

    fn handle_number(
        &self,
        tokens: &mut Vec<Token>,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        first_ch: char,
        line: usize,
        column: &mut usize,
    ) {
        let col = *column;
        let mut text = String::new();

        if first_ch == '-' {
            text.push('-');
            chars.next();
            *column += 1;
        }

        while let Some(&c) = chars.peek() {
            if Self::is_number_part(c) {
                text.push(c);
                chars.next();
                *column += 1;
            } else {
                break;
            }
        }

        let kind = if text == "true" || text == "false" {
            TokenKind::Boolean
        } else {
            TokenKind::Number
        };

        tokens.push(Token::new(kind, line, col, text));
    }

    fn handle_identifier(
        &self,
        tokens: &mut Vec<Token>,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        line: usize,
        column: &mut usize,
    ) {
        let col = *column;
        let mut text = String::new();

        while let Some(&c) = chars.peek() {
            if Self::is_id_cont(c) {
                text.push(c);
                chars.next();
                *column += 1;
            } else {
                break;
            }
        }

        let kind = match text.as_str() {
            "true" | "false" => TokenKind::Boolean,
            _ => TokenKind::Identifier,
        };

        tokens.push(Token::new(kind, line, col, text));
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
        assert_eq!(tokens[1].text, "import");
    }

    #[test]
    fn test_comment() {
        let lexer = DefaultLexer::new();
        let tokens = lexer
            .tokenize("host = localhost # This is a comment")
            .unwrap();

        assert!(tokens.iter().any(|t| t.kind == TokenKind::Comment));
    }

    #[test]
    fn test_hex_color_is_not_comment() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("tint = #ff6600").unwrap();

        assert!(!tokens.iter().any(|t| t.kind == TokenKind::Comment));
        assert!(
            tokens
                .iter()
                .any(|t| t.kind == TokenKind::Identifier && t.text == "#ff6600")
        );
    }

    #[test]
    fn test_schema_generics_and_url_tokenize() {
        let lexer = DefaultLexer::new();
        let content = "@schema S { items: list<string> }\nurl = https://localhost/test";
        let tokens = lexer.tokenize(content).unwrap();

        assert!(
            tokens
                .iter()
                .any(|t| t.kind == TokenKind::Identifier && t.text == "list<string>")
        );
        assert!(
            tokens
                .iter()
                .any(|t| t.kind == TokenKind::Identifier && t.text == "https://localhost/test")
        );
    }

    #[test]
    fn test_absolute_path_tokenize() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("root = /srv/app").unwrap();

        assert!(
            tokens
                .iter()
                .any(|t| t.kind == TokenKind::Identifier && t.text == "/srv/app")
        );
    }
}
