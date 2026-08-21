use std::collections::HashMap;

use agent_client_protocol::schema::v1::{
    ContentBlock, ContentChunk, SessionNotification, SessionUpdate, TextContent, ToolCall,
    ToolCallId, ToolCallUpdate, ToolCallUpdateFields,
};

/// Maximum number of stored notifications after merging.
const MAX_HISTORY: usize = 10_000;

/// History of session notifications with automatic merging of consecutive
/// streaming chunks of the same type.
///
/// Live subscribers receive every original chunk as-is, but storage keeps them
/// collapsed so that e.g. an `AgentThoughtChunk` streamed in many pieces is
/// replayed as one coherent block during `session/load`.
///
/// Tool call updates are also collapsed: updates for the same `toolCallId` are
/// applied into the matching `ToolCall` entry (or merged with the latest stored
/// update if the initial tool call has not arrived yet). This avoids replaying
/// a long stream of incremental updates and keeps history bounded by the number
/// of distinct tool calls.
#[derive(Clone, Debug, Default)]
pub struct NotificationHistory {
    items: Vec<SessionNotification>,
    /// Tracks the canonical history index for each active tool call id.
    tool_call_indices: HashMap<ToolCallId, usize>,
}

impl NotificationHistory {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SessionNotification> + '_ {
        self.items.iter()
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<SessionNotification> {
        self.items.clone()
    }

    /// Remove and return all stored notifications.
    pub fn drain(&mut self) -> Vec<SessionNotification> {
        std::mem::take(&mut self.items)
    }

    /// Append a notification, merging it with the previous one when both are
    /// consecutive streaming chunks of the same kind for the same session.
    ///
    /// Tool call notifications and tool call updates are accumulated by
    /// `toolCallId` instead of being appended as separate history entries.
    pub fn push(&mut self, notification: SessionNotification) {
        let session_id = notification.session_id.clone();
        let meta = notification.meta.clone();

        match notification.update {
            SessionUpdate::ToolCall(tool_call) => {
                self.push_tool_call(session_id, tool_call, meta);
            }
            SessionUpdate::ToolCallUpdate(update) => {
                self.push_tool_call_update(session_id, update, meta);
            }
            _ => {
                if let Some(last) = self.items.last_mut()
                    && let Some(merged) = try_merge(last, &notification)
                {
                    *last = merged;
                    return;
                }
                self.append(notification);
            }
        }
    }

    fn push_tool_call(
        &mut self,
        session_id: agent_client_protocol::schema::v1::SessionId,
        tool_call: ToolCall,
        meta: Option<agent_client_protocol::schema::v1::Meta>,
    ) {
        let id = tool_call.tool_call_id.clone();
        if let Some(&idx) = self.tool_call_indices.get(&id) {
            let existing_fields = self.items.get(idx).and_then(|item| match &item.update {
                SessionUpdate::ToolCallUpdate(update) => Some(update.fields.clone()),
                _ => None,
            });

            let mut final_tool_call = tool_call;
            if let Some(fields) = existing_fields {
                final_tool_call.update(fields);
            }

            let notification =
                SessionNotification::new(session_id, SessionUpdate::ToolCall(final_tool_call))
                    .meta(meta);
            if let Some(slot) = self.items.get_mut(idx) {
                *slot = notification;
            }
            return;
        }

        let notification =
            SessionNotification::new(session_id, SessionUpdate::ToolCall(tool_call)).meta(meta);
        self.append_with_id(id, notification);
    }

    fn push_tool_call_update(
        &mut self,
        session_id: agent_client_protocol::schema::v1::SessionId,
        update: ToolCallUpdate,
        meta: Option<agent_client_protocol::schema::v1::Meta>,
    ) {
        let id = update.tool_call_id.clone();
        if let Some(&idx) = self.tool_call_indices.get(&id) {
            if let Some(item) = self.items.get_mut(idx) {
                match &mut item.update {
                    SessionUpdate::ToolCall(tool_call) => {
                        tool_call.update(update.fields.clone());
                    }
                    SessionUpdate::ToolCallUpdate(existing) => {
                        merge_update_fields(&mut existing.fields, &update.fields);
                    }
                    _ => {}
                }
                if update.meta.is_some() {
                    item.meta.clone_from(&update.meta);
                }
            }
            return;
        }

        let notification =
            SessionNotification::new(session_id, SessionUpdate::ToolCallUpdate(update)).meta(meta);
        self.append_with_id(id, notification);
    }

    fn append(&mut self, notification: SessionNotification) {
        self.items.push(notification);
        self.enforce_limit();
    }

    fn append_with_id(&mut self, id: ToolCallId, notification: SessionNotification) {
        let idx = self.items.len();
        self.items.push(notification);
        self.tool_call_indices.insert(id, idx);
        self.enforce_limit();
    }

    fn enforce_limit(&mut self) {
        if self.items.len() > MAX_HISTORY {
            let excess = self.items.len() - MAX_HISTORY;
            for (i, item) in self.items.iter().enumerate().take(excess) {
                if let Some(id) = tool_call_id_of(&item.update)
                    && self.tool_call_indices.get(id) == Some(&i)
                {
                    self.tool_call_indices.remove(id);
                }
            }
            self.items.drain(..excess);
            for idx in self.tool_call_indices.values_mut() {
                *idx -= excess;
            }
        }
    }

    #[cfg(test)]
    fn tool_call_index_count(&self) -> usize {
        self.tool_call_indices.len()
    }
}

fn tool_call_id_of(update: &SessionUpdate) -> Option<&ToolCallId> {
    match update {
        SessionUpdate::ToolCall(tool_call) => Some(&tool_call.tool_call_id),
        SessionUpdate::ToolCallUpdate(update) => Some(&update.tool_call_id),
        _ => None,
    }
}

fn merge_update_fields(into: &mut ToolCallUpdateFields, from: &ToolCallUpdateFields) {
    if from.kind.is_some() {
        into.kind.clone_from(&from.kind);
    }
    if from.status.is_some() {
        into.status.clone_from(&from.status);
    }
    if from.title.is_some() {
        into.title.clone_from(&from.title);
    }
    if from.content.is_some() {
        into.content.clone_from(&from.content);
    }
    if from.locations.is_some() {
        into.locations.clone_from(&from.locations);
    }
    if from.raw_input.is_some() {
        into.raw_input.clone_from(&from.raw_input);
    }
    if from.raw_output.is_some() {
        into.raw_output.clone_from(&from.raw_output);
    }
}

fn try_merge(a: &SessionNotification, b: &SessionNotification) -> Option<SessionNotification> {
    use SessionUpdate::{AgentMessageChunk, AgentThoughtChunk, UserMessageChunk};

    if a.session_id != b.session_id {
        return None;
    }

    let merged_update = match (&a.update, &b.update) {
        (UserMessageChunk(ca), UserMessageChunk(cb)) => {
            merge_text_chunks(ca, cb).map(UserMessageChunk)
        }
        (AgentMessageChunk(ca), AgentMessageChunk(cb)) => {
            merge_text_chunks(ca, cb).map(AgentMessageChunk)
        }
        (AgentThoughtChunk(ca), AgentThoughtChunk(cb)) => {
            merge_text_chunks(ca, cb).map(AgentThoughtChunk)
        }
        _ => None,
    }?;

    Some(SessionNotification::new(
        a.session_id.clone(),
        merged_update,
    ))
}

fn merge_text_chunks(a: &ContentChunk, b: &ContentChunk) -> Option<ContentChunk> {
    match (&a.content, &b.content) {
        (ContentBlock::Text(ta), ContentBlock::Text(tb)) => Some(ContentChunk::new(
            ContentBlock::Text(TextContent::new(format!("{}{}", ta.text, tb.text))),
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use agent_client_protocol::schema::v1::{
        ToolCallStatus, ToolCallUpdate, ToolCallUpdateFields, UsageUpdate,
    };

    use super::*;

    #[test]
    fn test_merge_consecutive_text_chunks() {
        let mut history = NotificationHistory::new();
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::AgentThoughtChunk(ContentChunk::new(ContentBlock::Text(
                TextContent::new("hello ".to_string()),
            ))),
        ));
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::AgentThoughtChunk(ContentChunk::new(ContentBlock::Text(
                TextContent::new("world".to_string()),
            ))),
        ));
        assert_eq!(history.len(), 1);
        let stored = history.to_vec().pop().unwrap();
        assert!(matches!(stored.update, SessionUpdate::AgentThoughtChunk(_)));
        if let SessionUpdate::AgentThoughtChunk(chunk) = stored.update {
            if let ContentBlock::Text(text) = chunk.content {
                assert_eq!(text.text, "hello world");
            } else {
                panic!("expected text content");
            }
        }
    }

    #[test]
    fn test_different_update_kinds_do_not_merge() {
        let mut history = NotificationHistory::new();
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::UserMessageChunk(ContentChunk::new(ContentBlock::Text(
                TextContent::new("user".to_string()),
            ))),
        ));
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::AgentMessageChunk(ContentChunk::new(ContentBlock::Text(
                TextContent::new("agent".to_string()),
            ))),
        ));
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_history_limit() {
        let mut history = NotificationHistory::new();
        for i in 0..MAX_HISTORY + 5 {
            history.push(SessionNotification::new(
                "s1".to_string(),
                SessionUpdate::UsageUpdate(UsageUpdate::new(i as u64, 100)),
            ));
        }
        assert_eq!(history.len(), MAX_HISTORY);
    }

    #[test]
    fn test_tool_call_updates_accumulate_into_tool_call() {
        let mut history = NotificationHistory::new();
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::ToolCall(ToolCall::new("tc1", "read file")),
        ));
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                "tc1",
                ToolCallUpdateFields::new().status(ToolCallStatus::InProgress),
            )),
        ));
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                "tc1",
                ToolCallUpdateFields::new()
                    .title("done")
                    .status(ToolCallStatus::Completed),
            )),
        ));

        assert_eq!(history.len(), 1);
        assert_eq!(history.tool_call_index_count(), 1);

        let stored = history.to_vec().into_iter().next().unwrap();
        if let SessionUpdate::ToolCall(tool_call) = stored.update {
            assert_eq!(tool_call.tool_call_id, ToolCallId::new("tc1"));
            assert_eq!(tool_call.title, "done");
            assert_eq!(tool_call.status, ToolCallStatus::Completed);
        } else {
            panic!("expected a single ToolCall entry");
        }
    }

    #[test]
    fn test_orphan_tool_call_updates_merge_by_id() {
        let mut history = NotificationHistory::new();
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                "tc1",
                ToolCallUpdateFields::new().status(ToolCallStatus::InProgress),
            )),
        ));
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                "tc1",
                ToolCallUpdateFields::new()
                    .title("done")
                    .status(ToolCallStatus::Completed),
            )),
        ));

        assert_eq!(history.len(), 1);
        assert_eq!(history.tool_call_index_count(), 1);

        let stored = history.to_vec().into_iter().next().unwrap();
        if let SessionUpdate::ToolCallUpdate(update) = stored.update {
            assert_eq!(update.tool_call_id, ToolCallId::new("tc1"));
            assert_eq!(update.fields.status, Some(ToolCallStatus::Completed));
            assert_eq!(update.fields.title.as_deref(), Some("done"));
        } else {
            panic!("expected a single ToolCallUpdate entry");
        }
    }

    #[test]
    fn test_different_tool_calls_remain_separate() {
        let mut history = NotificationHistory::new();
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::ToolCall(ToolCall::new("tc1", "first")),
        ));
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::ToolCall(ToolCall::new("tc2", "second")),
        ));
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                "tc1",
                ToolCallUpdateFields::new().status(ToolCallStatus::Completed),
            )),
        ));

        assert_eq!(history.len(), 2);
        assert_eq!(history.tool_call_index_count(), 2);

        let items = history.to_vec();
        let first = items.iter().find(|n| matches!(&n.update, SessionUpdate::ToolCall(tc) if tc.tool_call_id == ToolCallId::new("tc1"))).unwrap();
        if let SessionUpdate::ToolCall(tc) = &first.update {
            assert_eq!(tc.status, ToolCallStatus::Completed);
        } else {
            panic!("expected ToolCall for tc1");
        }
    }

    #[test]
    fn test_late_tool_call_absorbs_prior_update() {
        let mut history = NotificationHistory::new();
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                "tc1",
                ToolCallUpdateFields::new()
                    .title("late title")
                    .status(ToolCallStatus::Completed),
            )),
        ));
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::ToolCall(ToolCall::new("tc1", "initial")),
        ));

        assert_eq!(history.len(), 1);
        let stored = history.to_vec().into_iter().next().unwrap();
        if let SessionUpdate::ToolCall(tool_call) = stored.update {
            assert_eq!(tool_call.tool_call_id, ToolCallId::new("tc1"));
            // Stored update had a newer title; it overrides the late ToolCall's title.
            assert_eq!(tool_call.title, "late title");
            assert_eq!(tool_call.status, ToolCallStatus::Completed);
        } else {
            panic!("expected ToolCall entry");
        }
    }

    #[test]
    fn test_history_limit_clears_evicted_tool_call_indices() {
        let mut history = NotificationHistory::new();
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::ToolCall(ToolCall::new("tc1", "title")),
        ));
        assert_eq!(history.tool_call_index_count(), 1);

        for i in 0..MAX_HISTORY {
            history.push(SessionNotification::new(
                "s1".to_string(),
                SessionUpdate::UsageUpdate(UsageUpdate::new(i as u64, 0)),
            ));
        }

        assert_eq!(history.len(), MAX_HISTORY);
        assert_eq!(history.tool_call_index_count(), 0);

        // After eviction, a new update for the same id must create a fresh entry
        // rather than merging with the removed tool call.
        history.push(SessionNotification::new(
            "s1".to_string(),
            SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                "tc1",
                ToolCallUpdateFields::new().status(ToolCallStatus::Completed),
            )),
        ));
        assert_eq!(history.len(), MAX_HISTORY);
        assert_eq!(history.tool_call_index_count(), 1);
        let last = history.to_vec().last().cloned().unwrap();
        assert!(matches!(last.update, SessionUpdate::ToolCallUpdate(_)));
    }
}
