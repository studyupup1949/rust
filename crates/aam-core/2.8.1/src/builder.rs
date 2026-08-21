//! Fluent builder for constructing AAML configuration content programmatically.
//!
//! [`AAMBuilder`] accumulates lines in memory and can either return them as a
//! `String` or write them directly to a file. Useful in tests and code generators.
//!

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

//! # High-level directive API
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
//! use aam_core::builder::{AAMBuilder, SchemaField};
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
use std::fmt::Write;
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
/// use aam_core::builder::SchemaField;
///
/// let f = SchemaField::required("host", "string");
/// assert_eq!(f.to_aaml(), "host: string");
///
/// let g = SchemaField::optional("debug", "bool");
/// assert_eq!(g.to_aaml(), "debug*: bool");
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaField {
    name: String,
    type_name: String,
    optional: bool,
}

impl SchemaField {
    /// Creates a **required** field (rendered as `name: type`).
    pub fn required(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            optional: false,
        }
    }

    /// Creates an **optional** field (rendered as `name*: type`).
    pub fn optional(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            optional: true,
        }
    }

    pub fn to_aaml_writer(&self, mut w: impl Write) -> std::fmt::Result {
        write!(
            w,
            "{}{}: {}",
            self.name,
            if self.optional { "*" } else { "" },
            self.type_name
        )
    }

    /// Renders the field as an AAML field declaration string.
    #[must_use]
    pub fn to_aaml(&self) -> String {
        let mut s = String::new();
        self.to_aaml_writer(&mut s).unwrap();
        s
    }
}

/// Enumeration of all built-in AAML types for use with the Builder API.
///
/// Can be passed directly to [`AAMBuilder::type_alias`] and similar methods.
///
/// Only available with the `builder-extras` feature (enabled by default).
///
/// # Example
/// ```
/// use aam_core::builder::BuiltInType;
///
/// let t = BuiltInType::I32;
/// assert_eq!(t.to_string(), "i32");
///
/// let v = BuiltInType::List(Box::new(BuiltInType::String));
/// assert_eq!(v.to_string(), "list<string>");
/// ```
#[cfg(feature = "builder-extras")]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BuiltInType {
    // ── Primitives ──────────────────────────────────────────────────────────────
    I32,
    F64,
    String,
    Bool,
    Color,

    // ── Math types ──────────────────────────────────────────────────────────────
    Vector2,
    Vector3,
    Vector4,
    Quaternion,
    Matrix3x3,
    Matrix4x4,

    // ── Time types ──────────────────────────────────────────────────────────────
    DateTime,
    Duration,
    Year,
    Day,
    Hour,
    Minute,

    // ── Physics types (common) ──────────────────────────────────────────────────
    Kilogram,
    Meter,

    // ── Special ─────────────────────────────────────────────────────────────────
    /// Generic inline object type (`schema`).
    Schema,

    /// Any other type specified as a raw string (e.g. `"physics::newton"`).
    Custom(String),

    /// List of another type (e.g. `list<f64>`).
    List(Box<Self>),
}

#[cfg(feature = "builder-extras")]
impl Display for BuiltInType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I32 => write!(f, "i32"),
            Self::F64 => write!(f, "f64"),
            Self::String => write!(f, "string"),
            Self::Bool => write!(f, "bool"),
            Self::Color => write!(f, "color"),
            Self::Vector2 => write!(f, "math::vector2"),
            Self::Vector3 => write!(f, "math::vector3"),
            Self::Vector4 => write!(f, "math::vector4"),
            Self::Quaternion => write!(f, "math::quaternion"),
            Self::Matrix3x3 => write!(f, "math::matrix3x3"),
            Self::Matrix4x4 => write!(f, "math::matrix4x4"),
            Self::DateTime => write!(f, "time::datetime"),
            Self::Duration => write!(f, "time::duration"),
            Self::Year => write!(f, "time::year"),
            Self::Day => write!(f, "time::day"),
            Self::Hour => write!(f, "time::hour"),
            Self::Minute => write!(f, "time::minute"),
            Self::Kilogram => write!(f, "physics::kilogram"),
            Self::Meter => write!(f, "physics::meter"),
            Self::Schema => write!(f, "schema"),
            Self::Custom(s) => write!(f, "{s}"),
            Self::List(inner) => write!(f, "list<{inner}>"),
        }
    }
}

#[cfg(feature = "builder-extras")]
impl From<&str> for BuiltInType {
    fn from(s: &str) -> Self {
        match s {
            "i32" => Self::I32,
            "f64" => Self::F64,
            "string" => Self::String,
            "bool" => Self::Bool,
            "color" => Self::Color,
            "math::vector2" => Self::Vector2,
            "math::vector3" => Self::Vector3,
            "math::vector4" => Self::Vector4,
            "math::quaternion" => Self::Quaternion,
            "math::matrix3x3" => Self::Matrix3x3,
            "math::matrix4x4" => Self::Matrix4x4,
            "time::datetime" => Self::DateTime,
            "time::duration" => Self::Duration,
            "time::year" => Self::Year,
            "time::day" => Self::Day,
            "time::hour" => Self::Hour,
            "time::minute" => Self::Minute,
            "physics::kilogram" => Self::Kilogram,
            "physics::meter" => Self::Meter,
            "schema" => Self::Schema,
            _ => s
                .strip_prefix("list<")
                .and_then(|s| s.strip_suffix('>'))
                .map_or_else(
                    || Self::Custom(s.to_string()),
                    |inner| Self::List(Box::new(Self::from(inner))),
                ),
        }
    }
}

/// A builder for inline object literals `{ key = value, ... }`.
///
/// Only available with the `builder-extras` feature (enabled by default).
///
/// # Example
/// ```
/// use aam_core::builder::InlineObject;
///
/// let obj = InlineObject::new()
///     .with_field("system", "cmake")
///     .with_field("command", "cmake")
///     .with_field("args", "[\"-G\", \"Ninja\"]");
///
/// assert_eq!(
///     obj.to_string(),
///     r#"{ system = cmake, command = cmake, args = ["-G", "Ninja"] }"#
/// );
/// ```
#[cfg(feature = "builder-extras")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InlineObject {
    fields: Vec<(String, String)>,
}

#[cfg(feature = "builder-extras")]
impl InlineObject {
    /// Creates a new empty inline object.
    #[must_use]
    pub const fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Adds a field to the inline object (builder-pattern, consumes self).
    #[must_use]
    pub fn with_field(mut self, key: &str, value: &str) -> Self {
        self.fields.push((key.to_string(), value.to_string()));
        self
    }

    /// Adds a field by mutable reference.
    pub fn add_field(&mut self, key: &str, value: &str) -> &mut Self {
        self.fields.push((key.to_string(), value.to_string()));
        self
    }

    /// Returns the fields as a slice.
    #[must_use]
    pub fn fields(&self) -> &[(String, String)] {
        &self.fields
    }

    /// Renders the object as an AAML inline object string.
    #[must_use]
    pub fn to_aaml(&self) -> String {
        self.to_string()
    }
}

#[cfg(feature = "builder-extras")]
impl Default for InlineObject {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "builder-extras")]
impl Display for InlineObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ ")?;
        for (i, (k, v)) in self.fields.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{k} = {v}")?;
        }
        write!(f, " }}")
    }
}

#[cfg(feature = "builder-extras")]
impl From<Vec<(String, String)>> for InlineObject {
    fn from(fields: Vec<(String, String)>) -> Self {
        Self { fields }
    }
}

#[cfg(feature = "builder-extras")]
impl From<InlineObject> for String {
    fn from(obj: InlineObject) -> Self {
        obj.to_string()
    }
}

/// Parses an inline object string `{ key = value, ... }` into a `HashMap<String, String>`.
///
/// Nested structures (e.g. `[a, b]` inside values) are preserved as raw strings.
///
/// # Feature
/// Enabled by default under the `builder-extras` feature.
///
/// # Example
/// ```
/// use aam_core::builder::parse_inline_to_map;
///
/// let map = parse_inline_to_map(r#"{ system = cmake, args = ["-G", "Ninja"] }"#).unwrap();
/// assert_eq!(map.get("system").unwrap(), "cmake");
/// assert_eq!(map.get("args").unwrap(), r#"["-G", "Ninja"]"#);
/// ```
#[cfg(feature = "builder-extras")]
pub fn parse_inline_to_map(
    s: &str,
) -> Result<std::collections::HashMap<String, String>, crate::error::AamlError> {
    let pairs = crate::aaml::parsing::parse_inline_object(s)?;
    Ok(pairs.into_iter().collect())
}

/// Accumulates AAML source lines and can flush them to a file or a `String`.
///
/// # Example
/// ```
/// use aam_core::builder::AAMBuilder;
///
/// let mut b = AAMBuilder::new();
/// b.add_line("host", "localhost");
/// b.add_line("port", "8080");
/// let content = b.build();
/// assert!(content.contains("host = localhost"));
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AAMBuilder {
    buffer: String,
}

impl AAMBuilder {
    /// Creates a new empty builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Creates a new builder with the given initial buffer capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: String::with_capacity(capacity),
        }
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
    /// use aam_core::builder::{AAMBuilder, SchemaField};
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
        self.push_sep();
        self.buffer.push_str("@schema ");
        self.buffer.push_str(name);
        self.buffer.push_str(" { ");

        let mut first = true;
        for field in fields {
            if !first {
                self.buffer.push_str(", ");
            }
            field.to_aaml_writer(&mut self.buffer).unwrap();
            first = false;
        }

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
    /// use aam_core::builder::{AAMBuilder, SchemaField};
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
        write!(&mut self.buffer, "@schema {name} {{").unwrap();
        for field in fields {
            write!(&mut self.buffer, "\n    ").unwrap();
            field.to_aaml_writer(&mut self.buffer).unwrap();
        }
        self.buffer.push_str("\n}");
        self
    }

    /// Appends a `@derive path` or `@derive path::Schema1::Schema2` directive.
    ///
    /// Pass `schemas` as an empty iterator (e.g. `[]`) to derive the entire file.
    ///
    /// # Example
    /// ```
    /// use aam_core::builder::AAMBuilder;
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
    /// use aam_core::builder::AAMBuilder;
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
    /// use aam_core::builder::AAMBuilder;
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

    /// Appends a `@type alias = builtin_type` directive using a [`BuiltInType`] enum.
    ///
    /// Only available with the `builder-extras` feature (enabled by default).
    ///
    /// # Example
    /// ```
    /// use aam_core::builder::{AAMBuilder, BuiltInType};
    ///
    /// let mut b = AAMBuilder::new();
    /// b.type_alias_builtin("pos", &BuiltInType::Vector3);
    /// assert!(b.build().contains("@type pos = math::vector3"));
    /// ```
    #[cfg(feature = "builder-extras")]
    pub fn type_alias_builtin(&mut self, alias: &str, builtin: &BuiltInType) -> &mut Self {
        self.type_alias(alias, &builtin.to_string())
    }

    /// Appends a `key = value` line where the value is an inline object.
    ///
    /// Only available with the `builder-extras` feature (enabled by default).
    ///
    /// # Example
    /// ```
    /// use aam_core::builder::{AAMBuilder, InlineObject};
    ///
    /// let mut b = AAMBuilder::new();
    /// let obj = InlineObject::new()
    ///     .with_field("system", "cmake")
    ///     .with_field("command", "cmake");
    /// b.add_inline("build", &obj);
    /// assert!(b.build().contains("build = { system = cmake, command = cmake }"));
    /// ```
    #[cfg(feature = "builder-extras")]
    pub fn add_inline(&mut self, key: &str, obj: &InlineObject) -> &mut Self {
        self.add_line(key, &obj.to_string())
    }

    /// Appends a `key = value` line where the value is parsed from an inline object string.
    ///
    /// Only available with the `builder-extras` feature (enabled by default).
    ///
    /// # Example
    /// ```
    /// use aam_core::builder::AAMBuilder;
    ///
    /// let mut b = AAMBuilder::new();
    /// b.add_inline_str("build", r#"{ system = cmake, command = cmake }"#);
    /// assert!(b.build().contains("build = { system = cmake, command = cmake }"));
    /// ```
    #[cfg(feature = "builder-extras")]
    pub fn add_inline_str(&mut self, key: &str, inline_str: &str) -> &mut Self {
        self.add_line(key, inline_str)
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
    #[deprecated(
        since = "1.1.0",
        note = "Prefer the typed directive methods (schema, derive, import, type_alias) over this method when possible."
    )]
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
    #[must_use]
    pub fn build(self) -> String {
        self.buffer
    }

    /// Returns a clone of the accumulated content as a `String`.
    #[must_use]
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
