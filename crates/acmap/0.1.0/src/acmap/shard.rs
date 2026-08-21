use std::collections::HashMap;
use std::hash::Hash;

use tokio::sync::mpsc;

use super::messages::ActorMessage;

pub(super) fn spawn_shard<K, V>(capacity: usize) -> mpsc::UnboundedSender<ActorMessage<K, V>>
where
    K: Eq + Hash + Clone + Send + 'static,
    V: Clone + Send + 'static,
{
    let (tx, mut rx) = mpsc::unbounded_channel::<ActorMessage<K, V>>();

    tokio::spawn(async move {
        let mut state: HashMap<K, V> = HashMap::with_capacity(capacity);

        while let Some(message) = rx.recv().await {
            match message {
                ActorMessage::InsertFast { key, value } => {
                    state.insert(key, value);
                }
                ActorMessage::InsertFastBatch { entries } => {
                    for (key, value) in entries {
                        state.insert(key, value);
                    }
                }
                ActorMessage::Insert { key, value, resp } => {
                    let prev = state.insert(key, value);
                    let _ = resp.send(prev);
                }
                ActorMessage::Get { key, resp } => {
                    let value = state.get(&key).cloned();
                    let _ = resp.send(value);
                }
                ActorMessage::Remove { key, resp } => {
                    let removed = state.remove_entry(&key);
                    let _ = resp.send(removed);
                }
                ActorMessage::Len { resp } => {
                    let _ = resp.send(state.len());
                }
                ActorMessage::IsEmpty { resp } => {
                    let _ = resp.send(state.is_empty());
                }
                ActorMessage::ContainsKey { key, resp } => {
                    let _ = resp.send(state.contains_key(&key));
                }
                ActorMessage::Clear => {
                    state.clear();
                }
            }
        }
    });

    tx
}
