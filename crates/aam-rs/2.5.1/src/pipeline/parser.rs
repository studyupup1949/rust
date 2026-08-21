//! Parser stage: builds an Abstract Syntax Tree from tokens.
//!
//! The Parser consumes tokens from the Lexer and produces an AST,
//! preserving line number information for error diagnostics.

use crate::error::{AamlError, ErrorDiagnostics};
use crate::pipeline::lexer::{Token, TokenKind};
use crate::pipeline::tasks::{ExecutionTask, ParseTask};

/// Represents a value in the AST, supporting nested structures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueNode<'a> {
    Literal(std::borrow::Cow<'a, str>),
    Object(std::sync::Arc<[(std::borrow::Cow<'a, str>, ValueNode<'a>)]>),
    List(std::sync::Arc<[ValueNode<'a>]>),
}

impl<'a> ValueNode<'a> {
    /// Converts the value node back to a string representation
    pub fn to_string(&self) -> String {
        match self {
            ValueNode::Literal(s) => s.to_string(),
            ValueNode::Object(pairs) => {
                let formatted_pairs: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, v.to_string()))
                    .collect();
                format!("{{ {} }}", formatted_pairs.join(", "))
            }
            ValueNode::List(items) => {
                let formatted_items: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                format!("[{}]", formatted_items.join(", "))
            }
        }
    }
}

/// A node in the Abstract Syntax Tree.
#[derive(Debug, Clone)]
pub enum AstNode<'a> {
    /// Key-value assignment: `key = value`
    Assignment {
        key: std::borrow::Cow<'a, str>,
        value: ValueNode<'a>,
        line: usize,
    },
    /// Directive: `@directive_name arguments`
    Directive {
        name: std::borrow::Cow<'a, str>,
        args: std::borrow::Cow<'a, str>,
        body: Option<ValueNode<'a>>,
        line: usize,
    },
}

impl<'a> AstNode<'a> {
    /// Returns the line number where this node appears
    pub fn line(&self) -> usize {
        match self {
            AstNode::Assignment { line, .. } => *line,
            AstNode::Directive { line, .. } => *line,
        }
    }
}

/// Trait for parsing tokens into an AST.
pub trait Parser: Send + Sync {
    /// Parses a token stream into an AST.
    ///
    /// # Errors
    /// Returns `AamlError::ParseError` if the token stream is malformed.
    fn parse<'a>(&self, tokens: &[Token<'a>]) -> Result<Vec<AstNode<'a>>, AamlError>;

    /// Parses with recovery and returns both partial AST and all parse errors.
    fn parse_with_recovery<'a>(&self, tokens: &[Token<'a>]) -> ParseOutput<'a>;

    /// Generates parse tasks from an AST
    fn generate_parse_tasks<'a>(&self, ast: &[AstNode<'a>]) -> Vec<ParseTask<'a>>;

    /// Generates execution tasks from an AST
    fn generate_execution_tasks<'a>(&self, ast: &[AstNode<'a>]) -> Vec<ExecutionTask<'a>>;
}

/// Parser output for LSP use-cases where partial AST plus diagnostics are required.
pub struct ParseOutput<'a> {
    pub ast: Vec<AstNode<'a>>,
    pub errors: Vec<AamlError>,
}

/// Default implementation of the Parser stage.
pub struct DefaultParser;

impl DefaultParser {
    pub fn new() -> Self {
        Self
    }

    /// Filters out comment and newline tokens
    fn filter_tokens<'a, 'b>(tokens: &'b [Token<'a>]) -> Vec<&'b Token<'a>> {
        use crate::pipeline::lexer::TokenKind;
        tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Comment)
            .collect()
    }

    /// Parses assignment tokens: `identifier = value`
    fn parse_assignment<'a>(
        tokens: &[&Token<'a>],
        start: usize,
    ) -> Result<(std::borrow::Cow<'a, str>, ValueNode<'a>, usize), AamlError> {
        use crate::pipeline::lexer::TokenKind;

        if start >= tokens.len() {
            return Err(AamlError::ParseError {
                line: tokens.get(start).map(|t| t.line).unwrap_or(1),
                content: "incomplete assignment".to_string(),
                details: "Expected: key = value".to_string(),
                diagnostics: Some(ErrorDiagnostics::new(
                    "Incomplete assignment",
                    "Assignment must have at least 3 tokens: key, =, value".to_string(),
                    "Check format: key = value".to_string(),
                )),
            });
        }

        if tokens[start].kind != TokenKind::Identifier {
            return Err(AamlError::ParseError {
                line: tokens[start].line,
                content: format!("Expected identifier, got {:?}", tokens[start].kind),
                details: "First token of assignment must be an identifier".to_string(),
                diagnostics: None,
            });
        }

        let mut assign_pos = start + 1;
        while assign_pos < tokens.len()
            && tokens[assign_pos].kind != TokenKind::Assign
            && tokens[assign_pos].kind != TokenKind::Newline
        {
            assign_pos += 1;
        }

        if assign_pos >= tokens.len() || tokens[assign_pos].kind != TokenKind::Assign {
            let got = tokens
                .get(start + 1)
                .map(|t| t.text.as_ref())
                .unwrap_or("<eof>");
            return Err(AamlError::ParseError {
                line: tokens
                    .get(start + 1)
                    .map(|t| t.line)
                    .unwrap_or(tokens[start].line),
                content: format!("Expected '=', got '{}'", got),
                details: "Assignment operator '=' expected after key".to_string(),
                diagnostics: None,
            });
        }

        let key = if assign_pos == start + 1 {
            tokens[start].text.clone()
        } else {
            let key_text = tokens[start..assign_pos]
                .iter()
                .map(|t| t.text.as_ref())
                .collect::<Vec<_>>()
                .join(" ");
            key_text.into()
        };

        let (value, consumed) = Self::parse_value(tokens, assign_pos + 1)?;
        Ok((key, value, consumed))
    }

    fn parse_braced_literal<'a, 'b>(
        tokens: &'b [&'b Token<'a>],
        start: usize,
    ) -> Result<(ValueNode<'a>, usize), AamlError> {
        use crate::pipeline::lexer::TokenKind;

        let mut depth = 0_i32;
        let mut pos = start;

        while pos < tokens.len() {
            match tokens[pos].kind {
                TokenKind::LeftBrace => depth += 1,
                TokenKind::RightBrace => {
                    depth -= 1;
                    if depth == 0 {
                        let text: String = tokens[start..=pos]
                            .iter()
                            .map(|t| t.text.as_ref())
                            .collect();
                        let normalized = text.replace(',', ", ");
                        return Ok((ValueNode::Literal(normalized.into()), pos + 1));
                    }
                }
                TokenKind::Newline if depth == 0 => break,
                _ => {}
            }
            pos += 1;
        }

        Err(AamlError::ParseError {
            line: tokens[start].line,
            content: "unclosed brace".to_string(),
            details: "Expected '}' to close inline object".to_string(),
            diagnostics: None,
        })
    }

    /// Parses a value (which may be literal, inline object, or inline list)
    fn parse_value<'a, 'b>(
        tokens: &'b [&'b Token<'a>],
        start: usize,
    ) -> Result<(ValueNode<'a>, usize), AamlError> {
        use crate::pipeline::lexer::TokenKind;

        if start >= tokens.len() {
            return Err(AamlError::ParseError {
                line: tokens.len(),
                content: "unexpected end of input".to_string(),
                details: "Expected a value after '='".to_string(),
                diagnostics: None,
            });
        }

        match &tokens[start].kind {
            TokenKind::LeftBrace => {
                // Prefer structured inline object; fallback to raw braced literal for nested schema-like values.
                match Self::parse_inline_object(tokens, start) {
                    Ok((obj, consumed)) => Ok((obj, consumed)),
                    Err(_) => Self::parse_braced_literal(tokens, start),
                }
            }
            TokenKind::LeftBracket => {
                // Inline list
                let (list, consumed) = Self::parse_inline_list(tokens, start)?;
                Ok((list, consumed))
            }
            _ => {
                let end_pos = tokens[start..]
                    .iter()
                    .position(|t| {
                        matches!(
                            t.kind,
                            TokenKind::Newline
                                | TokenKind::RightBrace
                                | TokenKind::RightBracket
                                | TokenKind::Comma
                        )
                    })
                    .map(|p| start + p)
                    .unwrap_or(tokens.len());

                if end_pos == start {
                    return Err(AamlError::ParseError {
                        line: tokens[start].line,
                        content: "invalid value".to_string(),
                        details: "Could not parse literal value".to_string(),
                        diagnostics: None,
                    });
                }

                let value_text: String = tokens[start..end_pos]
                    .iter()
                    .map(|t| t.text.as_ref())
                    .collect();

                Ok((ValueNode::Literal(value_text.into()), end_pos))
            }
        }
    }

    /// Parses an inline object: `{ key = val, key = val, ... }`
    fn parse_inline_object<'a, 'b>(
        tokens: &'b [&'b Token<'a>],
        start: usize,
    ) -> Result<(ValueNode<'a>, usize), AamlError> {
        use crate::pipeline::lexer::TokenKind;

        let mut pairs = Vec::new();
        let mut pos = start + 1;
        let mut expect_field = true;
        let mut has_any_field = false;

        while pos < tokens.len() {
            match tokens[pos].kind {
                TokenKind::RightBrace if expect_field && has_any_field => {
                    return Err(AamlError::ParseError {
                        line: tokens[pos].line,
                        content: "trailing comma in inline object".to_string(),
                        details: "Expected another field after ',' or remove trailing comma"
                            .to_string(),
                        diagnostics: None,
                    });
                }
                TokenKind::RightBrace => return Ok((ValueNode::Object(pairs.into()), pos + 1)),
                TokenKind::Identifier if expect_field => {
                    pos = Self::parse_inline_object_pair(tokens, pos, &mut pairs)?;
                    expect_field = false;
                    has_any_field = true;
                }
                TokenKind::Identifier => {
                    return Err(AamlError::ParseError {
                        line: tokens[pos].line,
                        content: "missing comma in inline object".to_string(),
                        details: "Expected ',' between object fields".to_string(),
                        diagnostics: None,
                    });
                }
                TokenKind::Comma if expect_field => {
                    return Err(AamlError::ParseError {
                        line: tokens[pos].line,
                        content: "unexpected comma in inline object".to_string(),
                        details: "Expected an object field before ','".to_string(),
                        diagnostics: None,
                    });
                }
                TokenKind::Comma => {
                    expect_field = true;
                    pos += 1;
                }
                _ => {
                    return Err(AamlError::ParseError {
                        line: tokens[pos].line,
                        content: "invalid inline object format".to_string(),
                        details: "Expected identifier or closing brace".to_string(),
                        diagnostics: None,
                    });
                }
            }
        }

        Err(AamlError::ParseError {
            line: tokens[start].line,
            content: "unclosed brace".to_string(),
            details: "Expected '}' to close inline object".to_string(),
            diagnostics: None,
        })
    }

    fn parse_inline_object_pair<'a, 'b>(
        tokens: &'b [&'b Token<'a>],
        pos: usize,
        pairs: &mut Vec<(std::borrow::Cow<'a, str>, ValueNode<'a>)>,
    ) -> Result<usize, AamlError> {
        use crate::pipeline::lexer::TokenKind;
        let key: std::borrow::Cow<'a, str> = tokens[pos].text.clone();
        if pos + 2 < tokens.len() && tokens[pos + 1].kind == TokenKind::Assign {
            let (value, next_pos) = Self::parse_value(tokens, pos + 2)?;
            pairs.push((key, value));
            Ok(next_pos)
        } else {
            Err(AamlError::ParseError {
                line: tokens[pos].line,
                content: "invalid inline object format".to_string(),
                details: "Expected '=' after key".to_string(),
                diagnostics: None,
            })
        }
    }

    /// Parses an inline list: `[item, item, ...]`
    fn parse_inline_list<'a, 'b>(
        tokens: &'b [&'b Token<'a>],
        start: usize,
    ) -> Result<(ValueNode<'a>, usize), AamlError> {
        use crate::pipeline::lexer::TokenKind;

        let mut items = Vec::new();
        let mut pos = start + 1;

        while pos < tokens.len() {
            if tokens[pos].kind == TokenKind::RightBracket {
                return Ok((ValueNode::List(items.into()), pos + 1));
            }

            if tokens[pos].kind == TokenKind::Comma {
                pos += 1;
                continue;
            }

            let (value, next_pos) = Self::parse_value(tokens, pos)?;
            items.push(value);
            pos = next_pos;
        }

        Err(AamlError::ParseError {
            line: tokens[start].line,
            content: "unclosed bracket".to_string(),
            details: "Expected ']' to close inline list".to_string(),
            diagnostics: None,
        })
    }

    fn missing_directive_name_error(line: usize) -> AamlError {
        AamlError::ParseError {
            line,
            content: "@".to_string(),
            details: "Directive name expected after '@'".to_string(),
            diagnostics: Some(ErrorDiagnostics::new(
                "Missing directive name",
                "Directive requires a name after '@'".to_string(),
                "Use format: @directive_name arguments".to_string(),
            )),
        }
    }

    fn should_stop_directive_args(kind: &TokenKind, brace_count: i32, bracket_count: i32) -> bool {
        brace_count == 0
            && bracket_count == 0
            && (*kind == TokenKind::At || *kind == TokenKind::Newline)
    }

    fn collect_directive_args<'a>(
        tokens_filtered: &[&Token<'a>],
        start_pos: usize,
    ) -> (String, usize) {
        let mut args = String::new();
        let mut arg_pos = start_pos;
        let mut brace_count = 0;
        let mut bracket_count = 0;

        while arg_pos < tokens_filtered.len() {
            let tk = tokens_filtered[arg_pos];
            match tk.kind {
                TokenKind::LeftBrace => brace_count += 1,
                TokenKind::RightBrace => brace_count -= 1,
                TokenKind::LeftBracket => bracket_count += 1,
                TokenKind::RightBracket => bracket_count -= 1,
                _ => {}
            }

            if Self::should_stop_directive_args(&tk.kind, brace_count, bracket_count) {
                break;
            }

            if !args.is_empty() {
                args.push(' ');
            }
            args.push_str(&tk.text);
            arg_pos += 1;
        }

        (args, arg_pos)
    }

    fn validate_schema_directive_args(line: usize, trimmed_args: &str) -> Result<(), AamlError> {
        let (name_part, body_opt) = trimmed_args.split_once('{').unwrap_or((trimmed_args, ""));
        if name_part.trim().is_empty() {
            return Err(AamlError::ParseError {
                line,
                content: "@schema".to_string(),
                details: "Schema name is missing".to_string(),
                diagnostics: None,
            });
        }

        if !trimmed_args.contains('{') || !trimmed_args.ends_with('}') {
            return Err(AamlError::ParseError {
                line,
                content: "@schema".to_string(),
                details: "Schema block must end with '}'".to_string(),
                diagnostics: None,
            });
        }

        let body = body_opt.trim_end_matches('}').trim();
        for field in body.split(|c| c == ',' || c == '\n') {
            let field = field.trim();
            if field.is_empty() {
                continue;
            }

            if !field.contains(':') {
                return Err(AamlError::ParseError {
                    line,
                    content: "invalid schema field".to_string(),
                    details: format!("Field '{}' must be of the form 'name: type'", field),
                    diagnostics: None,
                });
            }
        }

        Ok(())
    }

    fn parse_directive<'a>(
        tokens_filtered: &[&Token<'a>],
        pos: &mut usize,
        line: usize,
    ) -> Result<AstNode<'a>, AamlError> {
        if *pos + 1 >= tokens_filtered.len() {
            return Err(Self::missing_directive_name_error(line));
        }

        let dir_name: std::borrow::Cow<'a, str> = tokens_filtered[*pos + 1].text.clone();
        let (args, arg_pos) = Self::collect_directive_args(tokens_filtered, *pos + 2);

        *pos = arg_pos;

        let trimmed_args = args.trim();
        if &*dir_name == "schema" {
            Self::validate_schema_directive_args(line, trimmed_args)?;
        }

        Ok(AstNode::Directive {
            name: dir_name,
            args: trimmed_args.to_string().into(),
            body: None,
            line,
        })
    }

    fn synchronize(tokens_filtered: &[&Token<'_>], mut pos: usize) -> usize {
        while pos < tokens_filtered.len() {
            let kind = &tokens_filtered[pos].kind;
            if *kind == TokenKind::Newline || *kind == TokenKind::RightBrace {
                return pos + 1;
            }
            pos += 1;
        }
        pos
    }

    fn scope_from_key(key: &str) -> std::borrow::Cow<'static, str> {
        if let Some((prefix, _)) = key.rsplit_once('.') {
            return format!("root::{}", prefix.replace('.', "::")).into();
        }
        std::borrow::Cow::Borrowed("root")
    }

    fn emit_assignment_parse_tasks<'a>(
        tasks: &mut Vec<ParseTask<'a>>,
        key: std::borrow::Cow<'a, str>,
        value: &ValueNode<'a>,
        line: usize,
    ) {
        tasks.push(ParseTask::ProcessVariable {
            variable_name: key.clone(),
            value: value.to_string().into(),
            scope: Self::scope_from_key(&key).into_owned().into(),
            line,
        });

        if let ValueNode::Object(pairs) = value {
            for (field, child_value) in pairs.iter() {
                let child_key = format!("{}.{}", key, field);
                Self::emit_assignment_parse_tasks(tasks, child_key.into(), child_value, line);
            }
        }
    }

    fn build_type_task<'a>(args: &str, line: usize) -> ParseTask<'a> {
        let (type_name, type_spec) = args
            .split_once('=')
            .map(|(lhs, rhs)| (lhs.trim(), rhs.trim()))
            .unwrap_or(("", ""));

        ParseTask::RegisterType {
            type_name: type_name.to_string().into(),
            type_spec: type_spec.to_string().into(),
            line,
        }
    }

    fn build_schema_task<'a>(args: &str, line: usize) -> ParseTask<'a> {
        let schema_name = args.split_whitespace().next().unwrap_or("").to_string();
        let body = args
            .split_once('{')
            .and_then(|(_, b)| b.rsplit_once('}'))
            .map(|(b, _)| b)
            .unwrap_or("");
        let fields = body
            .split(|c: char| c == ',' || c == '\n')
            .filter_map(|field_def| {
                let (field_name, type_name) = field_def.split_once(':')?;
                let field_name = field_name.trim();
                let type_name = type_name.trim();

                if field_name.is_empty() {
                    return None;
                }

                Some(format!("{}:{}", field_name, type_name))
            })
            .collect::<Vec<_>>()
            .join(",");

        ParseTask::RegisterSchema {
            schema_name: schema_name.into(),
            fields: fields.into(),
            line,
        }
    }

    fn build_directive_task<'a>(
        name: &std::borrow::Cow<'a, str>,
        args: &std::borrow::Cow<'a, str>,
        line: usize,
    ) -> ParseTask<'a> {
        match &**name {
            "type" => Self::build_type_task(args, line),
            "schema" => Self::build_schema_task(args, line),
            "derive" => ParseTask::ResolveDeriveImport {
                derive_path: args.clone(),
                line,
            },
            _ => ParseTask::ExecuteDirective {
                directive_name: name.clone(),
                arguments: args.clone(),
                line,
            },
        }
    }
}

impl Default for DefaultParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for DefaultParser {
    fn parse<'a>(&self, tokens: &[Token<'a>]) -> Result<Vec<AstNode<'a>>, AamlError> {
        let result = self.parse_with_recovery(tokens);
        if result.errors.is_empty() {
            Ok(result.ast)
        } else {
            Err(result
                .errors
                .into_iter()
                .next()
                .unwrap_or(AamlError::ParseError {
                    line: 1,
                    content: "parse failed".to_string(),
                    details: "Unknown parser failure".to_string(),
                    diagnostics: None,
                }))
        }
    }

    fn parse_with_recovery<'a>(&self, tokens: &[Token<'a>]) -> ParseOutput<'a> {
        use crate::pipeline::lexer::TokenKind;

        let mut ast: Vec<AstNode<'a>> = Vec::new();
        let mut errors = Vec::new();
        let tokens_filtered = Self::filter_tokens(tokens);
        let mut pos = 0;

        while pos < tokens_filtered.len() {
            let token = tokens_filtered[pos];

            match &token.kind {
                TokenKind::At => {
                    match Self::parse_directive(&tokens_filtered, &mut pos, token.line) {
                        Ok(dir_node) => ast.push(dir_node),
                        Err(err) => {
                            errors.push(err);
                            pos = Self::synchronize(&tokens_filtered, pos + 1);
                        }
                    }
                }
                TokenKind::Identifier => match Self::parse_assignment(&tokens_filtered, pos) {
                    Ok((key, value, new_pos)) => {
                        ast.push(AstNode::Assignment {
                            key,
                            value,
                            line: token.line,
                        });
                        pos = new_pos;
                    }
                    Err(err) => {
                        errors.push(err);
                        pos = Self::synchronize(&tokens_filtered, pos + 1);
                    }
                },
                _ => pos += 1,
            }
        }

        ParseOutput { ast, errors }
    }
    fn generate_parse_tasks<'a>(&self, ast: &[AstNode<'a>]) -> Vec<ParseTask<'a>> {
        let mut tasks = Vec::new();

        for node in ast {
            match node {
                AstNode::Assignment { key, value, line } => {
                    Self::emit_assignment_parse_tasks(&mut tasks, key.clone(), value, *line);
                }
                AstNode::Directive {
                    name, args, line, ..
                } => tasks.push(Self::build_directive_task(name, args, *line)),
            }
        }

        tasks
    }

    fn generate_execution_tasks<'a>(&self, ast: &[AstNode<'a>]) -> Vec<ExecutionTask<'a>> {
        let mut tasks = Vec::new();

        for node in ast {
            match node {
                AstNode::Assignment { key, value, line } => {
                    tasks.push(ExecutionTask::SetValue {
                        key: key.clone(),
                        value: value.to_string().into(),
                        line: *line,
                    });
                }
                // Directives translated to execution tasks...
                AstNode::Directive {
                    name, args, line, ..
                } => {
                    if &**name == "import" {
                        tasks.push(ExecutionTask::ImportFile {
                            file_path: args.clone(),
                            merge_strategy: std::borrow::Cow::Borrowed("merge"),
                            line: *line,
                        });
                    } else if &**name == "derive" {
                        tasks.push(ExecutionTask::ExecuteInheritance {
                            derive_path: args.clone(),
                            child_key: std::borrow::Cow::Borrowed(""), // For AAM v2
                            line: *line,
                        });
                    }
                }
            }
        }
        tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::lexer::{DefaultLexer, Lexer};

    #[test]
    fn test_parse_simple_assignment() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("key = value").unwrap();
        let parser = DefaultParser::new();
        let ast = parser.parse(&*tokens).unwrap();

        assert_eq!(ast.len(), 1);
        match &ast[0] {
            AstNode::Assignment { key, value, .. } => {
                assert_eq!(&**key, "key");
                if let ValueNode::Literal(s) = value {
                    assert_eq!(&**s, "value");
                } else {
                    panic!("Expected ValueNode::Literal");
                }
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_parse_directive() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("@import base.aam").unwrap();
        let parser = DefaultParser::new();
        let ast = parser.parse(&*tokens).unwrap();

        assert_eq!(ast.len(), 1);
        match &ast[0] {
            AstNode::Directive { name, args: _, .. } => {
                assert_eq!(&**name, "import");
            }
            _ => panic!("Expected directive"),
        }
    }

    #[test]
    fn test_parse_multiple_assignments() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("a = b\nc = d").unwrap();
        let parser = DefaultParser::new();
        let ast = parser.parse(&*tokens).unwrap();

        assert_eq!(ast.len(), 2);
    }
}
