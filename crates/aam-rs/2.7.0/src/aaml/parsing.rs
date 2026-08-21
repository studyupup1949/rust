//! Parsing helpers: comment stripping, assignment parsing, multi-line block accumulation.

use crate::error::{AamlError, ErrorDiagnostics};

/// Strips an inline `#` comment from a raw source line, respecting quoted strings.
///
/// A `#` is a comment start only when it is preceded by whitespace (or at line start),
/// so unquoted color values like `tint = #ff6600` are stored correctly.
#[must_use]
pub fn strip_comment(line: &str) -> &str {
    let mut quote_state: Option<char> = None;
    let bytes = line.as_bytes();

    for (idx, c) in line.char_indices() {
        match (quote_state, c) {
            (None, '#') => {
                let preceded_by_space =
                    idx == 0 || bytes.get(idx - 1).is_some_and(u8::is_ascii_whitespace);
                let followed_by_space = bytes.get(idx + 1).is_none_or(u8::is_ascii_whitespace);
                if preceded_by_space && followed_by_space {
                    return &line[..idx];
                }
            }
            (None, '"' | '\'') => quote_state = Some(c),
            (Some(q), c) if c == q => quote_state = None,
            _ => {}
        }
    }
    line
}

/// Parses a `key = value` assignment and returns trimmed (key, value) slices.
///
/// The split point is the **first `=`** that appears outside any
/// `{ ... }` or `[ ... ]` nesting.  This allows values like
/// `pos = { x = 1.0, y = 2.0 }` or `tags = [a, b, c]` to be parsed
/// correctly.  Surrounding quotes are stripped from the value via
/// [`unwrap_quotes`], but `{...}` and `[...]` literals are returned as-is.
pub(super) fn parse_assignment(line: &str) -> Result<(&str, &str), AamlError> {
    // Find the first '=' outside nesting
    let mut depth: i32 = 0;
    let mut eq_pos: Option<usize> = None;
    for (i, ch) in line.char_indices() {
        match ch {
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            '=' if depth == 0 => {
                eq_pos = Some(i);
                break;
            }
            _ => {}
        }
    }

    let pos = eq_pos.ok_or_else(|| AamlError::MalformedLiteral {
        literal_type: "assignment".to_string(),
        content: line.to_string(),
        diagnostics: Some(Box::new(ErrorDiagnostics::new(
            "Missing assignment operator",
            format!("Line '{line}' does not contain '=' separator"),
            "Use format: key = value",
        ))),
    })?;
    let key = line[..pos].trim();
    let raw_val = line[pos + 1..].trim();

    if key.is_empty() {
        return Err(AamlError::InvalidValue {
            details: "Key is empty".to_string(),
            expected: "non-empty key name".to_string(),
            diagnostics: Some(Box::new(ErrorDiagnostics::new(
                "Empty key in assignment",
                format!("Line '{line}' has no key before '='"),
                "Provide a valid key name before the '=' operator",
            ))),
        });
    }

    validate_balanced_delimiters(raw_val, line)?;

    // Do NOT unwrap quotes when the value is an inline object or list literal
    let val = if raw_val.starts_with('{') || raw_val.starts_with('[') {
        raw_val
    } else {
        unwrap_quotes(raw_val)
    };

    Ok((key, val))
}

/// Ensures `{}` and `[]` are balanced in `value`, ignoring bracket-like chars in quotes.
fn validate_balanced_delimiters(value: &str, line: &str) -> Result<(), AamlError> {
    let mut stack: Vec<char> = Vec::new();
    let mut quote_state: Option<char> = None;
    let mut escaped = false;

    for ch in value.chars() {
        if let Some(q) = quote_state {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == q {
                quote_state = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote_state = Some(ch),
            '{' | '[' => stack.push(ch),
            '}' if stack.pop() != Some('{') => {
                return Err(AamlError::MalformedLiteral {
                    literal_type: "assignment".to_string(),
                    content: line.to_string(),
                    diagnostics: Some(Box::new(ErrorDiagnostics::new(
                        "Mismatched delimiters",
                        format!("Assignment '{line}' has an unmatched '}}'"),
                        "Ensure inline objects and lists use balanced braces/brackets",
                    ))),
                });
            }
            ']' if stack.pop() != Some('[') => {
                return Err(AamlError::MalformedLiteral {
                    literal_type: "assignment".to_string(),
                    content: line.to_string(),
                    diagnostics: Some(Box::new(ErrorDiagnostics::new(
                        "Mismatched delimiters",
                        format!("Assignment '{line}' has an unmatched ']'"),
                        "Ensure inline objects and lists use balanced braces/brackets",
                    ))),
                });
            }
            _ => {}
        }
    }

    if quote_state.is_some() {
        return Err(AamlError::MalformedLiteral {
            literal_type: "assignment".to_string(),
            content: line.to_string(),
            diagnostics: Some(Box::new(ErrorDiagnostics::new(
                "Unterminated quote",
                format!("Assignment '{line}' contains an unterminated quoted value"),
                "Close the opening quote in the assignment value",
            ))),
        });
    }

    if !stack.is_empty() {
        return Err(AamlError::MalformedLiteral {
            literal_type: "assignment".to_string(),
            content: line.to_string(),
            diagnostics: Some(Box::new(ErrorDiagnostics::new(
                "Unclosed delimiters",
                format!("Assignment '{line}' has unclosed '{{' or '['"),
                "Ensure inline objects and lists use balanced braces/brackets",
            ))),
        });
    }

    Ok(())
}

/// Strips a matching pair of surrounding `"…"` or `'…'` quotes from `s`.
///
/// Returns `s` unchanged (trimmed) if it is not quoted.
#[must_use]
pub fn unwrap_quotes(s: &str) -> &str {
    let s = s.trim();
    if s.len() >= 2 {
        if s.starts_with('"') && s.ends_with('"') {
            return &s[1..s.len() - 1];
        }
        if s.starts_with('\'') && s.ends_with('\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

/// Returns `true` when `text` is a directive that opens a `{` block that is
/// not yet closed on the same line — i.e. it needs multi-line accumulation.
pub(super) fn needs_accumulation(text: &str) -> bool {
    if !text.starts_with('@') {
        return false;
    }
    let opens = text.chars().filter(|&c| c == '{').count();
    let closes = text.chars().filter(|&c| c == '}').count();
    opens > closes
}

/// Returns `true` when the accumulated buffer has at least as many `}` as `{`.
pub(super) fn block_is_complete(buf: &str) -> bool {
    let opens = buf.chars().filter(|&c| c == '{').count();
    let closes = buf.chars().filter(|&c| c == '}').count();
    closes >= opens
}

/// Returns `true` when `value` is an inline object literal `{ ... }`.
#[must_use]
pub fn is_inline_object(value: &str) -> bool {
    let v = value.trim();
    v.starts_with('{') && v.ends_with('}')
}

/// Parses an inline object `{ key = val, key2 = val2, ... }` into `(key, value)` pairs.
///
/// Field separators are commas respecting `{}` / `[]` nesting, so values like
/// `{ base = { x = 1, y = 2 }, z = 3 }` are parsed correctly.
pub fn parse_inline_object(value: &str) -> Result<Vec<(String, String)>, AamlError> {
    let inner = value
        .trim()
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .ok_or_else(|| AamlError::MalformedLiteral {
            literal_type: "inline object".to_string(),
            content: value.to_string(),
            diagnostics: Some(Box::new(ErrorDiagnostics::new(
                "Malformed inline object",
                format!("Inline object must be wrapped in '{{}}', got: '{value}'"),
                "Wrap your object with curly braces: { key = value }",
            ))),
        })?;

    split_top_level_fields(inner)
        .into_iter()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|entry| {
            let (k, v) = split_field_pair(entry)?;
            let k = k.trim();

            if k.is_empty() {
                return Err(AamlError::InvalidValue {
                    details: format!("Empty key in field '{entry}'"),
                    expected: "non-empty field name".to_string(),
                    diagnostics: Some(Box::new(ErrorDiagnostics::new(
                        "Empty field name in inline object",
                        format!("Field entry '{entry}' has no valid key"),
                        "Provide a valid key name before '=' or ':'",
                    ))),
                });
            }

            let v = v.trim();
            let final_v = match v.chars().next() {
                Some('{' | '[') => v,
                _ => unwrap_quotes(v),
            };

            Ok((k.to_string(), final_v.to_string()))
        })
        .collect()
}

/// Splits `s` on commas that are not inside `{}`, `[]`, or quotes.
fn split_top_level_fields(s: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0;
    let mut in_quote: Option<char> = None;

    for (i, ch) in s.char_indices() {
        match ch {
            '"' | '\'' => match in_quote {
                Some(q) if q == ch => in_quote = None,
                None => in_quote = Some(ch),
                _ => {}
            },
            '{' | '[' if in_quote.is_none() => depth += 1,
            '}' | ']' if in_quote.is_none() => depth -= 1,
            ',' if depth == 0 && in_quote.is_none() => {
                items.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start <= s.len() {
        items.push(&s[start..]);
    }

    items
}

/// Splits `"key = val"` or `"key: val"` on the first `=` or `:` at depth 0.
fn split_field_pair(entry: &str) -> Result<(&str, &str), AamlError> {
    let mut depth: i32 = 0;
    for (i, ch) in entry.char_indices() {
        match ch {
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            '=' | ':' if depth == 0 => return Ok((&entry[..i], &entry[i + 1..])),
            _ => {}
        }
    }
    Err(AamlError::MalformedLiteral {
        literal_type: "field pair".to_string(),
        content: entry.to_string(),
        diagnostics: Some(Box::new(ErrorDiagnostics::new(
            "Missing field separator",
            format!("Field entry '{entry}' has no '=' or ':' separator"),
            "Use format: key = value or key: value",
        ))),
    })
}
