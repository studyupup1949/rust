//! Executer stage: materializes validated tasks into final runtime state.
//!
//! The Executer receives an `ExecutionDescriptor` containing pre-computed tasks
//! and executes them in a completely decoupled manner, WITHOUT any AAML struct dependency.
//! This enables clean separation of concerns, better LSP integration, and potential
//! parallel execution in future iterations.

use crate::error::AamlError;
use crate::error::ErrorDiagnostics;
use crate::pipeline::PipelineHashMap;
use crate::pipeline::execution_descriptor::ExecutionDescriptor;
use crate::pipeline::tasks::ExecutionTask;
use crate::pipeline::utils::resolve_relative_path;
use bumpalo::Bump;
use smol_str::SmolStr;
use std::path::Path;

/// Trait for executing the final materialization of configuration state.
///
/// The Executer operates purely on `ExecutionDescriptor` and task queues.
/// It NO LONGER instantiates or depends on the AAML struct.
pub trait Executer: Send + Sync {
    /// Executes the manifest to produce a final `FoundValue`.
    ///
    /// # Arguments
    /// - `manifest`: Comprehensive execution manifest with all tasks and context
    ///
    /// # Returns
    /// - `Ok(FoundValue)` on successful execution
    /// - `Err(AamlError)` if any execution phase fails
    fn execute(&self, manifest: &mut ExecutionDescriptor) -> Result<(), AamlError>;
}

/// Default implementation of the Executer stage.
///
/// This executor handles all execution tasks in a clean, isolated manner:
/// 1. Processes `SetValue` and `MergeValue` tasks to populate the output map
/// 2. Handles `ApplySchema` tasks for schema validation and enforcement
/// 3. Manages `ExecuteInheritance` for configuration inheritance
/// 4. Resolves cross-references via `ResolveReference` tasks
/// 5. Handles `ImportFile` tasks for external configuration merging
pub struct DefaultExecuter {
    // Can hold shared registries or execution strategies
}

impl DefaultExecuter {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    fn set_value(output_map: &mut PipelineHashMap<SmolStr, SmolStr>, key: &str, value: &str) {
        output_map.insert(SmolStr::from(key), SmolStr::from(value));
    }

    fn merge_value(output_map: &mut PipelineHashMap<SmolStr, SmolStr>, key: &str, value: &str) {
        let existing = output_map
            .get(key)
            .map(std::string::ToString::to_string)
            .unwrap_or_default();

        let merged: std::borrow::Cow<'_, str> = if existing.is_empty() {
            value.into()
        } else {
            format!("{existing} {value}").into()
        };

        output_map.insert(SmolStr::from(key), SmolStr::from(merged.as_ref()));
    }

    fn build_full_key(root: &str, field: &str) -> String {
        if root.is_empty() {
            field.to_string()
        } else {
            format!("{root}.{field}")
        }
    }

    fn schema_not_found_error(schema_name: &str) -> AamlError {
        AamlError::NotFound {
            key: schema_name.to_string(),
            context: "schema registry".to_string(),
            diagnostics: Some(Box::new(ErrorDiagnostics::new(
                "Schema not found",
                format!("Schema '{schema_name}' does not exist"),
                "Check your @schema definitions",
            ))),
        }
    }

    fn schema_field_error(
        schema_name: &str,
        field: &str,
        type_name: &str,
        details: String,
    ) -> AamlError {
        AamlError::SchemaValidationError {
            schema: schema_name.to_string(),
            field: field.to_string(),
            type_name: type_name.to_string(),
            details,
            diagnostics: None,
        }
    }

    fn validate_schema_field(
        schema_name: &str,
        field: &str,
        type_name: &str,
        is_optional: bool,
        full_key: &str,
        output_map: &PipelineHashMap<SmolStr, SmolStr>,
        context: &crate::pipeline::execution_descriptor::ExecutionContext,
    ) -> Result<(), AamlError> {
        let Some(value) = output_map.get(full_key) else {
            if is_optional {
                return Ok(());
            }

            return Err(Self::schema_field_error(
                schema_name,
                field,
                type_name,
                format!("Missing required field '{field}'"),
            ));
        };

        if let Err(err_msg) = crate::pipeline::utils::validate_type_value(value, type_name, context)
        {
            return Err(Self::schema_field_error(
                schema_name,
                field,
                type_name,
                format!(
                    "Type mismatch for field '{}': {}",
                    field,
                    err_msg.short_message()
                ),
            ));
        }

        Ok(())
    }

    fn apply_schema(
        schema_name: &str,
        root_keys: &[std::borrow::Cow<'_, str>],
        output_map: &PipelineHashMap<SmolStr, SmolStr>,
        context: &crate::pipeline::execution_descriptor::ExecutionContext,
    ) -> Result<(), AamlError> {
        let schema = context
            .schemas
            .get(schema_name)
            .ok_or_else(|| Self::schema_not_found_error(schema_name))?;

        for key in root_keys {
            for (field, (type_name, is_optional)) in &schema.fields {
                let full_key = Self::build_full_key(key.as_ref(), field);
                Self::validate_schema_field(
                    schema_name,
                    field,
                    type_name,
                    *is_optional,
                    &full_key,
                    output_map,
                    context,
                )?;
            }
        }

        Ok(())
    }

    fn child_field_key(child_key: &str, field: &str) -> String {
        if child_key.is_empty() {
            field.to_string()
        } else {
            format!("{child_key}.{field}")
        }
    }

    fn apply_inherited_field_default(
        child_key: &str,
        field: &str,
        type_name: &str,
        output_map: &mut PipelineHashMap<SmolStr, SmolStr>,
        context: &crate::pipeline::execution_descriptor::ExecutionContext,
    ) {
        let full_child_key = Self::child_field_key(child_key, field);
        if output_map.contains_key(full_child_key.as_str()) {
            return;
        }

        if let Some(default_val) = context.default_value_for_type(type_name) {
            output_map.insert(SmolStr::new(&full_child_key), SmolStr::new(default_val));
        }
    }

    fn execute_inheritance(
        derive_path: &str,
        child_key: &str,
        output_map: &mut PipelineHashMap<SmolStr, SmolStr>,
        context: &crate::pipeline::execution_descriptor::ExecutionContext,
    ) {
        let parts: Vec<&str> = derive_path.split("::").collect();
        if parts.len() < 2 {
            return;
        }

        for schema_name in &parts[1..] {
            if let Some(schema) = context.schemas.get(*schema_name) {
                for (field, (type_name, _is_optional)) in &schema.fields {
                    Self::apply_inherited_field_default(
                        child_key, field, type_name, output_map, context,
                    );
                }
            }
        }
    }

    fn import_file(
        file_path: &str,
        merge_strategy: &str,
        output_map: &mut PipelineHashMap<SmolStr, SmolStr>,
        source_dir: Option<&Path>,
    ) -> Result<(), AamlError> {
        let resolved = resolve_relative_path(file_path, source_dir);
        let content = std::fs::read_to_string(&resolved).map_err(|e| AamlError::IoError {
            details: e.to_string(),
            diagnostics: Some(Box::new(ErrorDiagnostics::new(
                "I/O operation failed",
                format!(
                    "Could not read imported file '{}': {}",
                    resolved.display(),
                    e
                ),
                "Check file permissions and ensure the path exists",
            ))),
        })?;

        let sub_pipeline = crate::pipeline::Pipeline::new();
        let arena = Bump::new();
        let sub_output = match sub_pipeline.process_with_arena_and_source_dir(
            &content,
            &arena,
            resolved.parent(),
        ) {
            Ok(out) => out,
            Err(errors) => {
                return Err(errors.into_iter().next().unwrap_or_else(|| {
                    AamlError::DirectiveError {
                        directive: "import".to_string(),
                        message: "Unknown error in imported file".to_string(),
                        diagnostics: None,
                    }
                }));
            }
        };

        for (k, v) in sub_output.map {
            if merge_strategy == "override" || !output_map.contains_key(&*k) {
                output_map.insert(k, v);
            }
        }
        Ok(())
    }

    fn resolve_reference(
        source_key: &str,
        target_key: &str,
        output_map: &mut PipelineHashMap<SmolStr, SmolStr>,
    ) -> Result<(), AamlError> {
        if let Some(target_value) = output_map.get(target_key) {
            output_map.insert(SmolStr::from(source_key), target_value.clone());
            return Ok(());
        }

        Err(AamlError::NotFound {
            key: target_key.to_string(),
            context: format!(
                "Reference target '{target_key}' not found when resolving '{source_key}'"
            ),
            diagnostics: None,
        })
    }

    /// Executes a single execution task within the context.
    fn execute_task(
        task: &ExecutionTask,
        output_map: &mut PipelineHashMap<SmolStr, SmolStr>,
        context: &crate::pipeline::execution_descriptor::ExecutionContext,
    ) -> Result<(), AamlError> {
        match task {
            ExecutionTask::SetValue { key, value, .. } => {
                Self::set_value(output_map, key.as_ref(), value.as_ref());
                Ok(())
            }
            ExecutionTask::MergeValue { key, value, .. } => {
                Self::merge_value(output_map, key.as_ref(), value.as_ref());
                Ok(())
            }
            ExecutionTask::ApplySchema {
                schema_name,
                root_keys,
                line: _,
            } => Self::apply_schema(schema_name.as_ref(), root_keys, output_map, context),
            ExecutionTask::ExecuteInheritance {
                derive_path,
                child_key,
                line: _,
            } => {
                Self::execute_inheritance(
                    derive_path.as_ref(),
                    child_key.as_ref(),
                    output_map,
                    context,
                );
                Ok(())
            }
            ExecutionTask::ImportFile {
                file_path,
                merge_strategy,
                line: _,
            } => Self::import_file(
                file_path.as_ref(),
                merge_strategy.as_ref(),
                output_map,
                crate::pipeline::source_dir::get().as_deref(),
            ),
            ExecutionTask::ResolveReference {
                source_key,
                target_key,
                ..
            } => Self::resolve_reference(source_key.as_ref(), target_key.as_ref(), output_map),
        }
    }
}

impl Default for DefaultExecuter {
    fn default() -> Self {
        Self::new()
    }
}

impl Executer for DefaultExecuter {
    fn execute(&self, manifest: &mut ExecutionDescriptor) -> Result<(), AamlError> {
        // Move map out to avoid cloning on every execution pass.
        let mut output_map = std::mem::take(&mut manifest.context.map);
        let context = &manifest.context;

        // Execute all execution tasks in order
        for task in &manifest.execution_tasks {
            Self::execute_task(task, &mut output_map, context)?;
        }

        // Update the manifest context with final output
        manifest.context.map = output_map;

        Ok(())
    }
}
