mod agent;
mod code;
mod end;
mod http_request;
mod if_else;
mod start;

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    common::Vars,
    runtime::Context,
    workflow::node::{NodeExecutionStatus, NodeId},
};

pub use agent::AgentAction;
pub use code::CodeAction;
pub use end::EndAction;
pub use http_request::HttpRequestAction;
pub use if_else::IfElseAction;
pub use start::StartAction;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, strum::AsRefStr, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ActionType {
    #[default]
    None,
    Agent,
    Code,
    End,
    HttpRequest,
    IfElse,
    Start,
}

#[async_trait]
#[typetag::serde(tag = "type")]
pub trait Action: Send + Sync {
    /// Creates a new instance of the action from the given inputs.
    ///
    /// # Arguments
    ///
    /// * `params` - The [`serde_json::Value`] containing the params for the action.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing the created action instance.
    fn create(params: serde_json::Value) -> Result<Self>
    where
        Self: Sized;

    /// Returns the schema of the action.
    ///
    /// # Returns
    ///
    /// Returns a [`serde_json::Value`] representing the schema of the action.
    fn schema() -> serde_json::Value
    where
        Self: Sized;

    /// Returns the type of the action.
    /// # Returns
    ///
    /// Returns the [`ActionType`] of the action.
    fn action_type(&self) -> ActionType;

    /// Executes the node's action with the given context and inputs.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The [`Context`] of the workflow.
    /// * `nid` - The id of the node.
    ///
    /// # Returns
    ///
    /// Returns an [`Result<ActionOutput>`] representing the output of the action.
    async fn run(
        &self,
        ctx: Arc<Context>,
        nid: NodeId,
    ) -> Result<ActionOutput>;
}

/// Output returned by an action's run method
#[derive(Debug, Clone)]
pub struct ActionOutput {
    /// action execution status
    pub status: NodeExecutionStatus,
    /// action outputs
    pub outputs: Vars,
    /// action error message
    pub error: Option<String>,
    /// action exception message
    pub exception: Option<String>,
}

impl ActionOutput {
    /// Create a successful action output
    pub fn success(outputs: Vars) -> Self {
        Self {
            status: NodeExecutionStatus::Succeeded,
            outputs,
            error: None,
            exception: None,
        }
    }

    /// Create a failed action output
    pub fn failed(error: String) -> Self {
        Self {
            status: NodeExecutionStatus::Failed,
            outputs: Vars::new(),
            error: Some(error),
            exception: None,
        }
    }

    /// Create an exception action output
    pub fn exception(message: String) -> Self {
        Self {
            status: NodeExecutionStatus::Exception,
            outputs: Vars::new(),
            error: None,
            exception: Some(message),
        }
    }

    /// Create a stopped action output
    pub fn stopped() -> Self {
        Self {
            status: NodeExecutionStatus::Stopped,
            outputs: Vars::new(),
            error: None,
            exception: None,
        }
    }
}
