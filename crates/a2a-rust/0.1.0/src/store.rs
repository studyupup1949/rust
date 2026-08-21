use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::A2AError;
use crate::types::{ListTasksRequest, ListTasksResponse, Task};

/// Persistence abstraction for task state exposed by the A2A server.
#[async_trait]
pub trait TaskStore: Send + Sync + 'static {
    /// Fetch a task by its identifier.
    async fn get(&self, task_id: &str) -> Result<Option<Task>, A2AError>;

    /// Insert or replace a task snapshot.
    async fn put(&self, task: &Task) -> Result<(), A2AError>;

    /// Implementations should reject invalid pagination inputs or delegate to
    /// `ListTasksRequest::validate()` before applying query semantics.
    async fn list(&self, req: &ListTasksRequest) -> Result<ListTasksResponse, A2AError>;

    /// Delete a task by identifier.
    async fn delete(&self, task_id: &str) -> Result<bool, A2AError>;
}

/// Configuration for the in-memory task store.
#[derive(Debug, Clone, Copy, Default)]
pub struct InMemoryTaskStoreConfig {
    /// Maximum age for stored entries before they are purged on access.
    pub entry_ttl: Option<Duration>,
    /// Maximum number of tasks retained before least-recently-used eviction.
    pub max_entries: Option<usize>,
}

#[derive(Debug, Clone)]
struct StoredTask {
    task: Task,
    updated_at: Instant,
    last_accessed_at: Instant,
}

/// In-process task store with TTL expiry and LRU capacity eviction.
#[derive(Debug)]
pub struct InMemoryTaskStore {
    config: InMemoryTaskStoreConfig,
    tasks: RwLock<BTreeMap<String, StoredTask>>,
}

impl Default for InMemoryTaskStore {
    fn default() -> Self {
        Self::with_config(InMemoryTaskStoreConfig::default())
    }
}

impl InMemoryTaskStore {
    /// Create a store with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a store with explicit TTL and capacity settings.
    pub fn with_config(config: InMemoryTaskStoreConfig) -> Self {
        Self {
            config,
            tasks: RwLock::new(BTreeMap::new()),
        }
    }
}

#[async_trait]
impl TaskStore for InMemoryTaskStore {
    async fn get(&self, task_id: &str) -> Result<Option<Task>, A2AError> {
        let mut tasks = self.tasks.write().await;
        purge_expired(&mut tasks, self.config);

        Ok(tasks.get_mut(task_id).map(|stored| {
            stored.last_accessed_at = Instant::now();
            stored.task.clone()
        }))
    }

    async fn put(&self, task: &Task) -> Result<(), A2AError> {
        let mut tasks = self.tasks.write().await;
        purge_expired(&mut tasks, self.config);

        let now = Instant::now();
        tasks.insert(
            task.id.clone(),
            StoredTask {
                task: task.clone(),
                updated_at: now,
                last_accessed_at: now,
            },
        );
        enforce_capacity(&mut tasks, self.config.max_entries);
        Ok(())
    }

    async fn list(&self, req: &ListTasksRequest) -> Result<ListTasksResponse, A2AError> {
        req.validate()?;

        let mut tasks = self.tasks.write().await;
        purge_expired(&mut tasks, self.config);

        let mut matching_tasks: Vec<Task> =
            tasks.values().map(|stored| stored.task.clone()).collect();
        matching_tasks.retain(|task| task_matches(task, req));
        matching_tasks.sort_by_key(|task| Reverse(task_sort_key(task)));

        // The in-memory store currently uses offset-style tokens for simplicity.
        // Downstream stores should prefer stable cursors that do not shift under writes.
        let start = req
            .page_token
            .as_deref()
            .unwrap_or("0")
            .parse::<usize>()
            .map_err(|_| A2AError::InvalidRequest("invalid pageToken".to_owned()))?;
        let requested_page_size = req.page_size.unwrap_or(50);
        let page_size = requested_page_size.clamp(1, 100) as usize;
        let total_size = matching_tasks.len() as i32;
        let page = matching_tasks
            .into_iter()
            .skip(start)
            .take(page_size)
            .map(|mut task| {
                apply_history_length(&mut task, req.history_length);
                if req.include_artifacts != Some(true) {
                    task.artifacts.clear();
                }
                task
            })
            .collect::<Vec<_>>();
        let accessed_at = Instant::now();
        for task in &page {
            if let Some(stored) = tasks.get_mut(&task.id) {
                stored.last_accessed_at = accessed_at;
            }
        }

        let next_start = start + page.len();
        let next_page_token = if next_start >= total_size as usize {
            String::new()
        } else {
            next_start.to_string()
        };

        Ok(ListTasksResponse {
            tasks: page,
            next_page_token,
            page_size: requested_page_size,
            total_size,
        })
    }

    async fn delete(&self, task_id: &str) -> Result<bool, A2AError> {
        let mut tasks = self.tasks.write().await;
        purge_expired(&mut tasks, self.config);

        Ok(tasks.remove(task_id).is_some())
    }
}

fn purge_expired(tasks: &mut BTreeMap<String, StoredTask>, config: InMemoryTaskStoreConfig) {
    let Some(entry_ttl) = config.entry_ttl else {
        return;
    };

    let now = Instant::now();
    tasks.retain(|_, stored| now.duration_since(stored.updated_at) < entry_ttl);
}

fn enforce_capacity(tasks: &mut BTreeMap<String, StoredTask>, max_entries: Option<usize>) {
    let Some(max_entries) = max_entries else {
        return;
    };

    while tasks.len() > max_entries {
        let Some(oldest_key) = tasks
            .iter()
            .min_by(|(left_id, left), (right_id, right)| {
                left.last_accessed_at
                    .cmp(&right.last_accessed_at)
                    .then_with(|| left_id.cmp(right_id))
            })
            .map(|(task_id, _)| task_id.clone())
        else {
            break;
        };

        tasks.remove(&oldest_key);
    }
}

fn task_matches(task: &Task, req: &ListTasksRequest) -> bool {
    if let Some(context_id) = &req.context_id
        && task.context_id.as_deref() != Some(context_id.as_str())
    {
        return false;
    }

    if let Some(status) = req.status
        && task.status.state != status
    {
        return false;
    }

    if let Some(after) = &req.status_timestamp_after {
        let Some(timestamp) = task.status.timestamp.as_ref() else {
            return false;
        };

        if timestamp < after {
            return false;
        }
    }

    true
}

fn task_sort_key(task: &Task) -> (String, String) {
    (
        task.status.timestamp.clone().unwrap_or_default(),
        task.id.clone(),
    )
}

fn apply_history_length(task: &mut Task, history_length: Option<i32>) {
    let Some(history_length) = history_length else {
        return;
    };

    if history_length <= 0 {
        task.history.clear();
        return;
    }

    let keep = history_length as usize;
    if task.history.len() > keep {
        let start = task.history.len() - keep;
        task.history = task.history.split_off(start);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::time::sleep;

    use super::{InMemoryTaskStore, InMemoryTaskStoreConfig, TaskStore};
    use crate::types::{ListTasksRequest, Task, TaskState, TaskStatus};

    #[tokio::test]
    async fn in_memory_task_store_lists_tasks_in_timestamp_order() {
        let store = InMemoryTaskStore::new();

        store
            .put(&Task {
                id: "task-1".to_owned(),
                context_id: Some("ctx-1".to_owned()),
                status: TaskStatus {
                    state: TaskState::Submitted,
                    message: None,
                    timestamp: Some("2026-03-12T12:00:00Z".to_owned()),
                },
                artifacts: Vec::new(),
                history: Vec::new(),
                metadata: None,
            })
            .await
            .expect("task should store");

        store
            .put(&Task {
                id: "task-2".to_owned(),
                context_id: Some("ctx-1".to_owned()),
                status: TaskStatus {
                    state: TaskState::Working,
                    message: None,
                    timestamp: Some("2026-03-12T13:00:00Z".to_owned()),
                },
                artifacts: Vec::new(),
                history: Vec::new(),
                metadata: None,
            })
            .await
            .expect("task should store");

        let response = store
            .list(&ListTasksRequest {
                tenant: None,
                context_id: Some("ctx-1".to_owned()),
                status: None,
                page_size: Some(10),
                page_token: None,
                history_length: None,
                status_timestamp_after: None,
                include_artifacts: None,
            })
            .await
            .expect("tasks should list");

        assert_eq!(response.tasks.len(), 2);
        assert_eq!(response.tasks[0].id, "task-2");
        assert_eq!(response.tasks[1].id, "task-1");
        assert_eq!(response.next_page_token, "");
    }

    #[tokio::test]
    async fn in_memory_task_store_excludes_artifacts_by_default() {
        let store = InMemoryTaskStore::new();

        store
            .put(&Task {
                id: "task-1".to_owned(),
                context_id: Some("ctx-1".to_owned()),
                status: TaskStatus {
                    state: TaskState::Completed,
                    message: None,
                    timestamp: Some("2026-03-12T12:00:00Z".to_owned()),
                },
                artifacts: vec![crate::types::Artifact {
                    artifact_id: "artifact-1".to_owned(),
                    name: None,
                    description: None,
                    parts: vec![crate::types::Part {
                        text: Some("done".to_owned()),
                        raw: None,
                        url: None,
                        data: None,
                        metadata: None,
                        filename: None,
                        media_type: None,
                    }],
                    metadata: None,
                    extensions: Vec::new(),
                }],
                history: Vec::new(),
                metadata: None,
            })
            .await
            .expect("task should store");

        let response = store
            .list(&ListTasksRequest {
                tenant: None,
                context_id: None,
                status: None,
                page_size: None,
                page_token: None,
                history_length: None,
                status_timestamp_after: None,
                include_artifacts: None,
            })
            .await
            .expect("tasks should list");

        assert_eq!(response.tasks.len(), 1);
        assert!(response.tasks[0].artifacts.is_empty());
        assert_eq!(response.page_size, 50);
    }

    #[tokio::test]
    async fn in_memory_task_store_expires_entries_by_ttl() {
        let store = InMemoryTaskStore::with_config(InMemoryTaskStoreConfig {
            entry_ttl: Some(Duration::from_millis(5)),
            max_entries: None,
        });

        store
            .put(&Task {
                id: "task-1".to_owned(),
                context_id: Some("ctx-1".to_owned()),
                status: TaskStatus {
                    state: TaskState::Submitted,
                    message: None,
                    timestamp: Some("2026-03-12T12:00:00Z".to_owned()),
                },
                artifacts: Vec::new(),
                history: Vec::new(),
                metadata: None,
            })
            .await
            .expect("task should store");

        sleep(Duration::from_millis(10)).await;

        let task = store.get("task-1").await.expect("lookup should succeed");
        assert!(task.is_none());
    }

    #[tokio::test]
    async fn in_memory_task_store_evicts_least_recently_used_when_capacity_is_exceeded() {
        let store = InMemoryTaskStore::with_config(InMemoryTaskStoreConfig {
            entry_ttl: None,
            max_entries: Some(2),
        });

        store
            .put(&Task {
                id: "task-1".to_owned(),
                context_id: Some("ctx-1".to_owned()),
                status: TaskStatus {
                    state: TaskState::Submitted,
                    message: None,
                    timestamp: Some("2026-03-12T12:00:00Z".to_owned()),
                },
                artifacts: Vec::new(),
                history: Vec::new(),
                metadata: None,
            })
            .await
            .expect("task should store");
        sleep(Duration::from_millis(2)).await;

        store
            .put(&Task {
                id: "task-2".to_owned(),
                context_id: Some("ctx-2".to_owned()),
                status: TaskStatus {
                    state: TaskState::Working,
                    message: None,
                    timestamp: Some("2026-03-12T12:01:00Z".to_owned()),
                },
                artifacts: Vec::new(),
                history: Vec::new(),
                metadata: None,
            })
            .await
            .expect("task should store");
        sleep(Duration::from_millis(2)).await;

        assert!(
            store
                .get("task-1")
                .await
                .expect("lookup should succeed")
                .is_some()
        );
        sleep(Duration::from_millis(2)).await;

        store
            .put(&Task {
                id: "task-3".to_owned(),
                context_id: Some("ctx-3".to_owned()),
                status: TaskStatus {
                    state: TaskState::Completed,
                    message: None,
                    timestamp: Some("2026-03-12T12:02:00Z".to_owned()),
                },
                artifacts: Vec::new(),
                history: Vec::new(),
                metadata: None,
            })
            .await
            .expect("task should store");

        assert!(
            store
                .get("task-1")
                .await
                .expect("lookup should succeed")
                .is_some()
        );
        assert!(
            store
                .get("task-2")
                .await
                .expect("lookup should succeed")
                .is_none()
        );
        assert!(
            store
                .get("task-3")
                .await
                .expect("lookup should succeed")
                .is_some()
        );
    }

    #[tokio::test]
    async fn in_memory_task_store_supports_concurrent_reads_and_writes() {
        let store = InMemoryTaskStore::with_config(InMemoryTaskStoreConfig {
            entry_ttl: None,
            max_entries: None,
        });
        let store = Arc::new(store);
        let mut tasks = Vec::new();

        for index in 0..16 {
            let store = Arc::clone(&store);
            tasks.push(tokio::spawn(async move {
                let task_id = format!("task-{index}");
                store
                    .put(&Task {
                        id: task_id.clone(),
                        context_id: Some("ctx-1".to_owned()),
                        status: TaskStatus {
                            state: TaskState::Working,
                            message: None,
                            timestamp: Some(format!("2026-03-12T12:{index:02}:00Z")),
                        },
                        artifacts: Vec::new(),
                        history: Vec::new(),
                        metadata: None,
                    })
                    .await
                    .expect("task should store");

                let fetched = store.get(&task_id).await.expect("lookup should succeed");
                assert!(fetched.is_some());
            }));
        }

        for task in tasks {
            task.await.expect("task should join");
        }

        let response = store
            .list(&ListTasksRequest {
                tenant: None,
                context_id: Some("ctx-1".to_owned()),
                status: None,
                page_size: Some(100),
                page_token: None,
                history_length: None,
                status_timestamp_after: None,
                include_artifacts: Some(true),
            })
            .await
            .expect("tasks should list");

        assert_eq!(response.tasks.len(), 16);
    }
}
