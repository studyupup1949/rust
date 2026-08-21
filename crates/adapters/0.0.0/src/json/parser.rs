//! Recursive-descent JSON parser that produces a [`Value`] from a token stream.
//!
//! Implements standard JSON specification grammar validation and AST tree building.

use super::lexer::{Lexer, Token};
use crate::error::JsonError;
use crate::value::Value;
use std::collections::BTreeMap;

/// Recursive-descent parser working over a custom stream of lexical tokens.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    /// Creates a new recursive-descent parser instance bound to the input string lifetime.
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    /// Parses the entire string payload, checking for trailing garbage tokens.
    ///
    /// # Errors
    ///
    /// Returns a [`JsonError`] if the payload violates standard JSON format rules.
    pub fn parse(&mut self) -> Result<Value, JsonError> {
        let val = self.parse_value()?;
        match self.lexer.next_token()? {
            Token::Eof => Ok(val),
            tok => Err(JsonError::new(format!(
                "unexpected token after value: {tok:?}"
            ))),
        }
    }

    fn parse_value(&mut self) -> Result<Value, JsonError> {
        match self.lexer.next_token()? {
            Token::LBrace => self.parse_object_body(),
            Token::LBracket => self.parse_array_body(),
            Token::Str(s) => Ok(Value::String(s)),
            Token::Number(n, is_float) => Ok(self.parse_number(n, is_float)),
            Token::Bool(b) => Ok(Value::Bool(b)),
            Token::Null => Ok(Value::Null),
            tok => Err(JsonError::new(format!("unexpected token: {tok:?}"))),
        }
    }

    fn parse_object_body(&mut self) -> Result<Value, JsonError> {
        let mut map = BTreeMap::new();
        if self.lexer.peek_token()? == Token::RBrace {
            self.lexer.next_token()?;
            return Ok(Value::Object(map));
        }
        loop {
            let key = match self.lexer.next_token()? {
                Token::Str(s) => s,
                tok => return Err(JsonError::new(format!("expected string key, got {tok:?}"))),
            };
            self.expect_token(Token::Colon)?;
            let val = self.parse_value()?;
            map.insert(key, val);
            match self.lexer.next_token()? {
                Token::Comma => {
                    if self.lexer.peek_token()? == Token::RBrace {
                        self.lexer.next_token()?;
                        break;
                    }
                }
                Token::RBrace => break,
                tok => return Err(JsonError::new(format!("expected ',' or '}}', got {tok:?}"))),
            }
        }
        Ok(Value::Object(map))
    }

    fn parse_array_body(&mut self) -> Result<Value, JsonError> {
        let mut arr = Vec::new();
        if self.lexer.peek_token()? == Token::RBracket {
            self.lexer.next_token()?;
            return Ok(Value::Array(arr));
        }
        loop {
            arr.push(self.parse_value()?);
            match self.lexer.next_token()? {
                Token::Comma => {
                    if self.lexer.peek_token()? == Token::RBracket {
                        self.lexer.next_token()?;
                        break;
                    }
                }
                Token::RBracket => break,
                tok => return Err(JsonError::new(format!("expected ',' or ']', got {tok:?}"))),
            }
        }
        Ok(Value::Array(arr))
    }

    fn parse_number(&self, n: f64, is_float: bool) -> Value {
        if !is_float && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
            Value::Int(n as i64)
        } else {
            Value::Float(n)
        }
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), JsonError> {
        let got = self.lexer.next_token()?;
        if got == expected {
            Ok(())
        } else {
            Err(JsonError::new(format!(
                "expected {expected:?}, got {got:?}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Value {
        Parser::new(s).parse().unwrap()
    }

    #[test]
    fn test_null() {
        assert_eq!(parse("null"), Value::Null);
    }

    #[test]
    fn test_bool() {
        assert_eq!(parse("true"), Value::Bool(true));
        assert_eq!(parse("false"), Value::Bool(false));
    }

    #[test]
    fn test_int() {
        assert_eq!(parse("42"), Value::Int(42));
        assert_eq!(parse("-7"), Value::Int(-7));
    }

    #[test]
    fn test_float() {
        assert_eq!(parse("1.23"), Value::Float(1.23));
        assert!(matches!(parse("1.5e2"), Value::Float(_)));
    }

    #[test]
    fn test_string() {
        assert_eq!(parse(r#""hello""#), Value::String("hello".into()));
    }

    #[test]
    fn test_empty_object() {
        assert_eq!(parse("{}"), Value::Object(BTreeMap::new()));
    }

    #[test]
    fn test_empty_array() {
        assert_eq!(parse("[]"), Value::Array(vec![]));
    }

    #[test]
    fn test_simple_object() {
        let v = parse(r#"{"a":1,"b":true}"#);
        assert_eq!(v.get("a"), Some(&Value::Int(1)));
        assert_eq!(v.get("b"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_array_of_values() {
        let v = parse("[1,\"two\",null,false]");
        let arr = v.as_array().unwrap();
        assert_eq!(arr[0], Value::Int(1));
        assert_eq!(arr[1], Value::String("two".into()));
        assert_eq!(arr[2], Value::Null);
        assert_eq!(arr[3], Value::Bool(false));
    }

    #[test]
    fn test_nested() {
        let v = parse(r#"{"user":{"name":"alice","age":30}}"#);
        let user = v.get("user").unwrap();
        assert_eq!(user.get("name"), Some(&Value::String("alice".into())));
        assert_eq!(user.get("age"), Some(&Value::Int(30)));
    }

    #[test]
    fn test_deeply_nested() {
        let v = parse(r#"{"a":{"b":{"c":{"d":42}}}}"#);
        let inner = v
            .get("a")
            .unwrap()
            .get("b")
            .unwrap()
            .get("c")
            .unwrap()
            .get("d")
            .unwrap();
        assert_eq!(inner, &Value::Int(42));
    }

    #[test]
    fn test_malformed_missing_value() {
        assert!(Parser::new(r#"{"key":}"#).parse().is_err());
    }

    #[test]
    fn test_malformed_trailing_data() {
        assert!(Parser::new("true false").parse().is_err());
    }

    #[test]
    fn test_malformed_unclosed_object() {
        assert!(Parser::new(r#"{"a":1"#).parse().is_err());
    }
}
