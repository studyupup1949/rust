use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    ActflowError, Result,
    common::Vars,
    runtime::Context,
    workflow::{
        actions::{Action, ActionOutput, ActionType},
        node::NodeId,
        template,
    },
};

use super::code_executor::{CodeLanguage, JavascriptExecutor, PythonExecutor};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Variable {
    variable: String,
    value_selector: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CodeAction {
    variables: Vec<Variable>,
    code_language: CodeLanguage,
    code: String,
}

#[async_trait]
#[typetag::serde]
impl Action for CodeAction {
    fn create(params: serde_json::Value) -> Result<Self> {
        jsonschema::validate(&params, &Self::schema())?;
        let action = serde_json::from_value::<Self>(params)?;
        Ok(action)
    }

    fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "variables": {
                    "type": "array",
                    "description": "Input variables to pass to the code",
                    "items": {
                        "type": "object",
                        "properties": {
                            "variable": {
                                "type": "string",
                                "description": "Variable name used in the code"
                            },
                            "value_selector": {
                                "type": "string",
                                "description": "Template to resolve the value, e.g. {{#nodeId.key#}}"
                            }
                        },
                        "required": ["variable", "value_selector"]
                    }
                },
                "code_language": {
                    "type": "string",
                    "enum": ["python3", "javascript"],
                    "description": "Programming language of the code"
                },
                "code": {
                    "type": "string",
                    "description": "Code to execute. Must define a function that takes a params dict/object and returns a result"
                }
            },
            "required": ["variables", "code_language", "code"]
        })
    }

    fn action_type(&self) -> ActionType {
        ActionType::Code
    }

    async fn run(
        &self,
        ctx: Arc<Context>,
        _nid: NodeId,
    ) -> Result<ActionOutput> {
        let mut params = Vars::new();
        for var in &self.variables {
            let value = template::resolve_template_to_values(&ctx, &var.value_selector)
                .ok()
                .and_then(|v| v.into_iter().next())
                .ok_or_else(|| ActflowError::Runtime(format!("variable '{}' not found", var.variable)))?;

            params.set(&var.variable, value);
        }

        let result = match self.code_language {
            CodeLanguage::Python3 => PythonExecutor::execute(&self.code, params.into()),
            CodeLanguage::Javascript => JavascriptExecutor::execute(&self.code, params.into()),
        }?;
        Ok(ActionOutput::success(result.into()))
    }
}
