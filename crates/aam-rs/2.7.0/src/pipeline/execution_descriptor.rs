//! Enhanced `ExecutionDescriptor` serving as the comprehensive execution manifest.
//!
//! The `ExecutionDescriptor` replaces direct AAML struct dependency in the Executer.
//! It bundles all necessary context, tasks, and metadata for clean execution.

use crate::pipeline::parser::AstNode;
use crate::pipeline::tasks::{ExecutionStats, ExecutionTask, ParseTask, ValidationTask};
use crate::pipeline::{PipelineBuildHasher, PipelineHashMap};
use smol_str::SmolStr;
use std::collections::HashSet;

#[inline]
fn new_pipeline_map<K, V>() -> PipelineHashMap<K, V> {
    PipelineHashMap::with_hasher(PipelineBuildHasher::default())
}

/// A comprehensive execution manifest that completely replaces AAML struct usage.
///
/// `ExecutionDescriptor` aggregates all necessary information for the Executer to
/// materialize the configuration without requiring the legacy AAML struct.
#[derive(Debug)]
pub struct ExecutionDescriptor<'a> {
    /// Original line numbers from source, indexed by AST node
    pub line_numbers: Vec<usize>,

    /// Execution context containing parsed configuration data
    pub context: ExecutionContext<'a>,

    /// Original parsed AST nodes for reference and diagnostics
    pub parsed_outputs: Vec<AstNode<'a>>,

    /// Lazy tasks for the parsing phase
    pub parse_tasks: Vec<ParseTask<'a>>,

    /// Lazy tasks for the validation phase
    pub validation_tasks: Vec<ValidationTask<'a>>,

    /// Lazy tasks for the execution phase
    pub execution_tasks: Vec<ExecutionTask<'a>>,

    /// Execution statistics
    pub stats: ExecutionStats,
}

/// Encapsulates all runtime context needed during execution.
///
/// This struct holds the accumulated state that would normally be scattered
/// across the AAML struct and various registries.
#[derive(Debug)]
pub struct ExecutionContext<'a> {
    /// Source file path or identifier (for error reporting)
    pub source: std::borrow::Cow<'a, str>,

    /// Key-value map accumulated during parsing (using `SmolStr` for SSO)
    pub map: PipelineHashMap<SmolStr, SmolStr>,

    /// Schema definitions accumulated from @schema directives
    pub schemas: PipelineHashMap<SmolStr, SchemaInfo>,

    /// Type definitions accumulated from @type directives
    pub types: PipelineHashMap<SmolStr, TypeInfo>,

    /// Registered commands (directives)
    pub commands: PipelineHashMap<std::borrow::Cow<'a, str>, CommandInfo<'a>>,

    /// Line number map: key → line number where it was defined
    pub key_line_map: PipelineHashMap<SmolStr, usize>,

    /// Scope tracking for nested configurations
    pub scope_stack: Vec<std::borrow::Cow<'a, str>>,

    /// Circular reference detection set
    pub visited_keys: HashSet<SmolStr>,

    /// Import cache to prevent re-importing the same file
    pub imported_files: HashSet<std::borrow::Cow<'a, str>>,
}

/// Information about a registered schema.
#[derive(Debug, Clone)]
pub struct SchemaInfo {
    /// Schema name
    pub name: SmolStr,

    /// Field name → (`type_name`, `is_optional`)
    pub fields: PipelineHashMap<SmolStr, (SmolStr, bool)>,

    /// Line number where schema was defined
    pub line: usize,
}

/// Information about a registered type.
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// Type name
    pub name: SmolStr,

    /// Type specification (e.g., "i32", "list<string>", "vector2")
    pub spec: SmolStr,

    /// Custom validation rules (if any)
    pub validator: Option<SmolStr>,

    /// Default value used by inheritance when a required field is missing.
    pub default_value: Option<SmolStr>,

    /// Optional metadata bag for pipeline/type adapters.
    pub metadata: PipelineHashMap<SmolStr, SmolStr>,

    /// Line number where type was defined
    pub line: usize,
}

/// Information about a registered command (directive).
#[derive(Debug, Clone)]
pub struct CommandInfo<'a> {
    /// Command name
    pub name: std::borrow::Cow<'a, str>,

    /// Expected argument pattern
    pub arg_pattern: std::borrow::Cow<'a, str>,

    /// Line number where command was registered
    pub line: usize,
}

impl<'a> ExecutionContext<'a> {
    /// Creates a new empty execution context.
    pub fn new(source: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        let mut context = Self {
            source: source.into(),
            map: new_pipeline_map(),
            schemas: new_pipeline_map(),
            types: new_pipeline_map(),
            commands: new_pipeline_map(),
            key_line_map: new_pipeline_map(),
            scope_stack: vec!["root".into()],
            visited_keys: HashSet::default(),
            imported_files: HashSet::default(),
        };

        context.register_builtin_type_defaults();
        context
    }

    fn register_builtin_type_defaults(&mut self) {
        for (name, default_value) in [
            ("i32", "0"),
            ("f64", "0"),
            ("bool", "false"),
            ("string", "\"\""),
            ("color", "#000000"),
        ] {
            self.types.insert(
                name.into(),
                TypeInfo {
                    name: name.into(),
                    spec: name.into(),
                    validator: None,
                    default_value: Some(default_value.into()),
                    metadata: new_pipeline_map(),
                    line: 0,
                },
            );
        }
    }

    /// Returns the registered inheritance default for a given type.
    #[must_use]
    pub fn default_value_for_type(&self, type_name: &str) -> Option<&str> {
        self.types
            .get(type_name)
            .and_then(|info| info.default_value.as_deref())
    }

    /// Returns the current scope as a string path.
    #[must_use]
    pub fn current_scope(&self) -> String {
        self.scope_stack.join("::")
    }

    /// Enters a new nested scope.
    pub fn push_scope(&mut self, scope_name: impl Into<std::borrow::Cow<'a, str>>) {
        self.scope_stack.push(scope_name.into());
    }

    /// Exits the current scope.
    pub fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    /// Sets a key-value pair in the map with line tracking.
    pub fn set_value(&mut self, key: impl AsRef<str>, value: impl AsRef<str>, line: usize) {
        let key_str = key.as_ref();
        let key_smol: SmolStr = key_str.into();
        self.map.insert(key_smol.clone(), value.as_ref().into());
        self.key_line_map.insert(key_smol, line);
    }

    /// Gets a value from the map.
    #[must_use]
    pub fn get_value(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(std::convert::AsRef::as_ref)
    }

    /// Registers a schema definition.
    pub fn register_schema(&mut self, schema: SchemaInfo) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// Registers a type definition.
    pub fn register_type(&mut self, type_def: TypeInfo) {
        self.types.insert(type_def.name.clone(), type_def);
    }

    /// Registers a command (directive).
    pub fn register_command(&mut self, command: CommandInfo<'a>) {
        self.commands.insert(command.name.clone(), command);
    }

    /// Checks if a key has been visited (for cycle detection).
    pub fn mark_visited(&mut self, key: &str) {
        self.visited_keys.insert(key.into());
    }

    /// Checks if a key has been visited.
    #[must_use]
    pub fn is_visited(&self, key: &str) -> bool {
        self.visited_keys.contains(key)
    }

    /// Clears visited set for a new traversal.
    pub fn reset_visited(&mut self) {
        self.visited_keys.clear();
    }

    /// Records an imported file.
    pub fn record_import(&mut self, file_path: impl Into<std::borrow::Cow<'a, str>>) {
        self.imported_files.insert(file_path.into());
    }

    /// Checks if a file has already been imported.
    #[must_use]
    pub fn is_imported(&self, file_path: &str) -> bool {
        self.imported_files.contains(file_path)
    }

    /// Returns the line number where a key was defined.
    #[must_use]
    pub fn get_line_for_key(&self, key: &str) -> Option<usize> {
        self.key_line_map.get(key).copied()
    }
}

impl<'a> ExecutionDescriptor<'a> {
    /// Creates a new execution descriptor from parsed AST.
    pub fn new(
        parsed_outputs: Vec<AstNode<'a>>,
        source: impl Into<std::borrow::Cow<'a, str>>,
    ) -> Self {
        let line_numbers = parsed_outputs.iter().map(AstNode::line).collect();

        Self {
            line_numbers,
            context: ExecutionContext::new(source),
            parsed_outputs,
            parse_tasks: Vec::new(),
            validation_tasks: Vec::new(),
            execution_tasks: Vec::new(),
            stats: ExecutionStats::default(),
        }
    }

    /// Adds a parse task to the manifest.
    pub fn add_parse_task(&mut self, task: ParseTask<'a>) {
        self.parse_tasks.push(task);
    }

    /// Adds multiple parse tasks.
    pub fn add_parse_tasks(&mut self, tasks: Vec<ParseTask<'a>>) {
        self.parse_tasks.extend(tasks);
    }

    /// Adds a validation task.
    pub fn add_validation_task(&mut self, task: ValidationTask<'a>) {
        self.validation_tasks.push(task);
    }

    /// Adds multiple validation tasks.
    pub fn add_validation_tasks(&mut self, tasks: Vec<ValidationTask<'a>>) {
        self.validation_tasks.extend(tasks);
    }

    /// Adds an execution task.
    pub fn add_execution_task(&mut self, task: ExecutionTask<'a>) {
        self.execution_tasks.push(task);
    }

    /// Adds multiple execution tasks.
    pub fn add_execution_tasks(&mut self, tasks: Vec<ExecutionTask<'a>>) {
        self.execution_tasks.extend(tasks);
    }

    /// Returns the total number of tasks.
    #[must_use]
    pub const fn task_count(&self) -> usize {
        self.parse_tasks.len() + self.validation_tasks.len() + self.execution_tasks.len()
    }

    /// Returns a summary of tasks by type.
    #[must_use]
    pub fn task_summary(&self) -> String {
        format!(
            "Parse tasks: {}, Validation tasks: {}, Execution tasks: {}",
            self.parse_tasks.len(),
            self.validation_tasks.len(),
            self.execution_tasks.len()
        )
    }

    /// Updates execution statistics.
    pub const fn update_stats(&mut self, stats: ExecutionStats) {
        self.stats = stats;
    }

    /// Retrieves the source identifier.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.context.source
    }

    /// Returns a mutable reference to the execution context.
    pub const fn context_mut(&mut self) -> &mut ExecutionContext<'a> {
        &mut self.context
    }

    /// Returns an immutable reference to the execution context.
    #[must_use]
    pub const fn context(&self) -> &ExecutionContext<'a> {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_scope_management() {
        let mut ctx = ExecutionContext::new("test.aam".to_string());
        assert_eq!(ctx.current_scope(), "root");

        ctx.push_scope("section".to_string());
        assert_eq!(ctx.current_scope(), "root::section");

        ctx.pop_scope();
        assert_eq!(ctx.current_scope(), "root");
    }

    #[test]
    fn test_execution_context_key_value_operations() {
        let mut ctx = ExecutionContext::new("test.aam".to_string());
        ctx.set_value("key1", "value1", 1);

        assert_eq!(ctx.get_value("key1"), Some("value1"));
        assert_eq!(ctx.get_line_for_key("key1"), Some(1));
    }

    #[test]
    fn test_visited_keys_tracking() {
        let mut ctx = ExecutionContext::new("test.aam".to_string());
        assert!(!ctx.is_visited("key1"));

        ctx.mark_visited("key1");
        assert!(ctx.is_visited("key1"));

        ctx.reset_visited();
        assert!(!ctx.is_visited("key1"));
    }

    #[test]
    fn test_execution_descriptor_task_management() {
        let mut desc = ExecutionDescriptor::new(vec![], "test.aam".to_string());

        desc.add_validation_task(ValidationTask::VerifySchemaExists {
            schema_name: "MySchema".to_string().into(),
            line: 1,
        });

        assert_eq!(desc.validation_tasks.len(), 1);
        assert_eq!(desc.task_count(), 1);
    }
}
