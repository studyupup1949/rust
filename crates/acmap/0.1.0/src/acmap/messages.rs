use tokio::sync::oneshot;

pub(super) enum ActorMessage<K, V> {
    InsertFast {
        key: K,
        value: V,
    },
    InsertFastBatch {
        entries: Vec<(K, V)>,
    },
    Insert {
        key: K,
        value: V,
        resp: oneshot::Sender<Option<V>>,
    },
    Get {
        key: K,
        resp: oneshot::Sender<Option<V>>,
    },
    Remove {
        key: K,
        resp: oneshot::Sender<Option<(K, V)>>,
    },
    Len {
        resp: oneshot::Sender<usize>,
    },
    IsEmpty {
        resp: oneshot::Sender<bool>,
    },
    ContainsKey {
        key: K,
        resp: oneshot::Sender<bool>,
    },
    Clear,
}
