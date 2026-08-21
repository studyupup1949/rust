//! Validator stage: generates validation tasks from AST.
//!
//! The Validator analyzes AST nodes and generates declarative `ValidationTask` items.
//! These tasks are later executed by `ValidateExecutor`, enabling lazy evaluation,
//! error aggregation (critical for LSP), and potential parallel execution.

use crate::error::{AamlError, ErrorDiagnostics};
use crate::pipeline::parser::{AstNode, ValueNode};
use crate::pipeline::tasks::ValidationTask;

/// Trait for task-based semantic validation.
///
/// Instead of directly validating, the Validator generates `ValidationTask` items.
/// This enables deferred execution and better error handling for LSP integration.
pub trait Validator: Send + Sync {
    /// Analyzes an AST and generates validation tasks.
    ///
    /// This method performs static analysis on the AST and produces a list of
    /// validation tasks that will be executed later by a `ValidateExecutor`.
    /// It does NOT mutate state or execute validations directly.
    ///
    /// # Arguments
    /// - `ast`: AST nodes to analyze
    ///
    /// # Returns
    /// - `Ok(Vec<ValidationTask>)` containing all validation tasks to be executed
    /// - `Err(AamlError)` if AST analysis itself fails (e.g., syntax errors)
    fn validate<'a>(&self, ast: &'a [AstNode<'a>]) -> Result<Vec<ValidationTask<'a>>, AamlError>;

    /// Performs quick syntactic checks that don't require deferred validation.
    ///
    /// This is called immediately during parsing to catch obvious issues early.
    fn check_syntax(&self, ast: &[AstNode<'_>]) -> Result<(), AamlError>;
}

/// Default implementation of the Validator stage.
///
/// This validator performs syntactic checks immediately and generates
/// semantic validation tasks for deferred execution.
pub struct DefaultValidator;

impl DefaultValidator {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn infer_literal_type(value: &str) -> &'static str {
        if value == "true" || value == "false" {
            "bool"
        } else if value.parse::<f64>().is_ok() {
            "f64"
        } else {
            "string"
        }
    }

    fn infer_value_type(value: &ValueNode<'_>) -> std::borrow::Cow<'static, str> {
        match value {
            ValueNode::Literal(literal) => Self::infer_literal_type(literal.as_ref()).into(),
            ValueNode::Object(_) => "object".into(),
            ValueNode::List(items) => {
                let inner = Self::infer_list_element_type(items);
                if inner == "any" {
                    "list".into()
                } else {
                    format!("list<{inner}>").into()
                }
            }
        }
    }

    fn infer_list_element_type(items: &[ValueNode<'_>]) -> std::borrow::Cow<'static, str> {
        let mut inferred: Option<std::borrow::Cow<'static, str>> = None;
        for item in items {
            let current = Self::infer_value_type(item);
            match &inferred {
                None => inferred = Some(current),
                Some(existing) if *existing == current => {}
                _ => return "any".into(),
            }
        }

        inferred.unwrap_or_else(|| "any".into())
    }

    fn build_value_tasks<'a>(
        tasks: &mut Vec<ValidationTask<'a>>,
        key: &str,
        value: &'a ValueNode<'a>,
        line: usize,
    ) {
        match value {
            ValueNode::Literal(_s) => {
                // Literal type checks are deferred until execution when schema/type context is available.
            }
            ValueNode::Object(pairs) => {
                tasks.push(ValidationTask::ValidateObjectStructure {
                    key: key.to_string().into(),
                    pairs: pairs.clone(),
                    line,
                });
            }
            ValueNode::List(items) => {
                let inferred_type = Self::infer_list_element_type(items);
                tasks.push(ValidationTask::ValidateListElements {
                    key: key.to_string().into(),
                    items: items.clone(),
                    element_type: inferred_type,
                    line,
                });
            }
        }
    }

    /// Generates validation tasks for an assignment node.
    fn generate_assignment_tasks<'a>(
        key: &std::borrow::Cow<'a, str>,
        value: &'a ValueNode<'a>,
        line: usize,
    ) -> Vec<ValidationTask<'a>> {
        let mut tasks = Vec::new();
        Self::build_value_tasks(&mut tasks, key, value, line);

        // Check for circular references
        tasks.push(ValidationTask::CheckNoCircularReference {
            key: key.to_string().into(),
            line,
        });

        tasks
    }

    /// Generates validation tasks for a directive node.
    fn generate_directive_tasks<'a>(
        name: &std::borrow::Cow<'a, str>,
        args: std::borrow::Cow<'a, str>,
        line: usize,
    ) -> Vec<ValidationTask<'a>> {
        if name.as_ref() == "derive" {
            return vec![ValidationTask::CheckDeriveCompleteness {
                derive_path: args,
                current_key: std::borrow::Cow::Borrowed(""),
                line,
            }];
        }

        // Directives are validated and executed in the execution phase.
        // The validator stage only needs to add extra checks for derive.
        Vec::new()
    }
}

impl Default for DefaultValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for DefaultValidator {
    fn validate<'a>(&self, ast: &'a [AstNode<'a>]) -> Result<Vec<ValidationTask<'a>>, AamlError> {
        // First, perform syntactic checks
        self.check_syntax(ast)?;

        // Now generate validation tasks for deferred execution
        let mut tasks = Vec::new();

        for node in ast {
            match node {
                AstNode::Assignment { key, value, line } => {
                    let node_tasks = Self::generate_assignment_tasks(key, value, *line);
                    tasks.extend(node_tasks);
                }
                AstNode::Directive {
                    name,
                    args,
                    line,
                    body: _,
                } => {
                    let node_tasks = Self::generate_directive_tasks(name, args.clone(), *line);
                    tasks.extend(node_tasks);
                }
            }
        }

        Ok(tasks)
    }

    fn check_syntax(&self, ast: &[AstNode<'_>]) -> Result<(), AamlError> {
        for node in ast {
            match node {
                AstNode::Assignment { key, value, line } => {
                    if key.is_empty() {
                        return Err(AamlError::ParseError {
                            line: *line,
                            content: format!("= {value}"),
                            details: "Empty key in assignment".to_string(),
                            diagnostics: Some(Box::new(ErrorDiagnostics::new(
                                "Empty key",
                                "Assignment keys must be non-empty".to_string(),
                                "Provide a valid key name".to_string(),
                            ))),
                        });
                    }
                }
                AstNode::Directive { name, line, .. } => {
                    if name.is_empty() {
                        return Err(AamlError::ParseError {
                            line: *line,
                            content: "@".to_string(),
                            details: "Empty directive name".to_string(),
                            diagnostics: None,
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_simple_assignment() {
        let node = AstNode::Assignment {
            key: "host".to_string().into(),
            value: ValueNode::Literal("localhost".to_string().into()),
            line: 1,
        };
        let validator = DefaultValidator::new();
        let nodes = [node];
        let tasks = validator.validate(&nodes).unwrap();
        assert!(!tasks.is_empty()); // Should generate validation tasks
    }

    #[test]
    fn test_validate_empty_key() {
        let node = AstNode::Assignment {
            key: "".to_string().into(),
            value: ValueNode::Literal("value".to_string().into()),
            line: 1,
        };
        let validator = DefaultValidator::new();
        assert!(validator.check_syntax(&[node]).is_err());
    }

    #[test]
    fn test_generate_tasks_for_list_value() {
        let node = AstNode::Assignment {
            key: "items".to_string().into(),
            value: ValueNode::List(
                vec![
                    ValueNode::Literal("a".to_string().into()),
                    ValueNode::Literal("b".to_string().into()),
                ]
                .into(),
            ),
            line: 1,
        };
        let validator = DefaultValidator::new();
        let nodes = [node];
        let tasks = validator.validate(&nodes).unwrap();

        // Should have generated ValidateListElements task among others
        let has_list_task = tasks
            .iter()
            .any(|t| matches!(t, ValidationTask::ValidateListElements { .. }));
        assert!(has_list_task);
    }

    #[test]
    fn test_generate_tasks_for_object_value() {
        let node = AstNode::Assignment {
            key: "config".to_string().into(),
            value: ValueNode::Object(
                vec![(
                    "foo".to_string().into(),
                    ValueNode::Literal("bar".to_string().into()),
                )]
                .into(),
            ),
            line: 1,
        };
        let validator = DefaultValidator::new();
        let nodes = [node];
        let tasks = validator.validate(&nodes).unwrap();

        // Should have generated ValidateObjectStructure task among others
        let has_object_task = tasks
            .iter()
            .any(|t| matches!(t, ValidationTask::ValidateObjectStructure { .. }));
        assert!(has_object_task);
    }
}
