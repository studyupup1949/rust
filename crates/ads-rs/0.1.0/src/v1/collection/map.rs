pub trait Map<K: Ord, V> {
    fn get(&self, key: &K) -> Option<V>;
    fn erase(&mut self, key: &K) -> Option<V>;
    fn insert(&mut self, key: K, value: V) -> Option<V>;
}
