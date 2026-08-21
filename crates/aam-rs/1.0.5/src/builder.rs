//! Fluent builder for constructing AAML configuration content programmatically.
//!
//! [`AAMBuilder`] accumulates lines in memory and can either return them as a
//! `String` or write them directly to a file. Useful in tests and code generators.

use std::fmt::Display;
use std::io;
use std::ops::Deref;
use std::path::Path;

/// Accumulates AAML source lines and can flush them to a file or a `String`.
///
/// # Example
/// ```
/// use aam_rs::builder::AAMBuilder;
///
/// let mut b = AAMBuilder::new();
/// b.add_line("host", "localhost");
/// b.add_line("port", "8080");
/// let content = b.build();
/// assert!(content.contains("host = localhost"));
/// ```
pub struct AAMBuilder {
    buffer: String,
}

impl AAMBuilder {
    /// Creates a new empty builder.
    pub fn new() -> Self {
        Self { buffer: String::new() }
    }

    /// Creates a new builder with the given initial buffer capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { buffer: String::with_capacity(capacity) }
    }

    /// Appends a `key = value` assignment line.
    ///
    /// A newline separator is inserted automatically between entries.
    pub fn add_line(&mut self, key: &str, value: &str) {
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
        self.buffer.push_str(key);
        self.buffer.push_str(" = ");
        self.buffer.push_str(value);
    }

    /// Appends a raw line (e.g. a directive such as `@schema ...`).
    ///
    /// A newline separator is inserted automatically between entries.
    pub fn add_raw(&mut self, raw_line: &str) {
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
        self.buffer.push_str(raw_line);
    }

    /// Writes the accumulated content to the file at `path`.
    ///
    /// The file is created or truncated.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        std::fs::write(path, self.buffer.as_bytes())
    }

    /// Consumes the builder and returns the accumulated content as a `String`.
    pub fn build(self) -> String {
        self.buffer
    }

    /// Returns a clone of the accumulated content as a `String`.
    pub fn as_string(&self) -> String {
        self.buffer.clone()
    }
}

impl Deref for AAMBuilder {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl Default for AAMBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for AAMBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.buffer)
    }
}