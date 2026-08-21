//! Native JSON engine — high-performance parse and stringify functionality.
//!
//! This module implements a zero-dependency JSON utility suite, including the
//! core lexer and parser logic, to minimize build times and runtime dependencies.

mod lexer;
mod parser;

pub use lexer::{Lexer, Token};
pub use parser::Parser;

use crate::error::JsonError;
use crate::value::Value;

/// Parses a raw JSON string into a structured [`Value`].
///
/// # Errors
///
/// Returns a [`JsonError`] if the input payload contains syntax errors or malformed tokens.
///
/// # Example
///
/// ```rust
/// use adapters::json::parse;
/// let v = parse(r#"{"key": 42}"#).unwrap();
/// assert_eq!(v.get("key").unwrap().as_int(), Some(42));
/// ```
pub fn parse(input: &str) -> Result<Value, JsonError> {
    Parser::new(input).parse()
}

/// Serializes the given [`Value`] into a compact, single-line JSON string.
pub fn stringify(value: &Value) -> Result<String, JsonError> {
    let mut buf = String::new();
    write_value(&mut buf, value);
    Ok(buf)
}

/// Serializes the given [`Value`] into a pretty-printed JSON string (2-space indenting).
pub fn stringify_pretty(value: &Value) -> Result<String, JsonError> {
    let mut buf = String::new();
    write_value_pretty(&mut buf, value, 0);
    Ok(buf)
}

fn write_value(buf: &mut String, value: &Value) {
    match value {
        Value::Null => buf.push_str("null"),
        Value::Bool(b) => buf.push_str(if *b { "true" } else { "false" }),
        Value::Int(n) => {
            buf.push_str(&n.to_string());
        }
        Value::Float(f) => {
            if f.is_finite() {
                let s = format_float(*f);
                buf.push_str(&s);
            } else {
                buf.push_str("null");
            }
        }
        Value::String(s) => write_json_string(buf, s),
        Value::Array(arr) => {
            buf.push('[');
            for (i, v) in arr.iter().enumerate() {
                if i > 0 {
                    buf.push(',');
                }
                write_value(buf, v);
            }
            buf.push(']');
        }
        Value::Object(map) => {
            buf.push('{');
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    buf.push(',');
                }
                write_json_string(buf, k);
                buf.push(':');
                write_value(buf, v);
            }
            buf.push('}');
        }
    }
}

fn write_value_pretty(buf: &mut String, value: &Value, indent: usize) {
    match value {
        Value::Array(arr) => {
            if arr.is_empty() {
                buf.push_str("[]");
                return;
            }
            buf.push('[');
            for (i, v) in arr.iter().enumerate() {
                if i > 0 {
                    buf.push(',');
                }
                buf.push('\n');
                push_indent(buf, indent + 1);
                write_value_pretty(buf, v, indent + 1);
            }
            buf.push('\n');
            push_indent(buf, indent);
            buf.push(']');
        }
        Value::Object(map) => {
            if map.is_empty() {
                buf.push_str("{}");
                return;
            }
            buf.push('{');
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    buf.push(',');
                }
                buf.push('\n');
                push_indent(buf, indent + 1);
                write_json_string(buf, k);
                buf.push_str(": ");
                write_value_pretty(buf, v, indent + 1);
            }
            buf.push('\n');
            push_indent(buf, indent);
            buf.push('}');
        }
        other => write_value(buf, other),
    }
}

fn push_indent(buf: &mut String, level: usize) {
    for _ in 0..level {
        buf.push_str("  ");
    }
}

fn write_json_string(buf: &mut String, s: &str) {
    buf.push('"');
    for ch in s.chars() {
        match ch {
            '"' => buf.push_str("\\\""),
            '\\' => buf.push_str("\\\\"),
            '\n' => buf.push_str("\\n"),
            '\r' => buf.push_str("\\r"),
            '\t' => buf.push_str("\\t"),
            '\x08' => buf.push_str("\\b"),
            '\x0C' => buf.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                buf.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => buf.push(c),
        }
    }
    buf.push('"');
}

fn format_float(f: f64) -> String {
    let s = format!("{f}");
    if s.contains('.') && !s.contains('e') && !s.contains('E') {
        let trimmed = s.trim_end_matches('0');
        if trimmed.ends_with('.') {
            format!("{trimmed}0")
        } else {
            trimmed.to_string()
        }
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_parse_null() {
        assert_eq!(parse("null").unwrap(), Value::Null);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse("true").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(parse("99").unwrap(), Value::Int(99));
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse("1.23").unwrap(), Value::Float(1.23));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse(r#""hello""#).unwrap(), Value::String("hello".into()));
    }

    #[test]
    fn test_stringify_null() {
        assert_eq!(stringify(&Value::Null).unwrap(), "null");
    }

    #[test]
    fn test_stringify_bool() {
        assert_eq!(stringify(&Value::Bool(false)).unwrap(), "false");
    }

    #[test]
    fn test_stringify_int() {
        assert_eq!(stringify(&Value::Int(-5)).unwrap(), "-5");
    }

    #[test]
    fn test_stringify_float() {
        let s = stringify(&Value::Float(1.23)).unwrap();
        assert!(s.starts_with("1.23"), "got: {s}");
    }

    #[test]
    fn test_stringify_string_escapes() {
        let s = stringify(&Value::String("a\"b\\c\nd".into())).unwrap();
        assert_eq!(s, r#""a\"b\\c\nd""#);
    }

    #[test]
    fn test_roundtrip_object() {
        let original = r#"{"age":30,"name":"alice"}"#;
        let v = parse(original).unwrap();
        let out = stringify(&v).unwrap();
        assert_eq!(out, original);
    }

    #[test]
    fn test_roundtrip_nested() {
        let original = r#"{"user":{"active":true,"score":9.5}}"#;
        let v = parse(original).unwrap();
        let out = stringify(&v).unwrap();
        assert_eq!(out, original);
    }

    #[test]
    fn test_roundtrip_all_types() {
        let original = r#"[null,true,42,1.23,"hi",{},[]]"#;
        let v = parse(original).unwrap();
        let out = stringify(&v).unwrap();
        assert_eq!(out, original);
    }

    #[test]
    fn test_pretty_print() {
        let mut m = BTreeMap::new();
        m.insert("a".to_string(), Value::Int(1));
        let v = Value::Object(m);
        let pretty = stringify_pretty(&v).unwrap();
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));
    }

    #[test]
    fn test_parse_unicode_escape() {
        let v = parse(r#""\u0048\u0065\u006C\u006C\u006F""#).unwrap();
        assert_eq!(v, Value::String("Hello".into()));
    }

    #[test]
    fn test_parse_empty_object() {
        assert_eq!(parse("{}").unwrap(), Value::Object(BTreeMap::new()));
    }

    #[test]
    fn test_parse_empty_array() {
        assert_eq!(parse("[]").unwrap(), Value::Array(vec![]));
    }

    #[test]
    fn test_parse_malformed() {
        assert!(parse("{bad}").is_err());
        assert!(parse("[1,2,").is_err());
    }

    #[test]
    fn test_int_vs_float() {
        assert!(matches!(parse("10").unwrap(), Value::Int(10)));
        assert!(matches!(parse("10.0").unwrap(), Value::Float(_)));
        assert!(matches!(parse("1.5e2").unwrap(), Value::Float(_)));
    }

    #[test]
    fn test_stringify_pretty_empty_containers() {
        assert_eq!(stringify_pretty(&Value::Array(vec![])).unwrap(), "[]");
        assert_eq!(
            stringify_pretty(&Value::Object(BTreeMap::new())).unwrap(),
            "{}"
        );
    }
}
