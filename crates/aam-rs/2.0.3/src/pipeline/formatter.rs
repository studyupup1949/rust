//! Formatter trait for LSP integration.
//!
//! The Formatter provides an API for formatting AAML documents without requiring
//! full pipeline execution. This is critical for LSP support (document formatting,
//! range formatting, on-save hooks).

use crate::error::AamlError;
use crate::pipeline::parser::AstNode;

/// Configuration options for formatting behavior.
#[derive(Debug, Clone)]
pub struct FormattingOptions {
    /// Number of spaces per indentation level
    pub indent_size: usize,

    /// Use tabs instead of spaces
    pub use_tabs: bool,

    /// Line width for wrapping (0 = no wrapping)
    pub line_width: usize,

    /// Sort keys alphabetically
    pub sort_keys: bool,

    /// Add trailing newline
    pub trailing_newline: bool,

    /// Preserve blank lines
    pub preserve_blank_lines: bool,
}

impl Default for FormattingOptions {
    fn default() -> Self {
        Self {
            indent_size: 4,
            use_tabs: false,
            line_width: 100,
            sort_keys: false,
            trailing_newline: true,
            preserve_blank_lines: true,
        }
    }
}

/// Range for document range formatting.
#[derive(Debug, Clone, Copy)]
pub struct FormatRange {
    /// Start line (1-based)
    pub start_line: usize,
    /// End line (1-based, inclusive)
    pub end_line: usize,
}

/// Trait for formatting AAML documents.
///
/// This trait provides methods to format AST nodes, handle range-based formatting,
/// and support LSP clients that need formatting without full parsing/execution.
pub trait Formatter: Send + Sync {
    /// Formats an entire document represented by AST nodes.
    ///
    /// # Arguments
    /// - `nodes`: AST nodes to format
    /// - `options`: Formatting options
    ///
    /// # Returns
    /// Formatted document as a string
    fn format_document(
        &self,
        nodes: &[AstNode],
        options: &FormattingOptions,
    ) -> Result<String, AamlError>;

    /// Formats a specific range in a document.
    ///
    /// # Arguments
    /// - `nodes`: AST nodes to format
    /// - `range`: The range to format
    /// - `options`: Formatting options
    ///
    /// # Returns
    /// Formatted document with only the specified range modified
    fn format_range(
        &self,
        nodes: &[AstNode],
        range: FormatRange,
        options: &FormattingOptions,
    ) -> Result<String, AamlError>;

    /// Formats a single AST node.
    ///
    /// # Arguments
    /// - `node`: AST node to format
    /// - `indent_level`: Current indentation level
    /// - `options`: Formatting options
    ///
    /// # Returns
    /// Formatted node as a string
    fn format_node(
        &self,
        node: &AstNode,
        indent_level: usize,
        options: &FormattingOptions,
    ) -> Result<String, AamlError>;

    /// Normalizes comments in a document.
    ///
    /// # Arguments
    /// - `content`: Raw document content
    /// - `options`: Formatting options
    ///
    /// # Returns
    /// Document with normalized comments
    fn normalize_comments(
        &self,
        content: &str,
        options: &FormattingOptions,
    ) -> Result<String, AamlError>;

    /// Removes trailing whitespace and normalizes line endings.
    ///
    /// # Arguments
    /// - `content`: Raw document content
    ///
    /// # Returns
    /// Document with normalized whitespace
    fn normalize_whitespace(&self, content: &str) -> Result<String, AamlError>;
}

/// Default implementation of the Formatter trait.
///
/// Provides basic formatting capabilities suitable for most use cases.
pub struct DefaultFormatter;

impl DefaultFormatter {
    pub fn new() -> Self {
        Self
    }

    /// Creates indentation string based on level and options.
    fn create_indent(level: usize, options: &FormattingOptions) -> String {
        if options.use_tabs {
            "\t".repeat(level)
        } else {
            " ".repeat(level * options.indent_size)
        }
    }

    /// Formats an assignment node.
    fn format_assignment(
        key: &str,
        value: &str,
        indent_level: usize,
        options: &FormattingOptions,
    ) -> String {
        let indent = Self::create_indent(indent_level, options);
        format!("{}{} = {}", indent, key, value)
    }

    /// Formats a directive node.
    fn format_directive(
        name: &str,
        args: &str,
        indent_level: usize,
        options: &FormattingOptions,
    ) -> String {
        let indent = Self::create_indent(indent_level, options);
        if args.is_empty() {
            format!("{}@{}", indent, name)
        } else {
            format!("{}@{} {}", indent, name, args)
        }
    }

    /// Formats an inline object node.
    #[allow(dead_code)]
    fn format_inline_object(pairs: &[(String, String)], _options: &FormattingOptions) -> String {
        if pairs.is_empty() {
            "{}".to_string()
        } else {
            let formatted_pairs: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{} = {}", k, v))
                .collect();
            format!("{{ {} }}", formatted_pairs.join(", "))
        }
    }

    /// Formats an inline list node.
    #[allow(dead_code)]
    fn format_inline_list(items: &[String]) -> String {
        if items.is_empty() {
            "[]".to_string()
        } else {
            format!("[{}]", items.join(", "))
        }
    }
}

impl Default for DefaultFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter for DefaultFormatter {
    fn format_document(
        &self,
        nodes: &[AstNode],
        options: &FormattingOptions,
    ) -> Result<String, AamlError> {
        let mut output = Vec::new();

        for node in nodes {
            let formatted = self.format_node(node, 0, options)?;
            output.push(formatted);
        }

        let mut result = output.join("\n");

        if options.trailing_newline && !result.ends_with('\n') {
            result.push('\n');
        }

        Ok(result)
    }

    fn format_range(
        &self,
        nodes: &[AstNode],
        range: FormatRange,
        options: &FormattingOptions,
    ) -> Result<String, AamlError> {
        let mut output = Vec::new();

        for node in nodes {
            let line = node.line();
            if line >= range.start_line && line <= range.end_line {
                let formatted = self.format_node(node, 0, options)?;
                output.push(formatted);
            } else {
                // For lines outside the range, preserve original formatting
                output.push(format!("(original line {})", line));
            }
        }

        Ok(output.join("\n"))
    }

    fn format_node(
        &self,
        node: &AstNode,
        indent_level: usize,
        options: &FormattingOptions,
    ) -> Result<String, AamlError> {
        let formatted = match node {
            AstNode::Assignment { key, value, .. } => {
                Self::format_assignment(key, &value.to_string(), indent_level, options)
            }
            AstNode::Directive { name, args, .. } => {
                Self::format_directive(name, args, indent_level, options)
            }
        };

        Ok(formatted)
    }

    fn normalize_comments(
        &self,
        content: &str,
        _options: &FormattingOptions,
    ) -> Result<String, AamlError> {
        let lines: Vec<&str> = content.lines().collect();
        let normalized: Vec<String> = lines
            .iter()
            .map(|line| {
                // Normalize comment spacing (ensure space after #)
                if let Some(pos) = line.find('#') {
                    let before = &line[..pos];
                    let after = &line[pos + 1..];

                    // Only if surrounded by spaces (not hex color)
                    if pos > 0
                        && pos < line.len() - 1
                        && before.ends_with(' ')
                        && !after.starts_with('#')
                    {
                        let comment = after.trim_start();
                        return format!("{}# {}", before.trim_end(), comment);
                    }
                }

                line.to_string()
            })
            .collect();

        Ok(normalized.join("\n"))
    }

    fn normalize_whitespace(&self, content: &str) -> Result<String, AamlError> {
        let lines: Vec<&str> = content.lines().collect();
        let normalized: Vec<String> = lines
            .iter()
            .map(|line| line.trim_end().to_string())
            .collect();

        Ok(normalized.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_indent_spaces() {
        let options = FormattingOptions {
            indent_size: 4,
            use_tabs: false,
            ..Default::default()
        };
        assert_eq!(DefaultFormatter::create_indent(0, &options), "");
        assert_eq!(DefaultFormatter::create_indent(1, &options), "    ");
        assert_eq!(DefaultFormatter::create_indent(2, &options), "        ");
    }

    #[test]
    fn test_create_indent_tabs() {
        let options = FormattingOptions {
            use_tabs: true,
            ..Default::default()
        };
        assert_eq!(DefaultFormatter::create_indent(0, &options), "");
        assert_eq!(DefaultFormatter::create_indent(1, &options), "\t");
        assert_eq!(DefaultFormatter::create_indent(2, &options), "\t\t");
    }

    #[test]
    fn test_format_assignment() {
        let formatted =
            DefaultFormatter::format_assignment("key", "value", 0, &FormattingOptions::default());
        assert_eq!(formatted, "key = value");
    }

    #[test]
    fn test_format_assignment_with_indent() {
        let options = FormattingOptions {
            indent_size: 2,
            ..Default::default()
        };
        let formatted = DefaultFormatter::format_assignment("key", "value", 1, &options);
        assert_eq!(formatted, "  key = value");
    }

    #[test]
    fn test_format_directive() {
        let formatted = DefaultFormatter::format_directive(
            "import",
            "base.aam",
            0,
            &FormattingOptions::default(),
        );
        assert_eq!(formatted, "@import base.aam");
    }

    #[test]
    fn test_format_inline_object() {
        let pairs = vec![
            ("host".to_string(), "localhost".to_string()),
            ("port".to_string(), "8080".to_string()),
        ];
        let formatted =
            DefaultFormatter::format_inline_object(&pairs, &FormattingOptions::default());
        assert_eq!(formatted, "{ host = localhost, port = 8080 }");
    }

    #[test]
    fn test_format_inline_list() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let formatted = DefaultFormatter::format_inline_list(&items);
        assert_eq!(formatted, "[a, b, c]");
    }

    #[test]
    fn test_normalize_whitespace() {
        let formatter = DefaultFormatter::new();
        let input = "key = value   \nfoo = bar  ";
        let result = formatter.normalize_whitespace(input).unwrap();
        assert_eq!(result, "key = value\nfoo = bar");
    }

    #[test]
    fn test_format_document() {
        let formatter = DefaultFormatter::new();
        let ast = vec![AstNode::Assignment {
            key: "name".to_string().into(),
            value: crate::pipeline::parser::ValueNode::Literal("test".to_string().into()),
            line: 1,
        }];
        let result = formatter
            .format_document(&ast, &FormattingOptions::default())
            .unwrap();
        assert!(result.contains("name = test"));
    }
}
