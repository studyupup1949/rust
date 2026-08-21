use std::fmt;
use std::hash::{BuildHasher, Hash, RandomState};

use tokio::sync::{mpsc, oneshot};

use super::messages::ActorMessage;
use super::shard::spawn_shard;

/// Actor-style async map.
///
/// Similar to DashMap's common API surface, but all operations are async and
/// values are copied/cloned across channel boundaries.
pub struct AcMap<K, V> {
    shards: Vec<mpsc::UnboundedSender<ActorMessage<K, V>>>,
    hasher: RandomState,
    shard_mask: usize,
}

impl<K, V> fmt::Debug for AcMap<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AcMap").finish_non_exhaustive()
    }
}

impl<K, V> Clone for AcMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            shards: self.shards.clone(),
            hasher: self.hasher.clone(),
            shard_mask: self.shard_mask,
        }
    }
}

impl<K, V> Default for AcMap<K, V>
where
    K: Eq + Hash + Clone + Send + 'static,
    V: Clone + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> AcMap<K, V>
where
    K: Eq + Hash + Clone + Send + 'static,
    V: Clone + Send + 'static,
{
    pub fn new() -> Self {
        Self::with_shards_and_capacity(default_shard_count(), 0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_shards_and_capacity(default_shard_count(), capacity)
    }

    pub fn with_shards_and_capacity(shards: usize, capacity: usize) -> Self {
        let shard_count = normalize_shard_count(shards);
        let per_shard_capacity = capacity.div_ceil(shard_count);
        let mut txs = Vec::with_capacity(shard_count);

        for _ in 0..shard_count {
            txs.push(spawn_shard(per_shard_capacity));
        }

        Self {
            shards: txs,
            hasher: RandomState::new(),
            shard_mask: shard_count - 1,
        }
    }

    fn shard_index(&self, key: &K) -> usize {
        (self.hasher.hash_one(key) as usize) & self.shard_mask
    }

    fn shard_tx(&self, key: &K) -> &mpsc::UnboundedSender<ActorMessage<K, V>> {
        &self.shards[self.shard_index(key)]
    }

    pub async fn insert(&self, key: K, value: V) -> Option<V> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.shard_tx(&key)
            .send(ActorMessage::Insert {
                key,
                value,
                resp: resp_tx,
            })
            .expect("AcMap actor task is not running");

        resp_rx.await.expect("AcMap actor dropped response channel")
    }

    pub fn insert_fast(&self, key: K, value: V) {
        self.shard_tx(&key)
            .send(ActorMessage::InsertFast { key, value })
            .expect("AcMap actor task is not running");
    }

    pub fn insert_fast_batch<I>(&self, entries: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut buckets: Vec<Vec<(K, V)>> = (0..self.shards.len()).map(|_| Vec::new()).collect();

        for (key, value) in entries {
            let idx = self.shard_index(&key);
            buckets[idx].push((key, value));
        }

        for (idx, batch) in buckets.into_iter().enumerate() {
            if batch.is_empty() {
                continue;
            }
            self.shards[idx]
                .send(ActorMessage::InsertFastBatch { entries: batch })
                .expect("AcMap actor task is not running");
        }
    }

    pub async fn get(&self, key: K) -> Option<V> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.shard_tx(&key)
            .send(ActorMessage::Get { key, resp: resp_tx })
            .expect("AcMap actor task is not running");

        resp_rx.await.expect("AcMap actor dropped response channel")
    }

    pub async fn remove(&self, key: K) -> Option<(K, V)> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.shard_tx(&key)
            .send(ActorMessage::Remove { key, resp: resp_tx })
            .expect("AcMap actor task is not running");

        resp_rx.await.expect("AcMap actor dropped response channel")
    }

    pub async fn len(&self) -> usize {
        let mut receivers = Vec::with_capacity(self.shards.len());

        for tx in &self.shards {
            let (resp_tx, resp_rx) = oneshot::channel();
            tx.send(ActorMessage::Len { resp: resp_tx })
                .expect("AcMap actor task is not running");
            receivers.push(resp_rx);
        }

        let mut total = 0usize;
        for rx in receivers {
            total += rx.await.expect("AcMap actor dropped response channel");
        }

        total
    }

    pub async fn is_empty(&self) -> bool {
        let mut receivers = Vec::with_capacity(self.shards.len());

        for tx in &self.shards {
            let (resp_tx, resp_rx) = oneshot::channel();
            tx.send(ActorMessage::IsEmpty { resp: resp_tx })
                .expect("AcMap actor task is not running");
            receivers.push(resp_rx);
        }

        for rx in receivers {
            if !rx.await.expect("AcMap actor dropped response channel") {
                return false;
            }
        }

        true
    }

    pub async fn contains_key(&self, key: K) -> bool {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.shard_tx(&key)
            .send(ActorMessage::ContainsKey { key, resp: resp_tx })
            .expect("AcMap actor task is not running");

        resp_rx.await.expect("AcMap actor dropped response channel")
    }

    pub fn clear(&self) {
        for tx in &self.shards {
            tx.send(ActorMessage::Clear)
                .expect("AcMap actor task is not running");
        }
    }
}

fn default_shard_count() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .max(4)
}

fn normalize_shard_count(shards: usize) -> usize {
    let n = shards.max(1);
    if n.is_power_of_two() {
        n
    } else {
        n.next_power_of_two()
    }
}
