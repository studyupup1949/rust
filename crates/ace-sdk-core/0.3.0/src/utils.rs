//! Utility functions: semver comparison and code extraction.

use regex::Regex;

// =============================================================================
// Semantic Versioning
// =============================================================================

/// Parsed semantic version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub build: Option<String>,
}

/// Parse a semantic version string.
///
/// Format: `MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]`
///
/// # Examples
/// ```
/// use ace_sdk_core::utils::parse_version;
///
/// let v = parse_version("3.6.2").unwrap();
/// assert_eq!(v.major, 3);
/// assert_eq!(v.minor, 6);
/// assert_eq!(v.patch, 2);
/// ```
pub fn parse_version(version: &str) -> Option<SemanticVersion> {
    let re = Regex::new(r"^(\d+)\.(\d+)\.(\d+)(?:-([^+]+))?(?:\+(.+))?$").ok()?;
    let caps = re.captures(version)?;

    Some(SemanticVersion {
        major: caps[1].parse().ok()?,
        minor: caps[2].parse().ok()?,
        patch: caps[3].parse().ok()?,
        prerelease: caps.get(4).map(|m| m.as_str().to_string()),
        build: caps.get(5).map(|m| m.as_str().to_string()),
    })
}

/// Compare two version strings.
///
/// Returns:
/// - `1` if a > b
/// - `0` if a == b
/// - `-1` if a < b
///
/// # Examples
/// ```
/// use ace_sdk_core::utils::compare_versions;
///
/// assert_eq!(compare_versions("2.0.0", "1.9.9"), 1);
/// assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
/// assert_eq!(compare_versions("1.0.0-beta", "1.0.0"), -1);
/// ```
pub fn compare_versions(a: &str, b: &str) -> i32 {
    let va = match parse_version(a) {
        Some(v) => v,
        None => return 0,
    };
    let vb = match parse_version(b) {
        Some(v) => v,
        None => return 0,
    };

    if va.major != vb.major {
        return if va.major > vb.major { 1 } else { -1 };
    }
    if va.minor != vb.minor {
        return if va.minor > vb.minor { 1 } else { -1 };
    }
    if va.patch != vb.patch {
        return if va.patch > vb.patch { 1 } else { -1 };
    }

    // Prerelease comparison (stable > prerelease)
    match (&va.prerelease, &vb.prerelease) {
        (Some(_), None) => -1,
        (None, Some(_)) => 1,
        (Some(a_pre), Some(b_pre)) => {
            let cmp = a_pre.cmp(b_pre);
            match cmp {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Greater => 1,
                std::cmp::Ordering::Equal => 0,
            }
        }
        (None, None) => 0,
    }
}

/// Check if current version satisfies a required version constraint.
///
/// Supports:
/// - `>=3.6.0` (greater than or equal)
/// - `^3.6.0` (compatible with, same major)
/// - `~3.6.0` (approximately equivalent, same major.minor)
/// - `3.6.0` (exact match)
pub fn satisfies_version(current: &str, required: &str) -> bool {
    if let Some(stripped) = required.strip_prefix(">=") {
        compare_versions(current, stripped) >= 0
    } else if let Some(stripped) = required.strip_prefix('^') {
        let base = match parse_version(stripped) {
            Some(v) => v,
            None => return false,
        };
        let curr = match parse_version(current) {
            Some(v) => v,
            None => return false,
        };
        curr.major == base.major && compare_versions(current, stripped) >= 0
    } else if let Some(stripped) = required.strip_prefix('~') {
        let base = match parse_version(stripped) {
            Some(v) => v,
            None => return false,
        };
        let curr = match parse_version(current) {
            Some(v) => v,
            None => return false,
        };
        curr.major == base.major
            && curr.minor == base.minor
            && compare_versions(current, stripped) >= 0
    } else {
        compare_versions(current, required) == 0
    }
}

// =============================================================================
// Code Extractor
// =============================================================================

/// Extracted code block from source.
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub code: String,
    pub name: String,
    pub block_type: CodeBlockType,
    pub file: String,
}

/// Type of extracted code block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeBlockType {
    Function,
    AsyncFunction,
    Method,
}

/// Check if code content is "interesting" enough to include in bootstrap.
///
/// Filters for: async/await, error handling, API calls, database usage.
pub fn is_interesting(code: &str) -> bool {
    code.contains("async ")
        || code.contains("await ")
        || code.contains("try {")
        || code.contains("catch (")
        || code.contains(".unwrap()")
        || code.contains("Result<")
        || code.contains("impl ")
        || code.contains("#[derive")
        || code.contains("pub fn ")
        || code.contains("pub async fn ")
}

/// Extract code blocks from markdown.
///
/// Finds triple-backtick fenced code blocks and returns substantial ones.
pub fn extract_code_blocks_from_markdown(markdown: &str) -> Vec<String> {
    let re = Regex::new(r"```(\w+)?\n([\s\S]*?)```").unwrap();
    let mut blocks = Vec::new();

    for caps in re.captures_iter(markdown) {
        if let Some(code) = caps.get(2) {
            let code_str = code.as_str().trim().to_string();
            if code_str.len() > 50 && is_interesting(&code_str) {
                blocks.push(code_str);
            }
        }
    }

    blocks
}

/// Extract complete function bodies from source code.
///
/// Detects:
/// - Regular functions (`fn name(...)` / `async fn name(...)`)
/// - `pub fn` / `pub async fn` variants
///
/// Returns a list of [`CodeBlock`] with the full function code.
pub fn extract_function_bodies(content: &str, file_path: &str) -> Vec<CodeBlock> {
    let mut blocks: Vec<CodeBlock> = Vec::new();

    // Match `[pub] [async] fn name(...) {`
    let fn_re =
        Regex::new(r"(?m)^[ \t]*(pub\s+)?(async\s+)?fn\s+(\w+)\s*\([^)]*\)[^{]*\{").unwrap();
    let name_re = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").unwrap();

    for m in fn_re.find_iter(content) {
        let start = m.start();
        // Find matching closing brace by counting
        if let Some(end) = find_matching_brace(content, m.end() - 1) {
            let code = &content[start..=end];
            // Extract function name from the regex
            if let Some(caps) = name_re.captures(code) {
                let name = caps.get(1).map(|m| m.as_str()).unwrap_or("unknown");
                let is_async = code.contains("async fn");

                blocks.push(CodeBlock {
                    code: code.to_string(),
                    name: name.to_string(),
                    block_type: if is_async {
                        CodeBlockType::AsyncFunction
                    } else {
                        CodeBlockType::Function
                    },
                    file: file_path.to_string(),
                });
            }
        }
    }

    blocks
}

/// Find the index of the matching closing brace for a '{' at `open_pos`.
fn find_matching_brace(content: &str, open_pos: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    if open_pos >= bytes.len() || bytes[open_pos] != b'{' {
        return None;
    }

    let mut depth = 1;
    let mut i = open_pos + 1;
    let mut in_string = false;
    let mut in_char = false;
    let mut escape = false;

    while i < bytes.len() && depth > 0 {
        let ch = bytes[i];

        if escape {
            escape = false;
            i += 1;
            continue;
        }

        if ch == b'\\' && (in_string || in_char) {
            escape = true;
            i += 1;
            continue;
        }

        if ch == b'"' && !in_char {
            in_string = !in_string;
        } else if ch == b'\'' && !in_string {
            in_char = !in_char;
        } else if !in_string && !in_char {
            if ch == b'{' {
                depth += 1;
            } else if ch == b'}' {
                depth -= 1;
            }
        }

        if depth == 0 {
            return Some(i);
        }

        i += 1;
    }

    None
}

/// Extract added lines from a git diff.
///
/// Filters for lines starting with '+' (added lines).
pub fn extract_added_lines_from_diff(diff: &str) -> String {
    diff.lines()
        .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
        .map(|line| &line[1..])
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let v = parse_version("3.6.2").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 6);
        assert_eq!(v.patch, 2);
        assert_eq!(v.prerelease, None);

        let v = parse_version("1.0.0-beta.1").unwrap();
        assert_eq!(v.prerelease, Some("beta.1".to_string()));

        let v = parse_version("1.0.0+build123").unwrap();
        assert_eq!(v.build, Some("build123".to_string()));
    }

    #[test]
    fn test_compare_versions() {
        assert_eq!(compare_versions("2.0.0", "1.9.9"), 1);
        assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
        assert_eq!(compare_versions("0.9.0", "1.0.0"), -1);
        assert_eq!(compare_versions("1.0.0-alpha", "1.0.0"), -1);
        assert_eq!(compare_versions("1.0.0", "1.0.0-beta"), 1);
    }

    #[test]
    fn test_satisfies_version() {
        assert!(satisfies_version("3.7.0", ">=3.6.0"));
        assert!(!satisfies_version("3.5.0", ">=3.6.0"));
        assert!(satisfies_version("3.9.0", "^3.6.0"));
        assert!(!satisfies_version("4.0.0", "^3.6.0"));
        assert!(satisfies_version("3.6.5", "~3.6.0"));
        assert!(!satisfies_version("3.7.0", "~3.6.0"));
        assert!(satisfies_version("1.0.0", "1.0.0"));
        assert!(!satisfies_version("1.0.1", "1.0.0"));
    }

    #[test]
    fn test_is_interesting() {
        assert!(is_interesting(
            "pub async fn handle() -> Result<(), Error> {}"
        ));
        assert!(is_interesting("impl MyStruct {"));
        assert!(!is_interesting("let x = 5;"));
    }

    #[test]
    fn test_extract_code_blocks_from_markdown() {
        let md = r#"
# Example

```rust
pub async fn fetch_data() -> Result<Vec<u8>, Error> {
    let client = reqwest::Client::new();
    client.get("https://api.example.com").send().await?.bytes().await
}
```

```text
not interesting plain text
```
"#;
        let blocks = extract_code_blocks_from_markdown(md);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].contains("async fn fetch_data"));
    }

    #[test]
    fn test_extract_added_lines_from_diff() {
        let diff = "+++ b/src/main.rs\n+fn new_function() {\n+    println!(\"hello\");\n+}\n-old line\n context line\n";
        let result = extract_added_lines_from_diff(diff);
        assert!(result.contains("fn new_function()"));
        assert!(!result.contains("old line"));
    }

    // =========================================================================
    // extract_function_bodies tests
    // =========================================================================

    #[test]
    fn test_extract_function_bodies_basic() {
        let code = r#"
fn hello() {
    println!("hello");
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
        let blocks = extract_function_bodies(code, "test.rs");
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].name, "hello");
        assert_eq!(blocks[0].block_type, CodeBlockType::Function);
        assert_eq!(blocks[1].name, "add");
    }

    #[test]
    fn test_extract_function_bodies_async() {
        let code = r#"
pub async fn fetch_data() -> Result<(), Error> {
    let resp = client.get("url").send().await?;
    Ok(())
}
"#;
        let blocks = extract_function_bodies(code, "http.rs");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "fetch_data");
        assert_eq!(blocks[0].block_type, CodeBlockType::AsyncFunction);
        assert_eq!(blocks[0].file, "http.rs");
    }

    #[test]
    fn test_extract_function_bodies_nested_braces() {
        let code = r#"
fn complex() {
    if true {
        for i in 0..10 {
            println!("{}", i);
        }
    }
}
"#;
        let blocks = extract_function_bodies(code, "test.rs");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "complex");
        assert!(blocks[0].code.contains("for i in 0..10"));
    }

    #[test]
    fn test_extract_function_bodies_empty() {
        let code = "// no functions here\nlet x = 5;\n";
        let blocks = extract_function_bodies(code, "test.rs");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_find_matching_brace() {
        assert_eq!(find_matching_brace("{ }", 0), Some(2));
        assert_eq!(find_matching_brace("{ { } }", 0), Some(6));
        assert_eq!(find_matching_brace("{ \"}\", }", 0), Some(7));
        assert!(find_matching_brace("{", 0).is_none());
    }
}
