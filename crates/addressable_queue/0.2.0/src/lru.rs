// Copyright 2018 Leonardo Schwarz <mail@leoschwarz.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use fifo;
use std::hash::Hash;

/// An addressable LRU queue.
pub struct Queue<K, V> {
    inner: fifo::Queue<K, V>,
}

impl<K, V> Queue<K, V>
where
    K: Clone + Eq + Hash
{
    /// Create a new instance of the queue.
    pub fn new() -> Self {
        Queue {
            inner: fifo::Queue::new(),
        }
    }

    // TODO
    /// Access an entry. If it exists it will also be moved to the end of the queue.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.inner.remove_key(key).map(|item| {
            let item_ref = &item;
            self.insert(key.clone(), item);
            item_ref
        })
    }

    /// Insert an entry at the end of the queue.
    pub fn insert(&mut self, key: K, value: V) {
        self.inner.insert(key, value);
    }

    /// Insert an entry at the beginning of the queue.
    pub fn insert_head(&mut self, key: K, value: V) {
        self.inner.insert(key, value);
    }

    /// Remove the current head of the queue, and return the value if there was one.
    pub fn remove_head(&mut self) -> Option<V> {
        self.inner.remove_head()
    }

    /// Remove the current tail of the queue, and return the value if there was one.
    pub fn remove_tail(&mut self) -> Option<V> {
        self.inner.remove_tail()
    }

    /// Remove a value by specifying its key.
    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        self.inner.remove_key(key)
    }
}
