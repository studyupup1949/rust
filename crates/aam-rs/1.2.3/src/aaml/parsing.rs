//! Parsing helpers: comment stripping, assignment parsing, multi-line block accumulation.

/// Strips an inline `#` comment from a raw source line, respecting quoted strings.
///
/// A `#` is a comment start only when it is preceded by whitespace (or at line start),
/// so unquoted color values like `tint = #ff6600` are stored correctly.
pub fn strip_comment(line: &str) -> &str {
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
/// The split point is the **first `=`** that appears outside of any
/// `{ ... }` or `[ ... ]` nesting.  This allows values like
/// `pos = { x = 1.0, y = 2.0 }` or `tags = [a, b, c]` to be parsed
/// correctly.  Surrounding quotes are stripped from the value via
/// [`unwrap_quotes`], but `{...}` and `[...]` literals are returned as-is.
pub(super) fn parse_assignment(line: &str) -> Result<(&str, &str), &'static str> {
    // Find the first '=' outside of nesting
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

    let pos = eq_pos.ok_or("Missing assignment operator '='")?;
    let key = line[..pos].trim();
    let raw_val = line[pos + 1..].trim();

    if key.is_empty() {
        return Err("Key cannot be empty");
    }

    // Do NOT unwrap quotes when the value is an inline object or list literal
    let val = if raw_val.starts_with('{') || raw_val.starts_with('[') {
        raw_val
    } else {
        unwrap_quotes(raw_val)
    };

    Ok((key, val))
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

/// Returns `true` when `value` is an inline object literal `{ ... }`.
pub fn is_inline_object(value: &str) -> bool {
    let v = value.trim();
    v.starts_with('{') && v.ends_with('}')
}

/// Parses an inline object `{ key = val, key2 = val2, ... }` into `(key, value)` pairs.
///
/// Field separators are commas respecting `{}` / `[]` nesting, so values like
/// `{ base = { x = 1, y = 2 }, z = 3 }` are parsed correctly.
// Medium Complexity 
pub fn parse_inline_object(value: &str) -> Result<Vec<(String, String)>, String> {
    let s = value.trim();
    let inner = s
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .ok_or_else(|| format!("Inline object must be wrapped in '{{}}', got: '{value}'"))?;

    let mut fields = Vec::new();
    for entry in split_top_level_fields(inner) {
        let entry = entry.trim().to_string();
        if entry.is_empty() { continue; }

        let (k, v) = split_field_pair(&entry)?;
        let k = k.trim();
        let v = if v.trim().starts_with('{') || v.trim().starts_with('[') {
            v.trim()
        } else {
            unwrap_quotes(v.trim())
        };
        if k.is_empty() {
            return Err(format!("Empty key in inline object field '{entry}'"));
        }
        fields.push((k.to_string(), v.to_string()));
    }
    Ok(fields)
}

/// Splits `s` on commas that are not inside `{}` or `[]` nesting.
fn split_top_level_fields(s: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut depth: i32 = 0;
    let mut cur = String::new();
    for ch in s.chars() {
        match ch {
            '{' | '[' => { depth += 1; cur.push(ch); }
            '}' | ']' => { depth -= 1; cur.push(ch); }
            ',' if depth == 0 => {
                items.push(cur.clone());
                cur.clear();
            }
            _ => { cur.push(ch); }
        }
    }
    items.push(cur);
    items
}

/// Splits `"key = val"` or `"key: val"` on the first `=` or `:` at depth 0.
fn split_field_pair(entry: &str) -> Result<(&str, &str), String> {
    let mut depth: i32 = 0;
    for (i, ch) in entry.char_indices() {
        match ch {
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            '=' | ':' if depth == 0 => return Ok((&entry[..i], &entry[i + 1..])),
            _ => {}
        }
    }
    Err(format!("Inline object field '{entry}' has no '=' or ':' separator"))
}