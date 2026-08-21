use super::{
    entry::{Entry, EntryId},
    registry::RegistryInterface,
};
use std::{
    any::Any,
    cmp::Ord,
    collections::BTreeMap,
    fmt::{self, Debug},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak},
};

#[derive(Debug)]
pub enum RegistryMapError {
    KeyAlreadyExists,
}

impl fmt::Display for RegistryMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key already exists in registry!")
    }
}

impl std::error::Error for RegistryMapError {}

/// [`RegistryMap`] is a map whose registered elements' lifetimes are controlled by the non-copyable [`Entry`] object.
pub struct RegistryMap<K, T>
where
    T: Send + Sync + 'static,
    K: Ord,
{
    inner: Arc<RwLock<Inner<K, T>>>,
}

// Note: Derive macro is not used here in order to make the implementation independent from T
impl<K, T> Default for RegistryMap<K, T>
where
    T: Send + Sync,
    K: Ord + Send + Sync + Clone + 'static,
{
    fn default() -> Self {
        RegistryMap::new()
    }
}

// Note: Derive macro is not used here in order to make the implementation independent from T
impl<K, T> Clone for RegistryMap<K, T>
where
    T: Send + Sync,
    K: Ord,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, T> Debug for RegistryMap<K, T>
where
    T: Send + Sync + Debug,
    K: Send + Sync + Clone + Ord + Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read().guard.map.fmt(f)
    }
}

impl<K, T> RegistryMap<K, T>
where
    T: Send + Sync,
    K: Ord + Send + Sync + Clone + 'static,
{
    /// Creates a new RegistryMap.
    pub fn new() -> Self {
        RegistryMap {
            inner: Arc::new(RwLock::new(Inner {
                map: BTreeMap::new(),
                entry_map: BTreeMap::new(),
                next_id: 0,
                remove_callback: None,
            })),
        }
    }

    /// Registers an element in the [`RegistryMap`].
    ///
    /// # Returns
    /// [`Entry`] which controls the lifetime of the registered element. If the key already exists, `None` is returned.
    #[must_use = "Entry will be immediately revoked if not used"]
    pub fn register(&self, key: K, value: T) -> Result<Entry<T>, RegistryMapError> {
        let mut lock = self.inner.write().unwrap();

        if lock.map.contains_key(&key) {
            return Err(RegistryMapError::KeyAlreadyExists);
        }

        let entry_id = lock.next_id;
        lock.map.insert(key.clone(), value);
        lock.entry_map.insert(entry_id, key);
        lock.next_id += 1;

        Ok(Entry::<T>::new(
            Arc::downgrade(&self.inner) as Weak<RwLock<dyn RegistryInterface + 'static>>,
            entry_id,
        ))
    }

    /// Creates a [`RegistryMapReadGuard`] which can be used to read the contents of the RegistryMap.
    pub fn read(&self) -> RegistryMapReadGuard<K, T> {
        RegistryMapReadGuard::<K, T> {
            guard: self.inner.read().unwrap(),
        }
    }

    /// Creates a [`RegistryMapWriteGuard`] which can be used to write the contents of the RegistryMap.
    pub fn write(&self) -> RegistryMapWriteGuard<K, T> {
        RegistryMapWriteGuard::<K, T> {
            guard: self.inner.write().unwrap(),
        }
    }

    /// Returns the number of elements in the RegistryMap.
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().map.len()
    }

    /// Returns true if the RegistryMap contains no elements.
    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().map.is_empty()
    }

    /// Sets a remove callback for the RegistryMap. \
    /// Note: If you call this multiple times. It will override the previous callback.
    pub fn set_remove_callback<C>(&self, callback: C)
    where
        C: FnMut(EntryId, K, T) + Send + Sync + 'static,
    {
        self.inner.write().unwrap().remove_callback = Some(Box::new(callback))
    }
}

#[derive(Default)]
struct Inner<K, T>
where
    T: Send + Sync,
{
    map: BTreeMap<K, T>,
    entry_map: BTreeMap<EntryId, K>,
    next_id: EntryId,
    remove_callback: Option<Box<dyn FnMut(EntryId, K, T) + Send + Sync>>,
}

impl<K, T> RegistryInterface for Inner<K, T>
where
    T: Send + Sync + 'static,
    K: Send + Sync + Ord,
{
    fn get(&self, entry_id: u32) -> Option<&dyn Any> {
        let Some(key) = self.entry_map.get(&entry_id) else {
            return None;
        };
        if let Some(value) = self.map.get(key) {
            Some(value)
        } else {
            None
        }
    }
    fn get_mut(&mut self, entry_id: EntryId) -> Option<&mut dyn Any> {
        let Some(key) = self.entry_map.get(&entry_id) else {
            return None;
        };
        if let Some(value) = self.map.get_mut(key) {
            Some(value)
        } else {
            None
        }
    }
    fn remove(&mut self, entry_id: EntryId) {
        let key = self
            .entry_map
            .remove(&entry_id)
            .expect("Failed to find key for EntryId during removal!");
        if let Some(value) = self.map.remove(&key) {
            if let Some(callback) = &mut self.remove_callback {
                callback(entry_id, key, value);
            }
        }
    }
}

/// Holds a read guard to the RegistryMap. See [`RegistryMap::read()`].
pub struct RegistryMapReadGuard<'a, K, T>
where
    T: Send + Sync,
{
    guard: RwLockReadGuard<'a, Inner<K, T>>,
}

impl<'a, K, T> RegistryMapReadGuard<'a, K, T>
where
    T: Send + Sync,
    K: Ord,
{
    /// Acquires an iterator over the RegistryMap.
    pub fn iter(&'a self) -> std::collections::btree_map::Iter<'a, K, T> {
        self.guard.map.iter()
    }

    /// Acquires a reference to an element from the RegistryMap.
    pub fn get(&self, key: &K) -> Option<&T> {
        self.guard.map.get(key)
    }
}

/// Holds a write guard to the RegistryMap. See [`RegistryMap::write()`].
pub struct RegistryMapWriteGuard<'a, K, T>
where
    T: Send + Sync,
{
    guard: RwLockWriteGuard<'a, Inner<K, T>>,
}

impl<'a, K, T> RegistryMapWriteGuard<'a, K, T>
where
    T: Send + Sync,
    K: Ord,
{
    /// Acquires an iterator over the RegistryMap.
    pub fn iter(&'a self) -> std::collections::btree_map::Iter<'a, K, T> {
        self.guard.map.iter()
    }

    /// Acquires a mutable iterator over the RegistryMap.
    pub fn iter_mut(&mut self) -> std::collections::btree_map::IterMut<'_, K, T> {
        self.guard.map.iter_mut()
    }

    /// Acquires a reference to an element in the RegistryMap.
    pub fn get(&self, key: &K) -> Option<&T> {
        self.guard.map.get(key)
    }

    /// Acquires a mutable reference to an element in the RegistryMap.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut T> {
        self.guard.map.get_mut(key)
    }
}
