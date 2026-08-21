use indexmap::IndexMap;
use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash},
};

pub struct MergeMap<K, V, MergeFn, S = RandomState>
where
    K: Eq + Hash,
    MergeFn: FnMut(&mut V, V),
    S: BuildHasher,
{
    map: IndexMap<K, V, S>,
    merge_fn: MergeFn,
}

#[allow(dead_code)]
impl<K, V, MergeFn, S> MergeMap<K, V, MergeFn, S>
where
    K: Eq + Hash,
    MergeFn: FnMut(&mut V, V),
    S: BuildHasher + Default,
{
    pub fn new(map: IndexMap<K, V, S>, merge_fn: MergeFn) -> Self {
        Self { map, merge_fn }
    }

    pub fn with_default(merge_fn: MergeFn) -> Self {
        Self {
            map: IndexMap::with_hasher(S::default()),
            merge_fn,
        }
    }

    pub fn upsert(&mut self, key: K, value: V) {
        if let Some(existing) = self.map.get_mut(&key) {
            (self.merge_fn)(existing, value);
        } else {
            self.map.insert(key, value);
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    pub fn into_inner(self) -> IndexMap<K, V, S> {
        self.map
    }
}
