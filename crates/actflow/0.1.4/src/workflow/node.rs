//! Workflow node definitions and execution types.
//!
//! Nodes are the building blocks of a workflow, each representing an action
//! to be executed (e.g., HTTP request, code execution, conditional branching).

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    ActflowError, Result,
    common::Vars,
    workflow::actions::{Action, ActionOutput, ActionType, AgentAction, CodeAction, EndAction, HttpRequestAction, IfElseAction, StartAction},
};

/// Unique identifier for a node within a workflow.
pub type NodeId = String;

/// State of a node or edge during workflow execution.
///
/// Tracks the lifecycle of nodes and edges through the execution process.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, strum::AsRefStr, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum NodeState {
    /// Initial state - not yet processed.
    #[default]
    Unknown,
    /// Node has been picked up for execution.
    Taken,
    /// Node has completed execution.
    Executed,
    /// Node was skipped (e.g., due to conditional branching).
    Skipped,
}

/// Execution status indicating the result of a node's action.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, strum::AsRefStr, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum NodeExecutionStatus {
    /// Action has not yet executed.
    #[default]
    Pending,
    /// Action completed successfully.
    Succeeded,
    /// Action failed with an error.
    Failed,
    /// Action encountered an unexpected exception.
    Exception,
    /// Action was stopped (e.g., due to shutdown).
    Stopped,
    /// Action is paused (for future use).
    Paused,
}

/// Error handling strategy for node failures.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, strum::AsRefStr, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ErrorStrategy {
    /// No special handling - propagate error.
    #[default]
    None,
    /// Use a default value on error.
    DefaultValue,
    /// Take the fail branch on error.
    FaileBranch,
}

#[derive(Deserialize)]
struct NodeMetadata {
    id: NodeId,
    title: String,
    #[serde(default)]
    desc: String,
    #[serde(default)]
    error_strategy: ErrorStrategy,
    #[serde(default)]
    default_value: Option<Vars>,
    #[serde(default)]
    retry: Option<RetryConfig>,
    uses: ActionType,
    // timeout in milliseconds
    #[serde(default)]
    timeout: Option<u64>,
    action: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct Node {
    /// node id
    pub id: NodeId,
    /// node title
    pub title: String,
    /// node description
    pub desc: String,
    /// error strategy
    pub error_strategy: ErrorStrategy,
    /// default value when error occurs
    pub default_value: Option<Vars>,
    /// retry config
    pub retry: Option<RetryConfig>,
    /// uses which action
    pub uses: ActionType,
    /// node execution state
    #[serde(default)]
    pub status: NodeState,
    /// action execution timeout
    pub timeout: Option<Duration>,
    /// action data
    pub action_data: serde_json::Value,
    /// node action
    pub action: Box<dyn Action>,
}

impl Clone for Node {
    fn clone(&self) -> Self {
        let action = Self::create_action(self.uses, self.action_data.clone()).unwrap();

        Self {
            id: self.id.clone(),
            title: self.title.clone(),
            desc: self.desc.clone(),
            error_strategy: self.error_strategy.clone(),
            default_value: self.default_value.clone(),
            retry: self.retry.clone(),
            uses: self.uses.clone(),
            status: self.status.clone(),
            timeout: self.timeout.clone(),
            action_data: self.action_data.clone(),
            action: action,
        }
    }
}

impl Node {
    pub fn new(input: Vars) -> Result<Self> {
        let node_input: NodeMetadata = serde_json::from_value(input.into()).map_err(|e| ActflowError::Node(format!("invalid node input: {}", e)))?;

        let action = Self::create_action(node_input.uses, node_input.action.clone())?;

        Ok(Self {
            id: node_input.id,
            title: node_input.title,
            desc: node_input.desc,
            error_strategy: node_input.error_strategy,
            default_value: node_input.default_value,
            retry: node_input.retry,
            uses: node_input.uses,
            status: NodeState::Unknown,
            timeout: node_input.timeout.map(Duration::from_millis),
            action_data: node_input.action,
            action,
        })
    }

    fn create_action(
        uses: ActionType,
        action_params: serde_json::Value,
    ) -> Result<Box<dyn Action>> {
        match uses {
            ActionType::Agent => Ok(Box::new(AgentAction::create(action_params)?)),
            ActionType::Code => Ok(Box::new(CodeAction::create(action_params)?)),
            ActionType::End => Ok(Box::new(EndAction::create(action_params)?)),
            ActionType::HttpRequest => Ok(Box::new(HttpRequestAction::create(action_params)?)),
            ActionType::IfElse => Ok(Box::new(IfElseAction::create(action_params)?)),
            ActionType::Start => Ok(Box::new(StartAction::create(action_params)?)),
            _ => return Err(ActflowError::Node(format!("invalid 'uses': {:?}", uses))),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RetryConfig {
    /// retry times
    pub times: u64,
    /// retry interval in milliseconds
    pub interval: u64,
}

/// Result of a node execution
#[derive(Debug, Clone)]
pub struct NodeResult {
    /// action execution status
    pub status: NodeExecutionStatus,
    /// action outputs
    pub outputs: Vars,
    /// action error message
    pub error: Option<String>,
    /// action exception message
    pub exception: Option<String>,
}

impl NodeResult {
    /// Create NodeResult from ActionOutput
    pub fn from_output(output: ActionOutput) -> Self {
        Self {
            status: output.status,
            outputs: output.outputs,
            error: output.error,
            exception: output.exception,
        }
    }

    pub fn from_result_output(output: Result<ActionOutput>) -> Self {
        match output {
            Ok(action_output) => Self {
                status: action_output.status,
                outputs: action_output.outputs,
                error: action_output.error,
                exception: action_output.exception,
            },
            Err(e) => Self {
                status: NodeExecutionStatus::Exception,
                outputs: Vars::new(),
                error: None,
                exception: Some(e.to_string()),
            },
        }
    }
}
