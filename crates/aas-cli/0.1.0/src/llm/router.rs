use crate::llm::traits::LLMProvider;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskType {
    FastAnalysis,   // Hermes
    DeepReasoning,  // Claude
    CodeEdit,       // Claude Code
    ExternalTask,   // OpenClaw
    Fallback,       // Mock
}

pub struct LLMRouter {
    providers: HashMap<TaskType, Arc<dyn LLMProvider>>,
}

impl LLMRouter {
    pub fn new() -> Self {
        LLMRouter {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, task_type: TaskType, provider: Arc<dyn LLMProvider>) {
        self.providers.insert(task_type, provider);
    }

    pub fn route(&self, task_type: TaskType) -> Arc<dyn LLMProvider> {
        self.providers
            .get(&task_type)
            .or_else(|| self.providers.get(&TaskType::Fallback))
            .cloned()
            .unwrap_or_else(|| {
                // Ultimate fallback: return a mock provider wrapped as dyn LLMProvider
                // This is a safety net; should not happen in practice
                Arc::new(crate::llm::mock::MockLLMProvider)
            })
    }
}

impl Default for LLMRouter {
    fn default() -> Self {
        Self::new()
    }
}
