//! Executor trait implementations for task-based validation and parsing.
//!
//! These executors consume declarative tasks and perform the actual execution,
//! completely decoupled from AAML struct manipulation.

use crate::error::{AamlError, ErrorDiagnostics};
use crate::pipeline::PipelineHashMap;
use crate::pipeline::execution_descriptor::ExecutionContext;
use crate::pipeline::lexer::Lexer;
use crate::pipeline::new_pipeline_hash_map;
use crate::pipeline::parser::Parser;
use crate::pipeline::tasks::{ParseTask, TaskError, TaskExecutionResult, ValidationTask};
use bumpalo::Bump;
use smol_str::SmolStr;

/// Trait for executing validation tasks.
///
/// A ValidateExecutor takes a stream of ValidationTasks and executes them,
/// returning aggregated results suitable for LSP integration.
pub trait ValidateExecutor: Send + Sync {
    /// Executes a single validation task within a given context.
    ///
    /// # Arguments
    /// - `task`: The validation task to execute
    /// - `context`: The current execution context
    ///
    /// # Returns
    /// - `Ok(true)` if validation passed
    /// - `Err(AamlError)` if validation failed
    fn execute_validation(
        &self,
        task: &ValidationTask,
        context: &ExecutionContext,
    ) -> Result<bool, AamlError>;

    /// Executes multiple validation tasks and aggregates results.
    ///
    /// This method is the main entry point for validation execution.
    /// It collects errors from multiple failing tasks rather than stopping
    /// at the first error, which is critical for LSP support.
    ///
    /// # Arguments
    /// - `tasks`: All validation tasks to execute
    /// - `context`: The execution context
    ///
    /// # Returns
    /// - `TaskExecutionResult` containing success status and error collection
    fn execute_batch(
        &self,
        tasks: &[ValidationTask],
        context: &ExecutionContext,
    ) -> TaskExecutionResult {
        let mut errors = Vec::new();

        for task in tasks {
            match self.execute_validation(task, context) {
                Ok(true) => {}
                Ok(false) => {
                    // Task executed but validation failed
                    errors.push(TaskError {
                        line: task.line(),
                        message: format!("Validation failed: {}", task.description()),
                        task_description: task.description(),
                        aaml_error: None,
                    });
                }
                Err(e) => {
                    errors.push(TaskError {
                        line: task.line(),
                        message: format!("Validation error: {}", e),
                        task_description: task.description(),
                        aaml_error: Some(format!("{:?}", e)),
                    });
                }
            }
        }

        TaskExecutionResult {
            success: errors.is_empty(),
            errors,
            stats: Default::default(),
        }
    }
}

/// Trait for executing parsing tasks.
///
/// A ParserExecutor takes parse tasks and performs parsing operations,
/// managing scopes, variables, and directive registration.
pub trait ParserExecutor: Send + Sync {
    /// Executes a single parsing task.
    ///
    /// # Arguments
    /// - `task`: The parsing task to execute
    /// - `context`: Mutable execution context (may be modified)
    ///
    /// # Returns
    /// - `Ok(())` if parsing succeeded
    /// - `Err(AamlError)` if parsing failed
    fn execute_parse<'a>(
        &self,
        task: &ParseTask<'_>,
        arena: &'a Bump,
        context: &mut ExecutionContext<'a>,
    ) -> Result<(), AamlError>;

    /// Executes multiple parsing tasks in sequence.
    ///
    /// # Arguments
    /// - `tasks`: All parsing tasks to execute
    /// - `context`: Mutable execution context
    ///
    /// # Returns
    /// - `TaskExecutionResult` with aggregated results
    fn execute_batch<'a>(
        &self,
        tasks: &[ParseTask<'_>],
        arena: &'a Bump,
        context: &mut ExecutionContext<'a>,
    ) -> TaskExecutionResult {
        let mut errors = Vec::new();

        for task in tasks {
            match self.execute_parse(task, arena, context) {
                Ok(()) => {}
                Err(e) => {
                    errors.push(TaskError {
                        line: task.line(),
                        message: format!("Parse error: {}", e),
                        task_description: task.description(),
                        aaml_error: Some(format!("{:?}", e)),
                    });
                }
            }
        }

        TaskExecutionResult {
            success: errors.is_empty(),
            errors,
            stats: Default::default(),
        }
    }
}

/// Default implementation of ValidateExecutor.
///
/// This provides basic validation task execution with type checking and schema verification.
pub struct DefaultValidateExecutor {
    // Can hold shared registries or validation strategies
}

impl DefaultValidateExecutor {
    pub fn new() -> Self {
        Self {}
    }

    /// Checks if a type exists in the context's type registry.
    fn type_exists(&self, context: &ExecutionContext, type_name: &str) -> bool {
        context.types.contains_key(type_name) || Self::is_builtin_type(type_name)
    }

    /// Checks if a built-in type is recognized.
    fn is_builtin_type(type_name: &str) -> bool {
        matches!(type_name, "string" | "i32" | "f64" | "bool" | "color")
    }

    /// Validates a value against a type (basic implementation).
    fn validate_type_value(
        &self,
        value: &str,
        type_name: &str,
        context: &ExecutionContext,
    ) -> Result<(), AamlError> {
        crate::pipeline::utils::validate_type_value(value, type_name, context)
    }

    fn validate_type_match(
        &self,
        key: &std::borrow::Cow<'_, str>,
        value: &std::borrow::Cow<'_, str>,
        type_name: &std::borrow::Cow<'_, str>,
        context: &ExecutionContext,
    ) -> Result<bool, AamlError> {
        if !self.type_exists(context, type_name) {
            return Err(AamlError::InvalidType {
                type_name: type_name.to_string(),
                details: format!("Type not found in registry for key '{}'", key),
                provided: value.to_string(),
                diagnostics: Some(ErrorDiagnostics::new(
                    "Unknown type",
                    format!("Type '{}' is not registered", type_name),
                    "Register the type using @type directive".to_string(),
                )),
            });
        }

        if let Err(e) = self.validate_type_value(value, type_name, context) {
            return Err(AamlError::InvalidType {
                type_name: type_name.to_string(),
                details: format!("Validation failed for key '{}'", key),
                provided: value.to_string(),
                diagnostics: Some(ErrorDiagnostics::new(
                    "Type validation failed",
                    e.to_string(),
                    format!("Ensure '{}' conforms to type '{}'", value, type_name),
                )),
            });
        }

        Ok(true)
    }

    fn verify_schema_exists(
        &self,
        schema_name: &std::borrow::Cow<'_, str>,
        context: &ExecutionContext,
    ) -> Result<bool, AamlError> {
        if context.schemas.contains_key(schema_name.as_ref()) {
            return Ok(true);
        }

        Err(AamlError::NotFound {
            key: schema_name.to_string(),
            context: "Schema not found in registry".to_string(),
            diagnostics: Some(ErrorDiagnostics::new(
                "Schema not defined",
                format!("Schema '{}' referenced but not defined", schema_name),
                "Define it using @schema directive".to_string(),
            )),
        })
    }

    fn verify_file_exists(&self, path: &std::borrow::Cow<'_, str>) -> Result<bool, AamlError> {
        if std::path::Path::new(path.as_ref()).exists() {
            return Ok(true);
        }

        Err(AamlError::IoError {
            details: format!("Imported file '{}' not found", path),
            diagnostics: Some(ErrorDiagnostics::new(
                "File missing",
                format!("The file '{}' does not exist.", path),
                "Check the file path in your import directive.",
            )),
        })
    }

    fn check_no_circular_reference(
        &self,
        key: &std::borrow::Cow<'_, str>,
        context: &ExecutionContext,
    ) -> Result<bool, AamlError> {
        let mut current_key: &str = key;
        let mut visited = std::collections::HashSet::new();

        while let Some(next_val) = context.map.get(current_key) {
            if !visited.insert(current_key) {
                return Err(AamlError::CircularDependency {
                    path: format!("{} -> {}", key, next_val),
                    diagnostics: Some(ErrorDiagnostics::new(
                        "Circular reference detected",
                        format!("Key '{}' references itself directly or indirectly", key),
                        "Break the reference loop".to_string(),
                    )),
                });
            }

            if context.map.contains_key(&**next_val) {
                current_key = next_val;
            } else {
                break;
            }
        }

        Ok(true)
    }

    #[inline]
    fn derive_schema_names(derive_path: &str) -> impl Iterator<Item = &str> {
        derive_path.split("::").skip(1)
    }

    #[inline]
    fn derived_required_key(current_key: &str, field: &str) -> String {
        if current_key.is_empty() {
            field.to_string()
        } else {
            format!("{}.{}", current_key, field)
        }
    }

    fn ensure_derived_schema_complete(
        &self,
        schema_name: &str,
        current_key: &str,
        context: &ExecutionContext,
    ) -> Result<(), AamlError> {
        let schema = context
            .schemas
            .get(schema_name)
            .ok_or_else(|| AamlError::NotFound {
                key: schema_name.to_string(),
                context: "schema derivation".to_string(),
                diagnostics: Some(ErrorDiagnostics::new(
                    "Schema not defined",
                    format!(
                        "Schema '{}' referenced in derive chain but not defined",
                        schema_name
                    ),
                    "Ensure the file being derived from defines this schema",
                )),
            })?;

        for (field, (type_name, is_optional)) in &schema.fields {
            if *is_optional {
                continue;
            }

            let full_key = Self::derived_required_key(current_key, field);
            if context.map.contains_key(full_key.as_str()) {
                continue;
            }

            return Err(AamlError::SchemaValidationError {
                schema: schema_name.to_string(),
                field: field.to_string(),
                type_name: type_name.to_string(),
                details: format!(
                    "Missing required field '{}' from derived schema '{}'",
                    field, schema_name
                ),
                diagnostics: Some(ErrorDiagnostics::new(
                    "Incomplete derivation",
                    format!("Derived object missing required field: {}", field),
                    "Add the field to satisfy the derived schema",
                )),
            });
        }

        Ok(())
    }

    fn check_derive_completeness(
        &self,
        derive_path: &std::borrow::Cow<'_, str>,
        current_key: &std::borrow::Cow<'_, str>,
        context: &ExecutionContext,
    ) -> Result<bool, AamlError> {
        if !derive_path.contains("::") {
            return Ok(true);
        }

        for schema_name in Self::derive_schema_names(derive_path) {
            self.ensure_derived_schema_complete(schema_name, current_key, context)?;
        }

        Ok(true)
    }

    fn validate_against_schema(
        &self,
        schema_name: &std::borrow::Cow<'_, str>,
        key: &std::borrow::Cow<'_, str>,
        value: &std::borrow::Cow<'_, str>,
        context: &ExecutionContext,
    ) -> Result<bool, AamlError> {
        let schema_info = context.schemas.get(schema_name.as_ref()).ok_or_else(|| {
            AamlError::SchemaValidationError {
                schema: schema_name.to_string(),
                field: key.to_string(),
                type_name: "schema".to_string(),
                details: format!("Schema '{}' not found", schema_name),
                diagnostics: None,
            }
        })?;

        if let Err(e) = crate::pipeline::utils::validate_inline_object_against_schema(
            value,
            schema_info,
            context,
        ) {
            return Err(AamlError::SchemaValidationError {
                schema: schema_name.to_string(),
                field: key.to_string(),
                type_name: "schema".to_string(),
                details: e.to_string(),
                diagnostics: None,
            });
        }

        Ok(true)
    }

    fn check_schema_completeness(
        &self,
        schema_name: &std::borrow::Cow<'_, str>,
        missing_fields: &[std::borrow::Cow<'_, str>],
    ) -> Result<bool, AamlError> {
        if missing_fields.is_empty() {
            return Ok(true);
        }

        let missing_str = missing_fields
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>()
            .join(", ");
        Err(AamlError::SchemaValidationError {
            schema: schema_name.to_string(),
            field: missing_str.clone(),
            type_name: "required".to_string(),
            details: format!(
                "Schema incomplete: missing required fields: {}",
                missing_str
            ),
            diagnostics: None,
        })
    }

    fn validate_list_elements(
        &self,
        key: &std::borrow::Cow<'_, str>,
        items: &[crate::pipeline::parser::ValueNode<'_>],
        element_type: &std::borrow::Cow<'_, str>,
        context: &ExecutionContext,
    ) -> Result<bool, AamlError> {
        if element_type.as_ref() == "any"
            || element_type.as_ref() == "list"
            || element_type.as_ref() == "object"
        {
            return Ok(true);
        }

        for item in items {
            if let Err(e) = self.validate_type_value(&item.to_string(), element_type, context) {
                return Err(AamlError::InvalidType {
                    type_name: element_type.to_string(),
                    details: format!("List element invalid in '{}'", key),
                    provided: item.to_string(),
                    diagnostics: Some(ErrorDiagnostics::new(
                        "List element validation failed",
                        e.to_string(),
                        format!("All elements in list must be of type '{}'", element_type),
                    )),
                });
            }
        }

        Ok(true)
    }

    fn validate_object_structure(
        &self,
        key: &std::borrow::Cow<'_, str>,
        pairs: &[(
            std::borrow::Cow<'_, str>,
            crate::pipeline::parser::ValueNode<'_>,
        )],
    ) -> Result<bool, AamlError> {
        if pairs.is_empty() {
            return Err(AamlError::InvalidValue {
                details: format!("Empty object for key '{}'", key),
                expected: "non-empty object".to_string(),
                diagnostics: None,
            });
        }

        Ok(true)
    }
}

impl Default for DefaultValidateExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidateExecutor for DefaultValidateExecutor {
    fn execute_validation(
        &self,
        task: &ValidationTask,
        context: &ExecutionContext,
    ) -> Result<bool, AamlError> {
        match task {
            ValidationTask::CheckTypeMatch {
                key,
                value,
                type_name,
                line: _,
            } => self.validate_type_match(key, value, type_name, context),
            ValidationTask::VerifySchemaExists {
                schema_name,
                line: _,
            } => self.verify_schema_exists(schema_name, context),
            ValidationTask::VerifyFileExists { path, line: _ } => self.verify_file_exists(path),
            ValidationTask::CheckNoCircularReference { key, line: _ } => {
                self.check_no_circular_reference(key, context)
            }
            ValidationTask::CheckDeriveCompleteness {
                derive_path,
                current_key,
                line: _,
            } => self.check_derive_completeness(derive_path, current_key, context),
            ValidationTask::ValidateAgainstSchema {
                schema_name,
                key,
                value,
                line: _,
            } => self.validate_against_schema(schema_name, key, value, context),
            ValidationTask::CheckSchemaCompleteness {
                schema_name,
                missing_fields,
                line: _,
            } => self.check_schema_completeness(schema_name, missing_fields),
            ValidationTask::ValidateListElements {
                key,
                items,
                element_type,
                line: _,
            } => self.validate_list_elements(key, items, element_type, context),
            ValidationTask::ValidateObjectStructure {
                key,
                pairs,
                line: _,
            } => self.validate_object_structure(key, pairs),
        }
    }
}

/// Default implementation of ParserExecutor.
///
/// This processes parsing tasks like variable registration, scope management,
/// and directive execution.
pub struct DefaultParserExecutor {
    // Can hold shared registries and command handlers
}

impl DefaultParserExecutor {
    pub fn new() -> Self {
        Self {}
    }

    fn process_variable<'a>(
        &self,
        variable_name: &std::borrow::Cow<'_, str>,
        value: &std::borrow::Cow<'_, str>,
        line: usize,
        context: &mut ExecutionContext<'a>,
    ) {
        context.set_value(variable_name.as_ref(), value.as_ref(), line);
    }

    fn manage_scope<'a>(
        &self,
        scope: &std::borrow::Cow<'_, str>,
        is_entry: bool,
        context: &mut ExecutionContext<'a>,
    ) {
        if is_entry {
            context.push_scope(scope.as_ref().to_string());
        } else {
            context.pop_scope();
        }
    }

    fn execute_directive(
        &self,
        directive_name: &std::borrow::Cow<'_, str>,
        arguments: &std::borrow::Cow<'_, str>,
        line: usize,
    ) -> Result<(), AamlError> {
        match directive_name.as_ref() {
            "import" | "derive" => Ok(()),
            _ => Err(AamlError::ParseError {
                line,
                content: format!("@{} {}", directive_name, arguments),
                details: format!("Unknown directive: @{}", directive_name),
                diagnostics: Some(ErrorDiagnostics::new(
                    "Unknown directive",
                    format!("Directive '@{}' is not recognized", directive_name),
                    "Known directives: @import, @derive, @schema, @type",
                )),
            }),
        }
    }

    fn register_type<'a>(
        &self,
        type_name: &std::borrow::Cow<'_, str>,
        type_spec: &std::borrow::Cow<'_, str>,
        line: usize,
        context: &mut ExecutionContext<'a>,
    ) {
        let inferred_default = if type_spec.starts_with("list<") {
            Some("[]".to_string())
        } else {
            None
        };

        context.register_type(crate::pipeline::execution_descriptor::TypeInfo {
            name: type_name.as_ref().into(),
            spec: type_spec.as_ref().into(),
            validator: None,
            default_value: inferred_default.map(Into::into),
            metadata: new_pipeline_hash_map(),
            line,
        });
    }

    fn parse_schema_fields(&self, fields: &str) -> PipelineHashMap<SmolStr, (SmolStr, bool)> {
        let mut schema_fields = new_pipeline_hash_map();

        for field_def in fields.split(',') {
            let parts: Vec<&str> = field_def.trim().split(':').collect();
            if parts.len() != 2 {
                continue;
            }

            let field_name = parts[0].trim().to_string();
            let type_name = parts[1].trim().to_string();
            let is_optional = field_name.ends_with('*');
            let clean_name = if is_optional {
                field_name.trim_end_matches('*').trim().to_string()
            } else {
                field_name
            };

            schema_fields.insert(clean_name.into(), (type_name.into(), is_optional));
        }

        schema_fields
    }

    fn auto_register_list_types<'a>(
        &self,
        schema_fields: &PipelineHashMap<SmolStr, (SmolStr, bool)>,
        line: usize,
        context: &mut ExecutionContext<'a>,
    ) {
        let field_type_names: Vec<SmolStr> = schema_fields
            .values()
            .map(|(type_name, _)| type_name.clone())
            .collect();

        for type_name in field_type_names {
            if !type_name.starts_with("list<") || context.types.contains_key(type_name.as_str()) {
                continue;
            }

            context.register_type(crate::pipeline::execution_descriptor::TypeInfo {
                name: type_name.clone(),
                spec: type_name,
                validator: None,
                default_value: Some("[]".into()),
                metadata: new_pipeline_hash_map(),
                line,
            });
        }
    }

    fn register_schema<'a>(
        &self,
        schema_name: &std::borrow::Cow<'_, str>,
        fields: &str,
        line: usize,
        context: &mut ExecutionContext<'a>,
    ) {
        let schema_fields = self.parse_schema_fields(fields);
        self.auto_register_list_types(&schema_fields, line, context);

        context.register_schema(crate::pipeline::execution_descriptor::SchemaInfo {
            name: schema_name.as_ref().into(),
            fields: schema_fields,
            line,
        });

        context.register_type(crate::pipeline::execution_descriptor::TypeInfo {
            name: schema_name.as_ref().into(),
            spec: "schema".into(),
            validator: None,
            default_value: Some("{}".into()),
            metadata: new_pipeline_hash_map(),
            line,
        });
    }

    fn resolve_derive_import<'a>(
        &self,
        derive_path: &std::borrow::Cow<'_, str>,
        arena: &'a Bump,
        context: &mut ExecutionContext<'a>,
    ) -> Result<(), AamlError> {
        let parts: Vec<&str> = derive_path.split("::").collect();
        if parts.is_empty() {
            return Err(AamlError::DirectiveError {
                directive: "derive".to_string(),
                message: "Empty derive path".to_string(),
                diagnostics: None,
            });
        }

        let file_path = parts[0].to_string();
        if !context.is_imported(&file_path) {
            let content_string =
                std::fs::read_to_string(&file_path).map_err(|e| AamlError::IoError {
                    details: format!("Failed to read imported file '{}': {}", file_path, e),
                    diagnostics: Some(ErrorDiagnostics::new(
                        "Import failed",
                        format!("Could not read file '{}'", file_path),
                        "Check if the file exists and is readable",
                    )),
                })?;

            let lexer = crate::pipeline::lexer::DefaultLexer::new();
            let parser = crate::pipeline::parser::DefaultParser::new();
            let _ctx_guard = crate::error::push_error_render_context(&file_path, &content_string);

            let content = arena.alloc_str(&content_string);

            let tokens = lexer.tokenize(content)?;
            let parse_output = parser.parse_with_recovery(&tokens);
            if let Some(first_error) = parse_output.errors.into_iter().next() {
                return Err(first_error);
            }

            let sub_tasks = parser.generate_parse_tasks(&parse_output.ast);
            for sub_task in sub_tasks {
                if let ParseTask::ProcessVariable { .. } = sub_task {
                    // Do not import variables during derive - only schemas and types
                    continue;
                }
                self.execute_parse(&sub_task, arena, context)?;
            }

            context.record_import(file_path);
        }

        Ok(())
    }

    fn resolve_module_reference<'a>(
        &self,
        module_name: &std::borrow::Cow<'_, str>,
        scope: &std::borrow::Cow<'_, str>,
        context: &ExecutionContext<'a>,
    ) -> Result<(), AamlError> {
        if !context.imported_files.contains(module_name)
            && !context.schemas.contains_key(module_name.as_ref())
        {
            return Err(AamlError::NotFound {
                key: module_name.to_string(),
                context: format!("module reference in scope '{}'", scope),
                diagnostics: Some(ErrorDiagnostics::new(
                    "Module not found",
                    format!(
                        "The module '{}' has not been imported or defined",
                        module_name
                    ),
                    "Check for a missing @import directive",
                )),
            });
        }

        Ok(())
    }
}

impl Default for DefaultParserExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserExecutor for DefaultParserExecutor {
    fn execute_parse<'a>(
        &self,
        task: &ParseTask<'_>,
        arena: &'a Bump,
        context: &mut ExecutionContext<'a>,
    ) -> Result<(), AamlError> {
        match task {
            ParseTask::ProcessVariable {
                variable_name,
                value,
                scope: _,
                line,
            } => {
                self.process_variable(variable_name, value, *line, context);
                Ok(())
            }

            ParseTask::ManageScope {
                scope,
                is_entry,
                line: _,
            } => {
                self.manage_scope(scope, *is_entry, context);
                Ok(())
            }

            ParseTask::ExecuteDirective {
                directive_name,
                arguments,
                line,
            } => self.execute_directive(directive_name, arguments, *line),

            ParseTask::RegisterType {
                type_name,
                type_spec,
                line,
            } => {
                self.register_type(type_name, type_spec, *line, context);
                Ok(())
            }

            ParseTask::RegisterSchema {
                schema_name,
                fields,
                line,
            } => {
                self.register_schema(schema_name, fields, *line, context);
                Ok(())
            }

            ParseTask::ResolveDeriveImport {
                derive_path,
                line: _,
            } => self.resolve_derive_import(derive_path, arena, context),

            ParseTask::ResolveModuleReference {
                module_name,
                scope,
                line: _,
            } => self.resolve_module_reference(module_name, scope, context),
        }
    }
}
