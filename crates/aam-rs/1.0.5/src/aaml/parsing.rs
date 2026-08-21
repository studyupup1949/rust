//! Parsing helpers: comment stripping, assignment parsing, multi-line block accumulation.

/// Strips an inline `#` comment from a raw source line, respecting quoted strings.
///
/// A `#` is a comment start only when it is preceded by whitespace (or at line start),
/// so unquoted color values like `tint = #ff6600` are stored correctly.
pub(super) fn strip_comment(line: &str) -> &str {
    let mut quote_state: Option<char> = None;
    let bytes = line.as_bytes();

    for (idx, c) in line.char_indices() {
        match (quote_state, c) {
            (None, '#') => {
                let preceded_by_space = idx == 0
                    || bytes.get(idx - 1).is_some_and(|b| b.is_ascii_whitespace());
                let followed_by_space = bytes
                    .get(idx + 1)
                    .is_none_or(|b| b.is_ascii_whitespace());
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
/// Surrounding quotes are stripped from the value via [`unwrap_quotes`].
pub(super) fn parse_assignment(line: &str) -> Result<(&str, &str), &'static str> {
    let (key, val) = line.split_once('=').ok_or("Missing assignment operator '='")?;
    let key = key.trim();
    if key.is_empty() {
        return Err("Key cannot be empty");
    }
    Ok((key, unwrap_quotes(val)))
}

/// Strips a matching pair of surrounding `"…"` or `'…'` quotes from `s`.
///
/// Returns `s` unchanged (trimmed) if it is not quoted.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_not_stripped() {
        assert_eq!(strip_comment("tint = #ff6600"), "tint = #ff6600");
        assert_eq!(strip_comment("tint=#ff6600"), "tint=#ff6600");
    }

    #[test]
    fn comment_after_space_stripped() {
        assert_eq!(strip_comment("key = value # comment").trim(), "key = value");
    }

    #[test]
    fn quoted_hash_preserved() {
        assert_eq!(strip_comment(r#"key = "val # not comment""#), r#"key = "val # not comment""#);
    }
}

