use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    Result,
    common::Vars,
    runtime::Context,
    workflow::{actions::ActionType, node::NodeId},
};

use super::{Action, ActionOutput};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StartAction;

#[async_trait]
#[typetag::serde]
impl Action for StartAction {
    fn create(_: serde_json::Value) -> Result<Self> {
        Ok(StartAction)
    }

    fn schema() -> serde_json::Value {
        json!({})
    }

    fn action_type(&self) -> ActionType {
        ActionType::Start
    }

    async fn run(
        &self,
        _: Arc<Context>,
        _: NodeId,
    ) -> Result<ActionOutput> {
        Ok(ActionOutput::success(Vars::new()))
    }
}
