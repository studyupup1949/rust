//! Type definitions and utilities for the Agent SDK.

use crate::proto;

/// Agent inputs as JSON value.
pub type Inputs = serde_json::Value;

/// Agent execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionStatus {
    Pending,
    Succeeded,
    Failed,
    Exception,
    Stopped,
    Paused,
}

impl From<ExecutionStatus> for proto::NodeExecutionStatus {
    fn from(status: ExecutionStatus) -> Self {
        match status {
            ExecutionStatus::Pending => proto::NodeExecutionStatus::Pending,
            ExecutionStatus::Succeeded => proto::NodeExecutionStatus::Succeeded,
            ExecutionStatus::Failed => proto::NodeExecutionStatus::Failed,
            ExecutionStatus::Exception => proto::NodeExecutionStatus::Exception,
            ExecutionStatus::Stopped => proto::NodeExecutionStatus::Stopped,
            ExecutionStatus::Paused => proto::NodeExecutionStatus::Paused,
        }
    }
}

/// Agent output result.
#[derive(Debug, Clone)]
pub struct AgentOutput {
    /// Execution status.
    pub status: ExecutionStatus,
    /// Output data.
    pub outputs: serde_json::Value,
    /// Error message (if status is Failed).
    pub error: String,
    /// Exception message (if status is Exception).
    pub exception: String,
}

impl AgentOutput {
    /// Create a successful output.
    pub fn success(outputs: serde_json::Value) -> Self {
        Self {
            status: ExecutionStatus::Succeeded,
            outputs,
            error: String::new(),
            exception: String::new(),
        }
    }

    /// Create a failed output with error message.
    pub fn failed(error: impl Into<String>) -> Self {
        Self {
            status: ExecutionStatus::Failed,
            outputs: serde_json::Value::Null,
            error: error.into(),
            exception: String::new(),
        }
    }

    /// Create an exception output.
    pub fn exception(exception: impl Into<String>) -> Self {
        Self {
            status: ExecutionStatus::Exception,
            outputs: serde_json::Value::Null,
            error: String::new(),
            exception: exception.into(),
        }
    }
}

impl From<AgentOutput> for proto::AgentOutput {
    fn from(output: AgentOutput) -> Self {
        Self {
            status: proto::NodeExecutionStatus::from(output.status) as i32,
            outputs: Some(json_to_prost_value(output.outputs)),
            error: output.error,
            exception: output.exception,
        }
    }
}

/// Convert prost_types::Value to serde_json::Value.
pub(crate) fn prost_value_to_json(value: prost_types::Value) -> serde_json::Value {
    use prost_types::value::Kind;

    match value.kind {
        Some(Kind::NullValue(_)) => serde_json::Value::Null,
        Some(Kind::NumberValue(n)) => serde_json::json!(n),
        Some(Kind::StringValue(s)) => serde_json::Value::String(s),
        Some(Kind::BoolValue(b)) => serde_json::Value::Bool(b),
        Some(Kind::StructValue(s)) => {
            let map: serde_json::Map<String, serde_json::Value> = s
                .fields
                .into_iter()
                .map(|(k, v)| (k, prost_value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        Some(Kind::ListValue(l)) => {
            let arr: Vec<serde_json::Value> =
                l.values.into_iter().map(prost_value_to_json).collect();
            serde_json::Value::Array(arr)
        }
        None => serde_json::Value::Null,
    }
}

/// Convert serde_json::Value to prost_types::Value.
pub(crate) fn json_to_prost_value(value: serde_json::Value) -> prost_types::Value {
    use prost_types::value::Kind;

    let kind = match value {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(b) => Kind::BoolValue(b),
        serde_json::Value::Number(n) => Kind::NumberValue(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Kind::StringValue(s),
        serde_json::Value::Array(arr) => Kind::ListValue(prost_types::ListValue {
            values: arr.into_iter().map(json_to_prost_value).collect(),
        }),
        serde_json::Value::Object(map) => Kind::StructValue(prost_types::Struct {
            fields: map
                .into_iter()
                .map(|(k, v)| (k, json_to_prost_value(v)))
                .collect(),
        }),
    };

    prost_types::Value { kind: Some(kind) }
}
