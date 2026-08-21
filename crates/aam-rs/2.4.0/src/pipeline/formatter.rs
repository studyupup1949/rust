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
            line_width: 100, // Если строка длиннее - сносим на новые строчки
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

pub trait Formatter: Send + Sync {
    fn format_document(
        &self,
        nodes: &[AstNode],
        options: &FormattingOptions,
    ) -> Result<String, AamlError>;

    fn format_range(
        &self,
        nodes: &[AstNode],
        range: FormatRange,
        options: &FormattingOptions,
    ) -> Result<String, AamlError>;

    fn format_node(
        &self,
        node: &AstNode,
        indent_level: usize,
        options: &FormattingOptions,
    ) -> Result<String, AamlError>;

    fn normalize_comments(
        &self,
        content: &str,
        options: &FormattingOptions,
    ) -> Result<String, AamlError>;

    fn normalize_whitespace(&self, content: &str) -> Result<String, AamlError>;
}

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

    /// Checks if the directive should be hoisted to the top (imports, derives)
    fn is_hoistable(node: &AstNode) -> bool {
        if let AstNode::Directive { name, .. } = node {
            matches!(name.as_ref(), "import" | "derive")
        } else {
            false
        }
    }

    /// Жестко контролируем пробелы вокруг знака равенства.
    fn format_assignment(
        key: &str,
        value: &str,
        indent_level: usize,
        options: &FormattingOptions,
    ) -> String {
        let indent = Self::create_indent(indent_level, options);
        // Trim just in case the raw value/key carries garbage whitespace
        format!("{}{} = {}", indent, key.trim(), value.trim())
    }

    /// Умное форматирование алиасов типов (@type name = alias)
    fn format_type_alias(args: &str, indent_level: usize, options: &FormattingOptions) -> String {
        let indent = Self::create_indent(indent_level, options);
        if let Some((name, alias)) = args.split_once('=') {
            format!("{}@type {} = {}", indent, name.trim(), alias.trim())
        } else {
            format!("{}@type {}", indent, args.trim())
        }
    }

    /// Разворачивает или схлопывает @schema в зависимости от line_width
    fn format_schema(args: &str, indent_level: usize, options: &FormattingOptions) -> String {
        let indent = Self::create_indent(indent_level, options);

        // Пытаемся вытащить имя схемы и её тело (между { })
        let Some((name_part, body_part)) = args.split_once('{') else {
            // Если схема пустая или кривая, фоллбечимся в обычную директиву
            return Self::format_directive("schema", args, indent_level, options);
        };

        let schema_name = name_part.trim();
        let body = body_part.trim_end_matches('}').trim();

        // Парсим пары ключ-значение
        let pairs: Vec<_> = body
            .split(|c| c == ',' || c == '\n')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let mut formatted_pairs = Vec::new();
        for pair in pairs {
            if let Some((k, v)) = pair.split_once(':') {
                formatted_pairs.push(format!("{}: {}", k.trim(), v.trim()));
            } else {
                formatted_pairs.push(pair.to_string());
            }
        }

        // Пробуем запихнуть всё в одну строчку
        let single_line = format!(
            "{}@schema {} {{ {} }}",
            indent,
            schema_name,
            formatted_pairs.join(", ")
        );

        // Если строка влезает в лимиты (или лимит отключен), возвращаем её
        if options.line_width == 0 || single_line.len() <= options.line_width {
            return single_line;
        }

        // Если слишком жирная — разворачиваем (glow-up time)
        let inner_indent = Self::create_indent(indent_level + 1, options);
        let mut lines = vec![format!("{}@schema {} {{", indent, schema_name)];

        for pair in formatted_pairs {
            lines.push(format!("{}{}", inner_indent, pair));
        }
        lines.push(format!("{}}}", indent));

        lines.join("\n")
    }

    fn format_directive(
        name: &str,
        args: &str,
        indent_level: usize,
        options: &FormattingOptions,
    ) -> String {
        let indent = Self::create_indent(indent_level, options);
        if args.trim().is_empty() {
            format!("{}@{}", indent, name.trim())
        } else {
            format!("{}@{} {}", indent, name.trim(), args.trim())
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
        let (hoistable, others): (Vec<_>, Vec<_>) =
            nodes.iter().partition(|n| Self::is_hoistable(n));

        let mut header: Vec<String> = hoistable
            .into_iter()
            .map(|n| self.format_node(n, 0, options))
            .collect::<Result<_, _>>()?;

        let body: Vec<String> = others
            .into_iter()
            .map(|n| self.format_node(n, 0, options))
            .collect::<Result<_, _>>()?;

        // Добавляем пустую строку-разделитель, если оба блока не пусты
        if !header.is_empty() && !body.is_empty() && options.preserve_blank_lines {
            header.push(String::new());
        }

        let mut result = [header, body].concat().join("\n");

        if options.trailing_newline && !result.is_empty() && !result.ends_with('\n') {
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
            let line = node.line(); // Предполагаем, что у AstNode есть метод .line()
            if line >= range.start_line && line <= range.end_line {
                let formatted = self.format_node(node, 0, options)?;
                output.push(formatted);
            } else {
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
                // Умный матчинг по имени директивы
                match name.as_ref() {
                    "schema" => Self::format_schema(args, indent_level, options),
                    "type" => Self::format_type_alias(args, indent_level, options),
                    _ => Self::format_directive(name.as_ref(), args, indent_level, options),
                }
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
                if let Some(pos) = line.find('#') {
                    let before = &line[..pos];
                    let after = &line[pos + 1..];

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
