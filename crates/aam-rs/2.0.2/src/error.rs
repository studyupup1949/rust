//! Error types for the AAML parser and validation pipeline with beautiful colored output.

use colored::Colorize;
use std::cell::RefCell;
use std::fmt;
use std::io;

#[derive(Debug, Clone)]
pub(crate) struct ErrorRenderContext {
    path: String,
    source: String,
}

std::thread_local! {
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

        let col = raw.find(alias_name).map(|p| p + 1).unwrap_or(1);
        return Some((line_num, raw.to_string(), col));
    }
    None
}

/// Error diagnostics with "What", "Why", and "Fix" guidance (inspired by Cargo).
#[derive(Debug, Clone)]
pub struct ErrorDiagnostics {
    /// What went wrong (short title).
    pub what: String,
    /// Why it happened (detailed explanation).
    pub why: String,
    /// How to fix it (suggested resolution).
    pub fix: String,
}

impl ErrorDiagnostics {
    /// Create new diagnostics.
    pub fn new(what: impl Into<String>, why: impl Into<String>, fix: impl Into<String>) -> Self {
        Self {
            what: what.into(),
            why: why.into(),
            fix: fix.into(),
        }
    }

    /// Pretty-print with colors (like Cargo).
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
        diagnostics: Option<ErrorDiagnostics>,
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
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// A key or type name was not found in the registry or map.
    NotFound {
        key: String,
        context: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// A value does not satisfy a basic type constraint (not schema-specific).
    InvalidValue {
        details: String,
        expected: String,
        diagnostics: Option<ErrorDiagnostics>,
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
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// A directive (`@import`, `@derive`, …) encountered an error in its arguments.
    DirectiveError {
        directive: String,
        message: String,
        diagnostics: Option<ErrorDiagnostics>,
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
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Missing required field in schema.
    MissingRequiredField {
        schema: String,
        field: String,
        field_type: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Circular dependency detected.
    CircularDependency {
        path: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Type registration conflict.
    TypeRegistrationConflict {
        type_name: String,
        existing: String,
        new: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Nesting depth exceeded (possible infinite loop).
    NestingDepthExceeded {
        depth: usize,
        context: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Malformed inline object or array literal.
    MalformedLiteral {
        literal_type: String,
        content: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Directive syntax is incorrect.
    DirectiveSyntaxError {
        directive: String,
        provided_syntax: String,
        expected_syntax: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Type conversion failed.
    TypeConversionError {
        from_type: String,
        to_type: String,
        value: String,
        diagnostics: Option<ErrorDiagnostics>,
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
        diagnostics: Option<ErrorDiagnostics>,
    },
}

/// Backward-compatible alias used by some bindings.
pub type AamError = AamlError;

impl AamlError {
    fn code(&self) -> &'static str {
        match self {
            AamlError::CircularDependency { .. } => "E001",
            AamlError::ParseError { .. } => "E002",
            AamlError::InvalidType { .. } => "E003",
            AamlError::SchemaValidationError { .. } => "E004",
            AamlError::MissingRequiredField { .. } => "E005",
            AamlError::NotFound { .. } => "E006",
            AamlError::DirectiveError { .. } => "E007",
            AamlError::DirectiveSyntaxError { .. } => "E008",
            AamlError::MalformedLiteral { .. } => "E009",
            AamlError::TypeRegistrationConflict { .. } => "E010",
            AamlError::TypeConversionError { .. } => "E011",
            AamlError::InvalidValue { .. } => "E012",
            AamlError::NestingDepthExceeded { .. } => "E013",
            AamlError::LexError { .. } => "E014",
            AamlError::IoError { .. } => "E015",
        }
    }

    fn title(&self) -> &'static str {
        match self {
            AamlError::CircularDependency { .. } => "cyclic dependency detected",
            AamlError::ParseError { .. } => "parse error",
            AamlError::InvalidType { .. } => "type validation failed",
            AamlError::SchemaValidationError { .. } => "schema validation failed",
            AamlError::MissingRequiredField { .. } => "missing required field",
            AamlError::NotFound { .. } => "entry not found",
            AamlError::DirectiveError { .. } => "directive execution failed",
            AamlError::DirectiveSyntaxError { .. } => "directive syntax error",
            AamlError::MalformedLiteral { .. } => "malformed literal",
            AamlError::TypeRegistrationConflict { .. } => "type registration conflict",
            AamlError::TypeConversionError { .. } => "type conversion failed",
            AamlError::InvalidValue { .. } => "invalid value",
            AamlError::NestingDepthExceeded { .. } => "nesting depth exceeded",
            AamlError::LexError { .. } => "lexical analysis failed",
            AamlError::IoError { .. } => "I/O operation failed",
        }
    }

    fn default_help(&self) -> &'static str {
        match self {
            AamlError::CircularDependency { .. } => {
                "types in AAM must be acyclic. Consider using a primitive type or breaking the loop."
            }
            AamlError::ParseError { .. } => {
                "check assignment/directive syntax near the highlighted line."
            }
            AamlError::InvalidType { .. } => {
                "ensure the value matches the declared type or update the type declaration."
            }
            AamlError::SchemaValidationError { .. } | AamlError::MissingRequiredField { .. } => {
                "fill required fields and ensure each field value matches its declared type."
            }
            AamlError::NotFound { .. } => {
                "verify the referenced key/type/schema exists and is in scope."
            }
            AamlError::DirectiveError { .. } | AamlError::DirectiveSyntaxError { .. } => {
                "check directive name and argument format."
            }
            AamlError::MalformedLiteral { .. } => {
                "ensure object/list literals are balanced and well-formed."
            }
            AamlError::TypeRegistrationConflict { .. } => {
                "rename the type or remove duplicate @type declarations."
            }
            AamlError::TypeConversionError { .. } => {
                "provide a value that can be converted to the requested type."
            }
            AamlError::InvalidValue { .. } => "provide a value in the expected format.",
            AamlError::NestingDepthExceeded { .. } => {
                "reduce recursion depth or split nested data."
            }
            AamlError::LexError { .. } => "remove or replace unsupported characters.",
            AamlError::IoError { .. } => "check file path and permissions.",
        }
    }

    fn render_compiler_style(&self) -> String {
        let ctx = get_error_render_context();
        let mut out = String::new();
        out.push_str(&format!(
            "{}[{}]: {}\n",
            "error".red().bold(),
            self.code().red().bold(),
            self.title().bold()
        ));

        match self {
            AamlError::CircularDependency { path, .. } => {
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
                    path.to_string()
                } else {
                    nodes.join(" -> ")
                };

                let first_alias = nodes.first().cloned().unwrap_or_else(|| "?".to_string());
                let first_loc = type_alias_line_info(&ctx.source, &first_alias);

                if let Some((line, _, col)) = first_loc {
                    out.push_str(&format!(
                        " {} {}:{}:{}\n",
                        "-->".blue().bold(),
                        ctx.path,
                        line,
                        col
                    ));
                } else {
                    out.push_str(&format!(" {} {}:?:?\n", "-->".blue().bold(), ctx.path));
                }
                out.push_str(&format!(" {}\n", "|".blue().bold()));

                for (idx, alias) in nodes.iter().enumerate() {
                    if let Some((line, src, col)) = type_alias_line_info(&ctx.source, alias) {
                        out.push_str(&format!(
                            " {} {} {}\n",
                            line.to_string().blue().bold(),
                            "|".blue().bold(),
                            src
                        ));
                        let marker = if idx == 0 {
                            "cycle starts here"
                        } else if idx == nodes.len().saturating_sub(1) {
                            "cycle closes here"
                        } else {
                            "part of cycle"
                        };
                        out.push_str(&format!(
                            " {} {}{} {}\n",
                            "|".blue().bold(),
                            " ".repeat(col.saturating_sub(1)),
                            "^".red().bold(),
                            marker
                        ));
                    }
                }

                out.push_str(&format!(" {}\n", "|".blue().bold()));
                out.push_str(&format!(
                    " {} {} {}\n",
                    "=".blue().bold(),
                    "cycle:".bold(),
                    chain
                ));
                out.push_str(&format!(
                    " {} returns to '{}'\n",
                    "=".blue().bold(),
                    cycle_start
                ));
            }
            AamlError::ParseError {
                line,
                content,
                details,
                ..
            } => {
                out.push_str(&format!(
                    " {} {}:{}:{}\n",
                    "-->".blue().bold(),
                    ctx.path,
                    line,
                    1
                ));
                out.push_str(&format!(" {}\n", "|".blue().bold()));
                let display_line = source_line(&ctx.source, *line).unwrap_or(content.as_str());
                let caret_col = display_line
                    .find(content.trim())
                    .map(|v| v + 1)
                    .unwrap_or(1);
                out.push_str(&format!(
                    " {} {} {}\n",
                    line.to_string().blue().bold(),
                    "|".blue().bold(),
                    display_line
                ));
                out.push_str(&format!(
                    " {} {} {}\n",
                    "|".blue().bold(),
                    "^".red().bold(),
                    details
                ));
                out.push_str(&format!(
                    " {} {}{}\n",
                    "|".blue().bold(),
                    " ".repeat(caret_col.saturating_sub(1)),
                    "^-- here".red().bold()
                ));
            }
            AamlError::LexError {
                line,
                column,
                character,
                ..
            } => {
                out.push_str(&format!(
                    " {} {}:{}:{}\n",
                    "-->".blue().bold(),
                    ctx.path,
                    line,
                    column
                ));
                if let Some(src) = source_line(&ctx.source, *line) {
                    out.push_str(&format!(
                        " {} {} {}\n",
                        line.to_string().blue().bold(),
                        "|".blue().bold(),
                        src
                    ));
                    out.push_str(&format!(
                        " {} {}{} invalid character '{}'\n",
                        "|".blue().bold(),
                        " ".repeat(column.saturating_sub(1)),
                        "^--".red().bold(),
                        character
                    ));
                } else {
                    out.push_str(&format!(
                        " {} invalid character '{}'\n",
                        "|".blue().bold(),
                        character
                    ));
                }
            }
            _ => {
                out.push_str(&format!(" {} {}:?:?\n", "-->".blue().bold(), ctx.path));
                out.push_str(&format!(
                    " {} {}\n",
                    "|".blue().bold(),
                    self.short_message()
                ));
            }
        }

        if let Some(diag) = self.diagnostics() {
            out.push_str(&format!(" {}\n", "|".blue().bold()));
            out.push_str(&format!(" {} {}\n", "help:".green().bold(), diag.fix));
        } else {
            out.push_str(&format!(" {}\n", "|".blue().bold()));
            out.push_str(&format!(
                " {} {}\n",
                "help:".green().bold(),
                self.default_help()
            ));
        }

        out
    }

    /// Get the primary error message (short form).
    pub fn short_message(&self) -> String {
        match self {
            AamlError::IoError { details, .. } => format!("IO error: {}", details),
            AamlError::ParseError {
                line,
                content,
                details,
                ..
            } => {
                format!("Parse error at line {}: {} ({})", line, content, details)
            }
            AamlError::NotFound { key, context, .. } => {
                format!("Key '{}' not found ({})", key, context)
            }
            AamlError::InvalidValue {
                details, expected, ..
            } => {
                format!("Invalid value: {} (expected: {})", details, expected)
            }
            AamlError::InvalidType {
                type_name,
                provided,
                details,
                ..
            } => {
                format!(
                    "Invalid type '{}': {} (got: {})",
                    type_name, details, provided
                )
            }
            AamlError::DirectiveError {
                directive, message, ..
            } => {
                format!("Directive '@{}' error: {}", directive, message)
            }
            AamlError::SchemaValidationError {
                schema,
                field,
                type_name,
                details,
                ..
            } => {
                format!(
                    "Schema '{}' field '{}' ({}): {}",
                    schema, field, type_name, details
                )
            }
            AamlError::MissingRequiredField {
                schema,
                field,
                field_type,
                ..
            } => {
                format!(
                    "Missing required field '{}' in schema '{}' (type: {})",
                    field, schema, field_type
                )
            }
            AamlError::CircularDependency { path, .. } => {
                format!("Circular dependency detected: {}", path)
            }
            AamlError::TypeRegistrationConflict {
                type_name,
                existing,
                new,
                ..
            } => {
                format!(
                    "Type '{}' already defined as '{}', cannot redefine as '{}'",
                    type_name, existing, new
                )
            }
            AamlError::NestingDepthExceeded { depth, context, .. } => {
                format!("Nesting depth exceeded ({}): {}", depth, context)
            }
            AamlError::MalformedLiteral {
                literal_type,
                content,
                ..
            } => {
                format!("Malformed {} literal: {}", literal_type, content)
            }
            AamlError::DirectiveSyntaxError {
                directive,
                provided_syntax,
                expected_syntax,
                ..
            } => {
                format!(
                    "Directive '@{}' syntax error: got '{}', expected '{}'",
                    directive, provided_syntax, expected_syntax
                )
            }
            AamlError::TypeConversionError {
                from_type,
                to_type,
                value,
                ..
            } => {
                format!(
                    "Cannot convert '{}' from {} to {}",
                    value, from_type, to_type
                )
            }
            AamlError::LexError {
                line,
                column,
                character,
                ..
            } => {
                format!(
                    "Lexical error at {}:{}: invalid character '{}'",
                    line, column, character
                )
            }
        }
    }

    /// Get the detailed diagnostics if available.
    pub fn diagnostics(&self) -> Option<&ErrorDiagnostics> {
        match self {
            AamlError::IoError { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::ParseError { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::NotFound { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::InvalidValue { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::InvalidType { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::DirectiveError { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::SchemaValidationError { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::MissingRequiredField { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::CircularDependency { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::TypeRegistrationConflict { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::NestingDepthExceeded { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::MalformedLiteral { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::DirectiveSyntaxError { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::TypeConversionError { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::LexError { diagnostics, .. } => diagnostics.as_ref(),
        }
    }
}

impl fmt::Display for AamlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render_compiler_style())
    }
}

impl std::error::Error for AamlError {}

impl From<io::Error> for AamlError {
    fn from(err: io::Error) -> Self {
        let details = err.to_string();
        let diagnostics = Some(ErrorDiagnostics::new(
            "I/O operation failed",
            format!("Could not read or write file: {}", details),
            "Check file permissions and ensure the path exists",
        ));
        AamlError::IoError {
            details,
            diagnostics,
        }
    }
}
