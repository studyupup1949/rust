use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use tonic::transport::Channel;

use crate::{
    ActflowError, Result,
    common::Vars,
    runtime::Context,
    workflow::{
        actions::{Action, ActionOutput, ActionType},
        node::{NodeExecutionStatus, NodeId},
        template,
    },
};

use super::pb::{self, agent_service_client::AgentServiceClient, agent_update::RelayMessage};

/// Agent action that calls a remote agent service via gRPC.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentAction {
    /// gRPC endpoint of the agent service (e.g., "http://127.0.0.1:50051")
    endpoint: String,
    /// Inputs to the agent
    inputs: serde_json::Value,
}

impl AgentAction {
    /// Map proto NodeExecutionStatus to workflow NodeExecutionStatus
    fn map_status(status: pb::NodeExecutionStatus) -> NodeExecutionStatus {
        match status {
            pb::NodeExecutionStatus::Pending => NodeExecutionStatus::Pending,
            pb::NodeExecutionStatus::Succeeded => NodeExecutionStatus::Succeeded,
            pb::NodeExecutionStatus::Failed => NodeExecutionStatus::Failed,
            pb::NodeExecutionStatus::Exception => NodeExecutionStatus::Exception,
            pb::NodeExecutionStatus::Stopped => NodeExecutionStatus::Stopped,
            pb::NodeExecutionStatus::Paused => NodeExecutionStatus::Paused,
        }
    }
}

#[async_trait]
#[typetag::serde]
impl Action for AgentAction {
    fn create(params: serde_json::Value) -> Result<Self> {
        jsonschema::validate(&params, &Self::schema())?;
        let action = serde_json::from_value::<Self>(params)?;
        Ok(action)
    }

    fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["endpoint"],
            "properties": {
                "endpoint": {
                    "type": "string",
                    "description": "gRPC endpoint of the agent service (e.g., 'http://127.0.0.1:50051')"
                },
                "inputs": {
                    "type": "object",
                    "description": "Inputs to the agent"
                }
            }
        })
    }

    fn action_type(&self) -> ActionType {
        ActionType::Agent
    }

    async fn run(
        &self,
        ctx: Arc<Context>,
        nid: NodeId,
    ) -> Result<ActionOutput> {
        // Connect to the agent service
        let channel = Channel::from_shared(self.endpoint.clone())
            .map_err(|e| ActflowError::Action(format!("Invalid endpoint: {}", e)))?
            .connect()
            .await
            .map_err(|e| ActflowError::Action(format!("Failed to connect to agent service: {}", e)))?;

        let mut client = AgentServiceClient::new(channel);

        // Resolve template variables in inputs
        let resolved_inputs = template::resolve_json_value(&ctx, &self.inputs)?;

        // Build the request
        let request = pb::RunRequest {
            pid: ctx.pid(),
            nid: nid.clone(),
            inputs: Some(json_to_prost_value(&resolved_inputs)),
        };

        // Call the agent service (streaming response)
        let response = client.run(request).await.map_err(|e| ActflowError::Action(format!("gRPC call failed: {}", e)))?;

        let mut stream = response.into_inner();

        let mut agent_output: Option<pb::AgentOutput> = None;

        // Get shutdown future for cancellation
        let shutdown = ctx.wait_shutdown();
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = &mut shutdown => {
                    // Context shutdown - shutdown agent service
                    let _ = client.shutdown(pb::Empty {}).await;
                    return Ok(ActionOutput::stopped());
                }
                // Process next stream message
                result = stream.next() => {
                    match result {
                        Some(Ok(update)) => {
                            match update.relay_message {
                                Some(RelayMessage::Log(log_content)) => {
                                    // Forward log to workflow context
                                    ctx.emit_log(nid.clone(), log_content);
                                }
                                Some(RelayMessage::Output(output)) => {
                                    // Store the final output and break
                                    agent_output = Some(output);
                                    break;
                                }
                                None => {}
                            }
                        }
                        Some(Err(e)) => {
                            return Err(ActflowError::Action(format!("Stream error: {}", e)));
                        }
                        None => {
                            // Stream ended without output
                            break;
                        }
                    }
                }
            }
        }

        // Process the final output
        match agent_output {
            Some(output) => {
                let status = Self::map_status(output.status());
                let outputs: Vars = output.outputs.map(|v| prost_value_to_json(&v)).unwrap_or(serde_json::Value::Null).into();

                let error = if output.error.is_empty() {
                    None
                } else {
                    Some(output.error)
                };
                let exception = if output.exception.is_empty() {
                    None
                } else {
                    Some(output.exception)
                };

                Ok(ActionOutput {
                    status,
                    outputs,
                    error,
                    exception,
                })
            }
            None => Err(ActflowError::Action("Agent service did not return any output".to_string())),
        }
    }
}

/// Convert serde_json::Value to prost_types::Value
fn json_to_prost_value(json: &serde_json::Value) -> prost_types::Value {
    use prost_types::value::Kind;

    let kind = match json {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(b) => Kind::BoolValue(*b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Kind::NumberValue(f)
            } else {
                Kind::NullValue(0)
            }
        }
        serde_json::Value::String(s) => Kind::StringValue(s.clone()),
        serde_json::Value::Array(arr) => {
            let values: Vec<prost_types::Value> = arr.iter().map(json_to_prost_value).collect();
            Kind::ListValue(prost_types::ListValue {
                values,
            })
        }
        serde_json::Value::Object(obj) => {
            let fields: BTreeMap<String, prost_types::Value> = obj.iter().map(|(k, v)| (k.clone(), json_to_prost_value(v))).collect();
            Kind::StructValue(prost_types::Struct {
                fields,
            })
        }
    };

    prost_types::Value {
        kind: Some(kind),
    }
}

/// Convert prost_types::Value to serde_json::Value
fn prost_value_to_json(prost: &prost_types::Value) -> serde_json::Value {
    use prost_types::value::Kind;

    match &prost.kind {
        Some(Kind::NullValue(_)) => serde_json::Value::Null,
        Some(Kind::BoolValue(b)) => serde_json::Value::Bool(*b),
        Some(Kind::NumberValue(n)) => serde_json::json!(*n),
        Some(Kind::StringValue(s)) => serde_json::Value::String(s.clone()),
        Some(Kind::ListValue(list)) => {
            let arr: Vec<serde_json::Value> = list.values.iter().map(prost_value_to_json).collect();
            serde_json::Value::Array(arr)
        }
        Some(Kind::StructValue(s)) => {
            let obj: serde_json::Map<String, serde_json::Value> = s.fields.iter().map(|(k, v)| (k.clone(), prost_value_to_json(v))).collect();
            serde_json::Value::Object(obj)
        }
        None => serde_json::Value::Null,
    }
}
