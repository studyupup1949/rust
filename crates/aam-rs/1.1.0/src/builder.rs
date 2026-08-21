//! Fluent builder for constructing AAML configuration content programmatically.
//!
//! [`AAMBuilder`] accumulates lines in memory and can either return them as a
//! `String` or write them directly to a file. Useful in tests and code generators.
//!
//! # High-level directive API
//!
//! Instead of calling [`AAMBuilder::add_raw`] manually, use the dedicated methods:
//!
//! | Method | Directive emitted |
//! |---|---|
//! | [`AAMBuilder::schema`] | `@schema Name { ... }` |
//! | [`AAMBuilder::derive`] | `@derive file.aam` / `@derive file.aam::A::B` |
//! | [`AAMBuilder::import`] | `@import file.aam` |
//! | [`AAMBuilder::type_alias`] | `@type alias = type` |
//! | [`AAMBuilder::comment`] | `# ...` |
//!
//! # Example
//! ```
//! use aam_rs::builder::{AAMBuilder, SchemaField};
//!
//! let mut b = AAMBuilder::new();
//! b.comment("Server configuration")
//!  .type_alias("port_t", "i32")
//!  .schema("Server", [
//!      SchemaField::required("host", "string"),
//!      SchemaField::required("port", "port_t"),
//!      SchemaField::optional("debug", "bool"),
//!  ])
//!  .add_line("host", "localhost")
//!  .add_line("port", "8080");
//!
//! let content = b.build();
//! assert!(content.contains("@schema Server {"));
//! assert!(content.contains("host = localhost"));
//! ```

use std::fmt::Display;
use std::io;
use std::ops::Deref;
use std::path::Path;

/// A single field declaration inside a `@schema` block.
///
/// Fields declared with [`SchemaField::optional`] are emitted with a `*` suffix,
/// meaning the key does not have to be present in the data map, but its value is
/// still type-checked when it *is* present.
///
/// # Example
/// ```
/// use aam_rs::builder::SchemaField;
///
/// let f = SchemaField::required("host", "string");
/// assert_eq!(f.to_aaml(), "host: string");
///
/// let g = SchemaField::optional("debug", "bool");
/// assert_eq!(g.to_aaml(), "debug*: bool");
/// ```
#[derive(Debug, Clone)]
pub struct SchemaField {
    name: String,
    type_name: String,
    optional: bool,
}

impl SchemaField {
    /// Creates a **required** field (rendered as `name: type`).
    pub fn required(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self { name: name.into(), type_name: type_name.into(), optional: false }
    }

    /// Creates an **optional** field (rendered as `name*: type`).
    pub fn optional(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self { name: name.into(), type_name: type_name.into(), optional: true }
    }

    /// Renders the field as an AAML field declaration string.
    pub fn to_aaml(&self) -> String {
        if self.optional {
            format!("{}*: {}", self.name, self.type_name)
        } else {
            format!("{}: {}", self.name, self.type_name)
        }
    }
}

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

    // ── Internal helpers ─────────────────────────────────────────────────────

    fn push_sep(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
    }

    // ── Key-value assignments ─────────────────────────────────────────────────

    /// Appends a `key = value` assignment line.
    ///
    /// A newline separator is inserted automatically between entries.
    /// Returns `&mut self` for chaining.
    pub fn add_line(&mut self, key: &str, value: &str) -> &mut Self {
        self.push_sep();
        self.buffer.push_str(key);
        self.buffer.push_str(" = ");
        self.buffer.push_str(value);
        self
    }

    // ── Comments ──────────────────────────────────────────────────────────────

    /// Appends a `# text` comment line.
    ///
    /// Returns `&mut self` for chaining.
    pub fn comment(&mut self, text: &str) -> &mut Self {
        self.push_sep();
        self.buffer.push_str("# ");
        self.buffer.push_str(text);
        self
    }

    // ── Directives ────────────────────────────────────────────────────────────

    /// Appends a `@schema Name { field1: type1, field2*: type2, ... }` directive.
    ///
    /// Use [`SchemaField::required`] and [`SchemaField::optional`] to build the
    /// field list.
    ///
    /// # Example
    /// ```
    /// use aam_rs::builder::{AAMBuilder, SchemaField};
    ///
    /// let mut b = AAMBuilder::new();
    /// b.schema("Point", [
    ///     SchemaField::required("x", "f64"),
    ///     SchemaField::required("y", "f64"),
    ///     SchemaField::optional("z", "f64"),
    /// ]);
    /// assert!(b.build().contains("@schema Point {"));
    /// ```
    pub fn schema(
        &mut self,
        name: &str,
        fields: impl IntoIterator<Item = SchemaField>,
    ) -> &mut Self {
        let fields_str: Vec<String> = fields.into_iter().map(|f| f.to_aaml()).collect();
        self.push_sep();
        self.buffer.push_str("@schema ");
        self.buffer.push_str(name);
        self.buffer.push_str(" { ");
        self.buffer.push_str(&fields_str.join(", "));
        self.buffer.push_str(" }");
        self
    }

    /// Appends a `@schema Name { ... }` directive using a multiline block format.
    ///
    /// Each field is placed on its own indented line, which is more readable for
    /// schemas with many fields.
    ///
    /// # Example
    /// ```
    /// use aam_rs::builder::{AAMBuilder, SchemaField};
    ///
    /// let mut b = AAMBuilder::new();
    /// b.schema_multiline("Server", [
    ///     SchemaField::required("host", "string"),
    ///     SchemaField::optional("port", "i32"),
    /// ]);
    /// let out = b.build();
    /// assert!(out.contains("    host: string"));
    /// ```
    pub fn schema_multiline(
        &mut self,
        name: &str,
        fields: impl IntoIterator<Item = SchemaField>,
    ) -> &mut Self {
        self.push_sep();
        self.buffer.push_str("@schema ");
        self.buffer.push_str(name);
        self.buffer.push_str(" {");
        for field in fields {
            self.buffer.push('\n');
            self.buffer.push_str("    ");
            self.buffer.push_str(&field.to_aaml());
        }
        self.buffer.push('\n');
        self.buffer.push('}');
        self
    }

    /// Appends a `@derive path` or `@derive path::Schema1::Schema2` directive.
    ///
    /// Pass `schemas` as an empty iterator (e.g. `[]`) to derive the entire file.
    ///
    /// # Example
    /// ```
    /// use aam_rs::builder::AAMBuilder;
    ///
    /// let mut b = AAMBuilder::new();
    /// b.derive("base.aam", ["Server", "Database"]);
    /// assert!(b.build().contains("@derive base.aam::Server::Database"));
    ///
    /// let mut b2 = AAMBuilder::new();
    /// b2.derive("base.aam", [] as [&str; 0]);
    /// assert!(b2.build().contains("@derive base.aam"));
    /// ```
    pub fn derive(
        &mut self,
        path: &str,
        schemas: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> &mut Self {
        self.push_sep();
        self.buffer.push_str("@derive ");
        self.buffer.push_str(path);
        for schema in schemas {
            self.buffer.push_str("::");
            self.buffer.push_str(schema.as_ref());
        }
        self
    }

    /// Appends a `@import path` directive.
    ///
    /// # Example
    /// ```
    /// use aam_rs::builder::AAMBuilder;
    ///
    /// let mut b = AAMBuilder::new();
    /// b.import("shared.aam");
    /// assert!(b.build().contains("@import shared.aam"));
    /// ```
    pub fn import(&mut self, path: &str) -> &mut Self {
        self.push_sep();
        self.buffer.push_str("@import ");
        self.buffer.push_str(path);
        self
    }

    /// Appends a `@type alias = type_name` directive.
    ///
    /// # Example
    /// ```
    /// use aam_rs::builder::AAMBuilder;
    ///
    /// let mut b = AAMBuilder::new();
    /// b.type_alias("pos", "math::vector3");
    /// assert!(b.build().contains("@type pos = math::vector3"));
    /// ```
    pub fn type_alias(&mut self, alias: &str, type_name: &str) -> &mut Self {
        self.push_sep();
        self.buffer.push_str("@type ");
        self.buffer.push_str(alias);
        self.buffer.push_str(" = ");
        self.buffer.push_str(type_name);
        self
    }

    /// Appends a raw line as-is (e.g. a directive not covered by the typed API).
    ///
    /// A newline separator is inserted automatically between entries.
    ///
    /// > **Note:** Prefer the typed directive methods ([`schema`](Self::schema),
    /// > [`derive`](Self::derive), [`import`](Self::import),
    /// > [`type_alias`](Self::type_alias)) over this method when possible.
    ///
    /// Returns `&mut self` for chaining.
    #[deprecated(since="1.1.0", note="Prefer the typed directive methods (schema, derive, import, type_alias) over this method when possible.")]
    pub fn add_raw(&mut self, raw_line: &str) -> &mut Self {
        self.push_sep();
        self.buffer.push_str(raw_line);
        self
    }

    // ── Output ────────────────────────────────────────────────────────────────

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

