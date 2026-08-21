#![doc = include_str!("../README.md")]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Core protocol types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Part {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "file")]
    File { file: FileContent },
    #[serde(rename = "data")]
    Data { data: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub name: Option<String>,
    pub mime_type: Option<String>,
    pub bytes: Option<String>,
    pub uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub session_id: Option<String>,
    pub status: TaskStatus,
    pub artifacts: Option<Vec<Artifact>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub state: TaskState,
    pub message: Option<Message>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskState {
    #[serde(rename = "submitted")]
    Submitted,
    #[serde(rename = "working")]
    Working,
    #[serde(rename = "input-required")]
    InputRequired,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "canceled")]
    Canceled,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: Option<String>,
    pub description: Option<String>,
    pub parts: Vec<Part>,
}

// Core protocol trait
#[async_trait]
pub trait A2AProtocol {
    async fn send_task(&self, message: Message) -> Result<Task, Box<dyn std::error::Error>>;
    async fn get_task(&self, task_id: &str) -> Result<Task, Box<dyn std::error::Error>>;
    async fn cancel_task(&self, task_id: &str) -> Result<Task, Box<dyn std::error::Error>>;
}

// Basic in-memory task store implementation
#[derive(Default)]
pub struct InMemoryTaskStore {
    tasks: HashMap<String, Task>,
}

impl InMemoryTaskStore {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn store_task(&mut self, task: Task) {
        self.tasks.insert(task.id.clone(), task);
    }

    pub fn get_task(&self, task_id: &str) -> Option<Task> {
        self.tasks.get(task_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_task_lifecycle() {
        let mut store = InMemoryTaskStore::new();

        let task = Task {
            id: "task1".to_string(),
            session_id: None,
            status: TaskStatus {
                state: TaskState::Submitted,
                message: None,
                timestamp: Utc::now().to_rfc3339(),
            },
            artifacts: None,
        };

        store.store_task(task.clone());

        let retrieved = store.get_task("task1").unwrap();
        assert!(matches!(retrieved.status.state, TaskState::Submitted));
    }
}
