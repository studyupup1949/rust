#![allow(clippy::format_push_string, clippy::too_many_lines)]

//! Error types for the AAML parser and validation pipeline with beautiful colored output.

use colored::Colorize;
use smol_str::SmolStr;
use std::cell::RefCell;
use std::fmt;
use std::io;

#[derive(Debug, Clone)]
pub(crate) struct ErrorRenderContext {
    path: String,
    source: String,
}

thread_local! {
    static ERROR_RENDER_CONTEXT: RefCell<ErrorRenderContext> = RefCell::new(ErrorRenderContext {
        path: "<raw_string>".to_string(),
        source: String::new(),
    });
}

pub(crate) struct ErrorRenderContextGuard {
    previous: ErrorRenderContext,
}

impl Drop for ErrorRenderContextGuard {
    fn drop(&mut self) {
        let previous = self.previous.clone();
        ERROR_RENDER_CONTEXT.with(|ctx| {
            *ctx.borrow_mut() = previous;
        });
    }
}

pub fn set_error_render_context(path: impl Into<String>, source: impl Into<String>) {
    let path = path.into();
    let source = source.into();
    ERROR_RENDER_CONTEXT.with(|ctx| {
        let mut ctx_mut = ctx.borrow_mut();
        ctx_mut.path = path;
        ctx_mut.source = source;
    });
}

pub(crate) fn push_error_render_context(
    path: impl Into<String>,
    source: impl Into<String>,
) -> ErrorRenderContextGuard {
    let previous = get_error_render_context();
    set_error_render_context(path, source);
    ErrorRenderContextGuard { previous }
}

fn get_error_render_context() -> ErrorRenderContext {
    ERROR_RENDER_CONTEXT.with(|ctx| ctx.borrow().clone())
}

fn source_line(source: &str, line: usize) -> Option<&str> {
    if line == 0 {
        return None;
    }
    source.lines().nth(line - 1)
}

fn type_alias_line_info(source: &str, alias_name: &str) -> Option<(usize, String, usize)> {
    for (idx, raw) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw.trim_start();
        if !trimmed.starts_with("@type") {
            continue;
        }

        let rest = trimmed.trim_start_matches("@type").trim_start();
        let Some((lhs, _rhs)) = rest.split_once('=') else {
            continue;
        };
        if lhs.trim() != alias_name {
            continue;
        }

        let col = raw.find(alias_name).map_or(1, |p| p + 1);
        return Some((line_num, raw.to_string(), col));
    }
    None
}

/// Error diagnostics with "What", "Why", and "Fix" guidance (inspired by Cargo).
#[derive(Debug, Clone)]
pub struct ErrorDiagnostics {
    /// What went wrong (short title).
    pub what: SmolStr,
    /// Why it happened (detailed explanation).
    pub why: SmolStr,
    /// How to fix it (suggested resolution).
    pub fix: SmolStr,
}

impl ErrorDiagnostics {
    /// Create new diagnostics.
    pub fn new(what: impl Into<SmolStr>, why: impl Into<SmolStr>, fix: impl Into<SmolStr>) -> Self {
        Self {
            what: what.into(),
            why: why.into(),
            fix: fix.into(),
        }
    }

    /// Pretty-print with colors (like Cargo).
    #[must_use]
    pub fn pretty_print(&self) -> String {
        format!(
            "{}\n{}\n\n{}\n{}\n\n{}\n{}\n",
            "error".red().bold(),
            format!("  {}", self.what).red(),
            "why".cyan().bold(),
            format!("  {}", self.why).cyan(),
            "fix".green().bold(),
            format!("  {}", self.fix).green(),
        )
    }
}

/// All errors that can be produced while parsing or validating an AAML document.
#[derive(Debug)]
pub enum AamlError {
    /// An I/O error occurred while reading a file.
    IoError {
        details: String,
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// A line could not be parsed as a valid AAML statement.
    ParseError {
        /// 1-based line number in the source file.
        line: usize,
        /// Raw content of the offending line.
        content: String,
        /// Human-readable explanation of why parsing failed.
        details: String,
        /// Diagnostic guidance.
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// A key or type name was not found in the registry or map.
    NotFound {
        key: String,
        context: String,
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// A value does not satisfy a basic type constraint (not schema-specific).
    InvalidValue {
        details: String,
        expected: String,
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// A value failed validation against a registered or built-in type.
    InvalidType {
        /// Name of the type that rejected the value.
        type_name: String,
        /// Details from the type validator.
        details: String,
        /// What was provided.
        provided: String,
        /// Diagnostic guidance.
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// A directive (`@import`, `@derive`, …) encountered an error in its arguments.
    DirectiveError {
        directive: String,
        message: String,
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// A schema constraint was violated during parsing or explicit validation.
    SchemaValidationError {
        /// Name of the schema that declared the field.
        schema: String,
        /// Name of the field that failed validation.
        field: String,
        /// Declared type of the field.
        type_name: String,
        /// Human-readable description of the failure.
        details: String,
        /// Diagnostic guidance.
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// Missing the required field in schema.
    MissingRequiredField {
        schema: String,
        field: String,
        field_type: String,
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// Circular dependency detected.
    CircularDependency {
        path: String,
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// Type registration conflict.
    TypeRegistrationConflict {
        type_name: String,
        existing: String,
        new: String,
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// Nesting depth exceeded (possible infinite loop).
    NestingDepthExceeded {
        depth: usize,
        context: String,
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// Malformed inline object or array literal.
    MalformedLiteral {
        literal_type: String,
        content: String,
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// Directive syntax is incorrect.
    DirectiveSyntaxError {
        directive: String,
        provided_syntax: String,
        expected_syntax: String,
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// Type conversion failed.
    TypeConversionError {
        from_type: String,
        to_type: String,
        value: String,
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },

    /// Lexical analysis error (invalid character or token).
    LexError {
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
        /// The invalid character
        character: String,
        /// Diagnostic guidance
        diagnostics: Option<Box<ErrorDiagnostics>>,
    },
}

/// Backward-compatible alias used by some bindings.
pub type AamError = AamlError;

impl AamlError {
    const fn code(&self) -> &'static str {
        match self {
            Self::CircularDependency { .. } => "E001",
            Self::ParseError { .. } => "E002",
            Self::InvalidType { .. } => "E003",
            Self::SchemaValidationError { .. } => "E004",
            Self::MissingRequiredField { .. } => "E005",
            Self::NotFound { .. } => "E006",
            Self::DirectiveError { .. } => "E007",
            Self::DirectiveSyntaxError { .. } => "E008",
            Self::MalformedLiteral { .. } => "E009",
            Self::TypeRegistrationConflict { .. } => "E010",
            Self::TypeConversionError { .. } => "E011",
            Self::InvalidValue { .. } => "E012",
            Self::NestingDepthExceeded { .. } => "E013",
            Self::LexError { .. } => "E014",
            Self::IoError { .. } => "E015",
        }
    }

    const fn title(&self) -> &'static str {
        match self {
            Self::CircularDependency { .. } => "cyclic dependency detected",
            Self::ParseError { .. } => "parse error",
            Self::InvalidType { .. } => "type validation failed",
            Self::SchemaValidationError { .. } => "schema validation failed",
            Self::MissingRequiredField { .. } => "missing required field",
            Self::NotFound { .. } => "entry not found",
            Self::DirectiveError { .. } => "directive execution failed",
            Self::DirectiveSyntaxError { .. } => "directive syntax error",
            Self::MalformedLiteral { .. } => "malformed literal",
            Self::TypeRegistrationConflict { .. } => "type registration conflict",
            Self::TypeConversionError { .. } => "type conversion failed",
            Self::InvalidValue { .. } => "invalid value",
            Self::NestingDepthExceeded { .. } => "nesting depth exceeded",
            Self::LexError { .. } => "lexical analysis failed",
            Self::IoError { .. } => "I/O operation failed",
        }
    }

    const fn default_help(&self) -> &'static str {
        match self {
            Self::CircularDependency { .. } => {
                "types in AAM must be acyclic. Consider using a primitive type or breaking the loop."
            }
            Self::ParseError { .. } => {
                "check assignment/directive syntax near the highlighted line."
            }
            Self::InvalidType { .. } => {
                "ensure the value matches the declared type or update the type declaration."
            }
            Self::SchemaValidationError { .. } | Self::MissingRequiredField { .. } => {
                "fill required fields and ensure each field value matches its declared type."
            }
            Self::NotFound { .. } => {
                "verify the referenced key/type/schema exists and is in scope."
            }
            Self::DirectiveError { .. } | Self::DirectiveSyntaxError { .. } => {
                "check directive name and argument format."
            }
            Self::MalformedLiteral { .. } => {
                "ensure object/list literals are balanced and well-formed."
            }
            Self::TypeRegistrationConflict { .. } => {
                "rename the type or remove duplicate @type declarations."
            }
            Self::TypeConversionError { .. } => {
                "provide a value that can be converted to the requested type."
            }
            Self::InvalidValue { .. } => "provide a value in the expected format.",
            Self::NestingDepthExceeded { .. } => "reduce recursion depth or split nested data.",
            Self::LexError { .. } => "remove or replace unsupported characters.",
            Self::IoError { .. } => "check file path and permissions.",
        }
    }

    fn render_error_header(&self, ctx: &ErrorRenderContext, out: &mut String) {
        use std::fmt::Write;
        let _ = writeln!(
            out,
            "{}[{}]: {}",
            "error".red().bold(),
            self.code().red().bold(),
            self.title().bold()
        );
        let _ = writeln!(out, " {} {}:?:?", "-->".blue().bold(), ctx.path);
        let _ = writeln!(out, " {} {}", "|".blue().bold(), self.short_message());
    }

    fn render_circular_dep(&self, ctx: &ErrorRenderContext, out: &mut String) {
        use std::fmt::Write;
        let Self::CircularDependency { path, .. } = self else {
            return;
        };
        let nodes: Vec<String> = path
            .split("->")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .collect();
        let cycle_start = nodes
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let chain = if nodes.is_empty() {
            path.clone()
        } else {
            nodes.join(" -> ")
        };

        let first_alias = nodes.first().cloned().unwrap_or_else(|| "?".to_string());
        let first_loc = type_alias_line_info(&ctx.source, &first_alias);

        if let Some((line, _, col)) = first_loc {
            let _ = writeln!(
                out,
                " {} {}:{}:{}",
                "-->".blue().bold(),
                ctx.path,
                line,
                col
            );
        }
        let _ = writeln!(out, " {}", "|".blue().bold());

        for (idx, alias) in nodes.iter().enumerate() {
            if let Some((line, src, col)) = type_alias_line_info(&ctx.source, alias) {
                let _ = writeln!(
                    out,
                    " {} {} {}",
                    line.to_string().blue().bold(),
                    "|".blue().bold(),
                    src
                );
                let marker = if idx == 0 {
                    "cycle starts here"
                } else if idx == nodes.len().saturating_sub(1) {
                    "cycle closes here"
                } else {
                    "part of cycle"
                };
                let _ = writeln!(
                    out,
                    " {} {}{} {}",
                    "|".blue().bold(),
                    " ".repeat(col.saturating_sub(1)),
                    "^".red().bold(),
                    marker
                );
            }
        }

        let _ = writeln!(out, " {}", "|".blue().bold());
        let _ = writeln!(out, " {} {} {}", "=".blue().bold(), "cycle:".bold(), chain);
        let _ = writeln!(out, " {} returns to '{}'", "=".blue().bold(), cycle_start);
    }

    fn render_parse_error(&self, ctx: &ErrorRenderContext, out: &mut String) {
        use std::fmt::Write;
        let Self::ParseError {
            line,
            content,
            details,
            ..
        } = self
        else {
            return;
        };
        let _ = writeln!(out, " {} {}:{}:{}", "-->".blue().bold(), ctx.path, line, 1);
        let _ = writeln!(out, " {}", "|".blue().bold());
        let display_line = source_line(&ctx.source, *line).unwrap_or(content.as_str());
        let caret_col = display_line.find(content.trim()).map_or(1, |v| v + 1);
        let _ = writeln!(
            out,
            " {} {} {}",
            line.to_string().blue().bold(),
            "|".blue().bold(),
            display_line
        );
        let _ = writeln!(
            out,
            " {} {} {}",
            "|".blue().bold(),
            "^".red().bold(),
            details
        );
        let _ = writeln!(
            out,
            " {} {}{}",
            "|".blue().bold(),
            " ".repeat(caret_col.saturating_sub(1)),
            "^-- here".red().bold()
        );
    }

    fn render_lex_error(&self, ctx: &ErrorRenderContext, out: &mut String) {
        use std::fmt::Write;
        let Self::LexError {
            line,
            column,
            character,
            ..
        } = self
        else {
            return;
        };
        let _ = writeln!(
            out,
            " {} {}:{}:{}",
            "-->".blue().bold(),
            ctx.path,
            line,
            column
        );
        if let Some(src) = source_line(&ctx.source, *line) {
            let _ = writeln!(
                out,
                " {} {} {}",
                line.to_string().blue().bold(),
                "|".blue().bold(),
                src
            );
            let _ = writeln!(
                out,
                " {} {}{} invalid character '{}'",
                "|".blue().bold(),
                " ".repeat(column.saturating_sub(1)),
                "^--".red().bold(),
                character
            );
        } else {
            let _ = writeln!(
                out,
                " {} invalid character '{}'",
                "|".blue().bold(),
                character
            );
        }
    }

    fn render_compiler_style(&self) -> String {
        use std::fmt::Write;
        let ctx = get_error_render_context();
        let mut out = String::new();
        let _ = writeln!(
            out,
            "{}[{}]: {}",
            "error".red().bold(),
            self.code().red().bold(),
            self.title().bold()
        );

        match self {
            Self::CircularDependency { .. } => self.render_circular_dep(&ctx, &mut out),
            Self::ParseError { .. } => self.render_parse_error(&ctx, &mut out),
            Self::LexError { .. } => self.render_lex_error(&ctx, &mut out),
            _ => self.render_error_header(&ctx, &mut out),
        }

        let _ = writeln!(out, " {}", "|".blue().bold());
        if let Some(diag) = self.diagnostics() {
            let _ = writeln!(out, " {} {}", "help:".green().bold(), diag.fix);
        } else {
            let _ = writeln!(out, " {} {}", "help:".green().bold(), self.default_help());
        }

        out
    }

    /// Get the primary error message (short form).
    #[must_use]
    pub fn short_message(&self) -> String {
        match self {
            Self::IoError { details, .. } => format!("IO error: {details}"),
            Self::ParseError {
                line,
                content,
                details,
                ..
            } => {
                format!("Parse error at line {line}: {content} ({details})")
            }
            Self::NotFound { key, context, .. } => {
                format!("Key '{key}' not found ({context})")
            }
            Self::InvalidValue {
                details, expected, ..
            } => {
                format!("Invalid value: {details} (expected: {expected})")
            }
            Self::InvalidType {
                type_name,
                provided,
                details,
                ..
            } => {
                format!("Invalid type '{type_name}': {details} (got: {provided})")
            }
            Self::DirectiveError {
                directive, message, ..
            } => {
                format!("Directive '@{directive}' error: {message}")
            }
            Self::SchemaValidationError {
                schema,
                field,
                type_name,
                details,
                ..
            } => {
                format!("Schema '{schema}' field '{field}' ({type_name}): {details}")
            }
            Self::MissingRequiredField {
                schema,
                field,
                field_type,
                ..
            } => {
                format!(
                    "Missing required field '{field}' in schema '{schema}' (type: {field_type})"
                )
            }
            Self::CircularDependency { path, .. } => {
                format!("Circular dependency detected: {path}")
            }
            Self::TypeRegistrationConflict {
                type_name,
                existing,
                new,
                ..
            } => {
                format!(
                    "Type '{type_name}' already defined as '{existing}', cannot redefine as '{new}'"
                )
            }
            Self::NestingDepthExceeded { depth, context, .. } => {
                format!("Nesting depth exceeded ({depth}): {context}")
            }
            Self::MalformedLiteral {
                literal_type,
                content,
                ..
            } => {
                format!("Malformed {literal_type} literal: {content}")
            }
            Self::DirectiveSyntaxError {
                directive,
                provided_syntax,
                expected_syntax,
                ..
            } => {
                format!(
                    "Directive '@{directive}' syntax error: got '{provided_syntax}', expected '{expected_syntax}'"
                )
            }
            Self::TypeConversionError {
                from_type,
                to_type,
                value,
                ..
            } => {
                format!("Cannot convert '{value}' from {from_type} to {to_type}")
            }
            Self::LexError {
                line,
                column,
                character,
                ..
            } => {
                format!("Lexical error at {line}:{column}: invalid character '{character}'")
            }
        }
    }

    /// Get the detailed diagnostics if available.
    #[must_use]
    pub fn diagnostics(&self) -> Option<&ErrorDiagnostics> {
        match self {
            Self::IoError { diagnostics, .. }
            | Self::ParseError { diagnostics, .. }
            | Self::NotFound { diagnostics, .. }
            | Self::InvalidValue { diagnostics, .. }
            | Self::InvalidType { diagnostics, .. }
            | Self::DirectiveError { diagnostics, .. }
            | Self::SchemaValidationError { diagnostics, .. }
            | Self::MissingRequiredField { diagnostics, .. }
            | Self::CircularDependency { diagnostics, .. }
            | Self::TypeRegistrationConflict { diagnostics, .. }
            | Self::NestingDepthExceeded { diagnostics, .. }
            | Self::MalformedLiteral { diagnostics, .. }
            | Self::DirectiveSyntaxError { diagnostics, .. }
            | Self::TypeConversionError { diagnostics, .. }
            | Self::LexError { diagnostics, .. } => diagnostics.as_deref(),
        }
    }
}

impl fmt::Display for AamlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render_compiler_style())
    }
}

impl std::error::Error for AamlError {}

#[cfg(feature = "serde")]
impl serde::de::Error for AamlError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        AamlError::InvalidValue {
            details: msg.to_string(),
            expected: String::new(),
            diagnostics: None,
        }
    }
}

impl From<io::Error> for AamlError {
    fn from(err: io::Error) -> Self {
        let details = err.to_string();
        let diagnostics = Some(Box::new(ErrorDiagnostics::new(
            "I/O operation failed",
            format!("Could not read or write file: {details}"),
            "Check file permissions and ensure the path exists",
        )));
        Self::IoError {
            details,
            diagnostics,
        }
    }
}
