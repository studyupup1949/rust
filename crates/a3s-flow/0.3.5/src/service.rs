use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    ExecutionState, FlowCapabilities, FlowEngine, FlowError, FlowEvent, Node, NodeDescriptor,
    ValidationIssue,
};

pub type NodeFactory = Arc<dyn Fn() -> Arc<dyn Node> + Send + Sync>;

#[derive(Clone)]
pub struct FlowService {
    engine: Arc<FlowEngine>,
    node_factories: Arc<HashMap<String, NodeFactory>>,
}

impl FlowService {
    pub fn new(engine: Arc<FlowEngine>) -> Self {
        Self::with_factories(engine, HashMap::new())
    }

    pub fn with_factories(
        engine: Arc<FlowEngine>,
        node_factories: HashMap<String, NodeFactory>,
    ) -> Self {
        Self {
            engine,
            node_factories: Arc::new(node_factories),
        }
    }

    pub fn engine(&self) -> Arc<FlowEngine> {
        Arc::clone(&self.engine)
    }

    pub fn capabilities(&self) -> FlowCapabilities {
        self.engine.capabilities()
    }

    pub fn node_types(&self) -> Vec<String> {
        self.engine.node_types()
    }

    pub fn node_descriptors(&self) -> Vec<NodeDescriptor> {
        self.engine.node_descriptors()
    }

    pub fn validate(&self, definition: &Value) -> Vec<ValidationIssue> {
        self.engine.validate(definition)
    }

    pub async fn start_execution(
        &self,
        definition: &Value,
        variables: HashMap<String, Value>,
    ) -> crate::Result<Uuid> {
        self.engine.start(definition, variables).await
    }

    pub async fn get_execution(&self, id: Uuid) -> crate::Result<ExecutionState> {
        self.engine.state(id).await
    }

    pub async fn subscribe(&self, id: Uuid) -> crate::Result<broadcast::Receiver<FlowEvent>> {
        self.engine.subscribe(id).await
    }

    pub async fn pause_execution(&self, id: Uuid) -> crate::Result<ExecutionState> {
        self.engine.pause(id).await?;
        self.engine.state(id).await
    }

    pub async fn resume_execution(&self, id: Uuid) -> crate::Result<ExecutionState> {
        self.engine.resume(id).await?;
        self.engine.state(id).await
    }

    pub async fn terminate_execution(&self, id: Uuid) -> crate::Result<()> {
        self.engine.terminate(id).await
    }

    pub async fn get_context(&self, id: Uuid) -> crate::Result<HashMap<String, Value>> {
        self.engine.get_context(id).await
    }

    pub async fn set_context_entry(
        &self,
        id: Uuid,
        key: String,
        value: Value,
    ) -> crate::Result<()> {
        self.engine.set_context_entry(id, key, value).await
    }

    pub async fn delete_context_entry(&self, id: Uuid, key: &str) -> crate::Result<bool> {
        self.engine.delete_context_entry(id, key).await
    }

    pub async fn run_named_flow(
        &self,
        name: &str,
        variables: HashMap<String, Value>,
    ) -> crate::Result<Uuid> {
        self.engine.start_named(name, variables).await
    }

    pub fn register_node_type(
        &self,
        factory_name: &str,
        descriptor: Option<NodeDescriptor>,
    ) -> crate::Result<(String, bool)> {
        let factory = self
            .node_factories
            .get(factory_name)
            .cloned()
            .ok_or_else(|| {
                FlowError::InvalidDefinition(format!("unknown node factory: {factory_name}"))
            })?;
        let node = factory();
        let node_type = node.node_type().to_string();
        let replaced = self.engine.node_types().contains(&node_type);
        match descriptor {
            Some(descriptor) => self
                .engine
                .register_node_type_with_descriptor(node, descriptor),
            None => self.engine.register_node_type(node),
        }
        Ok((node_type, replaced))
    }

    pub fn unregister_node_type(&self, node_type: &str) -> crate::Result<bool> {
        self.engine.unregister_node_type(node_type)
    }
}

#[allow(dead_code)]
fn _assert_send_sync<T: Send + Sync>() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FlowStore, MemoryFlowStore, NodeRegistry};
    use async_trait::async_trait;
    use serde_json::json;

    struct SlowNode;

    #[async_trait]
    impl Node for SlowNode {
        fn node_type(&self) -> &str {
            "slow"
        }

        async fn execute(&self, _ctx: crate::ExecContext) -> crate::Result<Value> {
            Ok(json!({ "ok": true }))
        }
    }

    #[tokio::test]
    async fn register_node_type_uses_factory_registry() {
        let engine = Arc::new(FlowEngine::new(NodeRegistry::with_defaults()));
        let mut factories: HashMap<String, NodeFactory> = HashMap::new();
        factories.insert("slow-test-node".into(), Arc::new(|| Arc::new(SlowNode)));
        let service = FlowService::with_factories(engine, factories);

        let (node_type, replaced) = service.register_node_type("slow-test-node", None).unwrap();
        assert_eq!(node_type, "slow");
        assert!(!replaced);
        assert!(service.node_types().contains(&"slow".to_string()));
    }

    #[tokio::test]
    async fn run_named_flow_uses_engine_store() {
        let flow_store = Arc::new(MemoryFlowStore::new());
        flow_store
            .save(
                "hello",
                &json!({
                    "nodes": [{ "id": "a", "type": "noop" }],
                    "edges": []
                }),
            )
            .await
            .unwrap();
        let engine = Arc::new(
            FlowEngine::new(NodeRegistry::with_defaults())
                .with_flow_store(flow_store as Arc<dyn FlowStore>),
        );
        let service = FlowService::new(engine);

        let id = service
            .run_named_flow("hello", HashMap::new())
            .await
            .unwrap();
        let state = service.get_execution(id).await.unwrap();
        assert!(matches!(
            state,
            ExecutionState::Running | ExecutionState::Completed(_)
        ));
    }
}
