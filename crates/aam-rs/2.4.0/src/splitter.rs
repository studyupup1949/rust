use std::collections::HashMap;

/// Parses a multi-section AAML text and returns a `HashMap` mapping each file name
/// (extracted from `# filename.aam` headers) to an [`AAMBuilder`] containing the
/// section's content.
///
/// Each non-empty line inside a section is expected to be an assignment `key = value`
/// and is added via [`AAMBuilder::add_line`]. Lines that do not follow this format,
/// including comments and blank lines, are silently ignored.
///
/// # Example
/// ```
/// # use std::collections::HashMap;
/// # use aam_rs::builder::AAMBuilder;
/// # use aam_rs::breaker::split_aam;
/// let input = "# kvantum.aam\na = b\nx = d\n# another.aam\nc = d\ng = f\n";
/// let map = split_aam(input);
/// assert_eq!(map.len(), 2);
/// assert_eq!(map["kvantum.aam"].as_string(), "a = b\nx = d");
/// assert_eq!(map["another.aam"].as_string(), "c = d\ng = f");
/// ```
pub fn split_aam(input: &str) -> HashMap<String, AAMBuilder> {
    let mut result = HashMap::new();
    let mut current_name: Option<String> = None;
    let mut current_builder = AAMBuilder::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(filename) = parse_section_header(trimmed) {
            if let Some(prev_name) = current_name.take() {
                let old_builder = std::mem::replace(&mut current_builder, AAMBuilder::new());
                result.insert(prev_name, old_builder);
            }
            current_name = Some(filename.to_owned());
            continue;
        }

        if current_name.is_some() {
            if let Some((key, value)) = parse_assignment(trimmed) {
                current_builder.add_line(key, value);
            }
        }
    }

    if let Some(prev_name) = current_name {
        result.insert(prev_name, current_builder);
    }

    result
}

/// Checks if a line is a section header (`# filename.aam`) and returns the file name.
fn parse_section_header(s: &str) -> Option<&str> {
    if s.starts_with('#') {
        let rest = s[1..].trim();
        if rest.ends_with(".aam") {
            return Some(rest);
        }
    }
    None
}

/// Splits a string `key = value` into a key-value pair, trimming whitespace.
fn parse_assignment(s: &str) -> Option<(&str, &str)> {
    let mut parts = s.splitn(2, '=');
    let key = parts.next()?.trim();
    let value = parts.next().map(|v| v.trim())?;
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}
