//! Core AAML parser and runtime.
//!
//! [`AAML`] is the main entry point for parsing `.aam` configuration files.
//! It supports:
//! - Key-value assignments (`key = value`)
//! - Directives: `@import`, `@derive`, `@schema`, `@type`
//! - Runtime type validation via registered or built-in types
//! - Schema-based struct validation with [`AAML::apply_schema`]

use crate::commands::{self, Command};
use crate::error::AamlError;
use crate::commands::schema::SchemaDef;
use crate::types::Type;
use std::collections::HashMap;
use std::fs;
use std::ops::{Add, AddAssign};
use std::path::Path;
use std::sync::Arc;

mod lookup;
mod validation;
pub mod parsing;
pub mod types_registry;
#[cfg(feature = "serde")]
pub mod serialize;

#[cfg(feature = "perf-hash")]
type Hasher = ahash::RandomState;

#[cfg(not(feature = "perf-hash"))]
type Hasher = std::collections::hash_map::RandomState;

type AamlString = Box<str>;

/// The main AAML parser and configuration store.
///
/// Holds a flat key-value map, registered type definitions, command handlers,
/// and schema definitions. All directives (`@derive`, `@schema`, etc.) are
/// processed at parse time.
///
/// # Example
/// ```no_run
/// use aam_rs::aaml::AAML;
///
/// let cfg = AAML::parse("host = localhost\nport = 8080").unwrap();
/// assert_eq!(cfg.find_obj("host").unwrap().as_str(), "localhost");
/// ```
pub struct AAML {
    map: HashMap<AamlString, AamlString, Hasher>,
    commands: HashMap<String, Arc<dyn Command>>,
    types: HashMap<String, Box<dyn Type>>,
    schemas: HashMap<String, SchemaDef>,
}

impl std::fmt::Debug for AAML {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AAML")
            .field("map", &self.map)
            .field("commands_count", &self.commands.len())
            .finish()
    }
}

impl AAML {
    /// Creates a new empty [`AAML`] instance with all default commands registered.
    pub fn new() -> AAML {
        let mut instance = AAML {
            map: HashMap::with_hasher(Hasher::new()),
            commands: HashMap::new(),
            types: HashMap::new(),
            schemas: HashMap::new(),
        };
        instance.register_default_commands();
        instance
    }

    /// Creates a new [`AAML`] instance pre-allocated for `capacity` key-value entries.
    pub fn with_capacity(capacity: usize) -> AAML {
        let mut instance = AAML {
            map: HashMap::with_capacity_and_hasher(capacity, Hasher::default()),
            commands: HashMap::new(),
            types: HashMap::new(),
            schemas: HashMap::new(),
        };
        instance.register_default_commands();
        instance
    }

    // ── Internal accessors used by commands ──────────────────────────────────

    pub(crate) fn get_schemas_mut(&mut self) -> &mut HashMap<String, SchemaDef> {
        &mut self.schemas
    }

    pub fn get_schema(&self, name: &str) -> Option<&SchemaDef> {
        self.schemas.get(name)
    }

    pub(crate) fn get_map_mut(&mut self) -> &mut HashMap<AamlString, AamlString, Hasher> {
        &mut self.map
    }

    // ── Type registry ────────────────────────────────────────────────────────

    /// Registers a custom command handler.
    pub fn register_command<C: Command + 'static>(&mut self, command: C) {
        self.commands.insert(command.name().to_string(), Arc::new(command));
    }

    /// Registers a named type definition for use in schema field validation.
    pub fn register_type<T: Type + 'static>(&mut self, name: String, type_def: T) {
        self.types.insert(name, Box::new(type_def));
    }

    /// Returns the type handler registered under `name`, or `None`.
    pub fn get_type(&self, name: &str) -> Option<&dyn Type> {
        self.types.get(name).map(|b| b.as_ref())
    }

    /// Removes the type registered under `name`.
    pub fn unregister_type(&mut self, name: &str) {
        self.types.remove(name);
    }

    /// Validates `value` against a type registered under `type_name`.
    pub fn check_type(&self, type_name: &str, value: &str) -> Result<(), AamlError> {
        self.types
            .get(type_name)
            .ok_or_else(|| AamlError::NotFound(type_name.to_string()))?
            .validate(value)
    }

    /// Validates `value` against the type registered as `type_name`, also
    /// resolving built-in primitive types and module paths.
    pub fn validate_value(&self, type_name: &str, value: &str) -> Result<(), AamlError> {
        let make_err = |e: AamlError| AamlError::InvalidType {
            type_name: type_name.to_string(),
            details: e.to_string(),
        };

        if let Some(type_def) = self.types.get(type_name) {
            return type_def.validate(value).map_err(make_err);
        }

        crate::types::resolve_builtin(type_name)
            .map_err(|_| AamlError::NotFound(type_name.to_string()))?
            .validate(value)
            .map_err(make_err)
    }

    // ── Parsing ──────────────────────────────────────────────────────────────

    /// Parses AAML content from a string, merging it into this instance.
    ///
    /// Multi-line directives (e.g. a `@schema` body spread across several lines)
    /// are accumulated until the opening `{` is matched by a closing `}`.
    pub fn merge_content(&mut self, content: &str) -> Result<(), AamlError> {
        self.map.reserve(content.len() / 40);
        let mut pending: Option<(String, usize)> = None;

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;
            if let Some(result) = self.accumulate_or_process(line, line_num, &mut pending)? {
                self.process_line(&result.0, result.1)?;
            }
        }

        if let Some((buf, start)) = pending {
            self.process_line(&buf, start)?;
        }
        Ok(())
    }

    /// Handles one source line: either appends it to a pending multi-line block
    /// or processes it immediately. Returns `Some((text, line_num))` when a
    /// complete directive has been accumulated and is ready to process.
    fn accumulate_or_process(
        &mut self,
        line: &str,
        line_num: usize,
        pending: &mut Option<(String, usize)>,
    ) -> Result<Option<(String, usize)>, AamlError> {
        if let Some((buf, start)) = pending {
            buf.push(' ');
            buf.push_str(parsing::strip_comment(line).trim());
            if parsing::block_is_complete(buf) {
                let complete = buf.clone();
                let start_line = *start;
                *pending = None;
                return Ok(Some((complete, start_line)));
            }
            return Ok(None);
        }

        let stripped = parsing::strip_comment(line).trim();
        if parsing::needs_accumulation(stripped) {
            *pending = Some((stripped.to_string(), line_num));
            return Ok(None);
        }

        self.process_line(line, line_num)?;
        Ok(None)
    }

    /// Reads a file from disk and merges its content into this instance.
    pub fn merge_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), AamlError> {
        let content = fs::read_to_string(file_path)?;
        self.merge_content(&content)
    }

    /// Parses an AAML string and returns a new [`AAML`] instance.
    pub fn parse(content: &str) -> Result<Self, AamlError> {
        let mut aaml = AAML::new();
        aaml.merge_content(content)?;
        Ok(aaml)
    }

    /// Loads an AAML file from disk and returns a new [`AAML`] instance.
    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self, AamlError> {
        let content = fs::read_to_string(file_path)?;
        Self::parse(&content)
    }

    /// Strips surrounding `"…"` or `'…'` quotes. Returns the trimmed string unchanged
    /// if it is not quoted.
    pub fn unwrap_quotes(s: &str) -> &str {
        parsing::unwrap_quotes(s)
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    fn register_default_commands(&mut self) {
        self.register_command(commands::import::ImportCommand);
        self.register_command(commands::typecm::TypeCommand);
        self.register_command(commands::schema::SchemaCommand);
        self.register_command(commands::derive::DeriveCommand);
    }

    fn process_line(&mut self, raw_line: &str, line_num: usize) -> Result<(), AamlError> {
        let line = parsing::strip_comment(raw_line).trim();
        if line.is_empty() {
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix('@') {
            return self.process_directive(rest, line_num);
        }
        self.process_assignment(line, line_num)
    }

    fn process_assignment(&mut self, line: &str, line_num: usize) -> Result<(), AamlError> {
        match parsing::parse_assignment(line) {
            Ok((key, value)) => {
                self.validate_against_schemas(key, value)?;
                self.map.insert(Box::from(key), Box::from(value));
                Ok(())
            }
            Err(details) => Err(AamlError::ParseError {
                line: line_num,
                content: line.to_string(),
                details: details.to_string(),
            }),
        }
    }

    fn process_directive(&mut self, content: &str, line_num: usize) -> Result<(), AamlError> {
        let mut parts = content.splitn(2, char::is_whitespace);
        let command_name = parts.next().unwrap_or("").trim();
        let args = parts.next().unwrap_or("");

        if command_name.is_empty() {
            return Err(AamlError::ParseError {
                line: line_num,
                content: content.to_string(),
                details: "Empty directive".to_string(),
            });
        }

        let command = self.commands.get(command_name).cloned();
        match command {
            Some(cmd) => cmd.execute(self, args),
            None => Err(AamlError::ParseError {
                line: line_num,
                content: content.to_string(),
                details: format!("Unknown directive: @{}", command_name),
            }),
        }
    }
}

impl Add for AAML {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self.map.reserve(rhs.map.len());
        self.map.extend(rhs.map);
        self.types.extend(rhs.types);
        self
    }
}

impl AddAssign for AAML {
    fn add_assign(&mut self, rhs: Self) {
        self.map.reserve(rhs.map.len());
        self.map.extend(rhs.map);
        self.types.extend(rhs.types);
    }
}

impl Default for AAML {
    fn default() -> Self {
        Self::new()
    }
}

