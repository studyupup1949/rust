use super::entry::{Entry, EntryId};
use std::{
    any::Any,
    collections::BTreeMap,
    fmt::Debug,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak},
};

/// [`Registry`] is a container whose registered elements' lifetimes are controlled by the non-copyable [`Entry`] object.
pub struct Registry<T>
where
    T: Send + Sync + 'static,
{
    inner: Arc<RwLock<Inner<T>>>,
}

// Note: Derive macro is not used here in order to make the implementation independent from T
impl<T> Default for Registry<T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        Registry::new()
    }
}

// Note: Derive macro is not used here in order to make the implementation independent from T
impl<T> Clone for Registry<T>
where
    T: Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Debug for Registry<T>
where
    T: Send + Sync + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read().guard.map.fmt(f)
    }
}

impl<T> Registry<T>
where
    T: Send + Sync,
{
    /// Creates a new registry.
    pub fn new() -> Self {
        Registry {
            inner: Arc::new(RwLock::new(Inner {
                map: BTreeMap::new(),
                next_id: 0,
                remove_callback: None,
            })),
        }
    }

    /// Registers an element in the [`Registry`].
    ///
    /// # Returns
    /// [`Entry`] which controls the lifetime of the registered element.
    #[must_use = "Entry will be immediately revoked if not used"]
    pub fn register(&self, value: T) -> Entry<T> {
        let mut lock = self.inner.write().unwrap();

        let entry_id = lock.next_id;
        lock.map.insert(entry_id, value);
        lock.next_id += 1;

        Entry::<T>::new(
            Arc::downgrade(&self.inner) as Weak<RwLock<dyn RegistryInterface>>,
            entry_id,
        )
    }

    /// Creates a [`RegistryReadGuard`] which can be used to read the contents of the registry.
    pub fn read(&self) -> RegistryReadGuard<T> {
        RegistryReadGuard::<T> {
            guard: self.inner.read().unwrap(),
        }
    }

    /// Creates a [`RegistryWriteGuard`] which can be used to write the contents of the registry.
    pub fn write(&self) -> RegistryWriteGuard<T> {
        RegistryWriteGuard::<T> {
            guard: self.inner.write().unwrap(),
        }
    }

    /// Returns the number of elements in the registry.
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().map.len()
    }

    /// Returns true if the registry contains no elements.
    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().map.is_empty()
    }

    /// Sets a remove callback for the registry. \
    /// Note: If you call this multiple times. It will override the previous callback.
    pub fn set_remove_callback<C>(&self, callback: C)
    where
        C: Fn(EntryId, T) + Send + Sync + 'static,
    {
        self.inner.write().unwrap().remove_callback = Some(Box::new(callback))
    }
}

#[derive(Default)]
struct Inner<T>
where
    T: Send + Sync,
{
    map: BTreeMap<EntryId, T>,
    next_id: EntryId,
    remove_callback: Option<Box<dyn Fn(EntryId, T) + Send + Sync>>,
}

impl<T: 'static> RegistryInterface for Inner<T>
where
    T: Send + Sync,
{
    fn get(&self, entry_id: EntryId) -> Option<&dyn Any> {
        if let Some(value) = self.map.get(&entry_id) {
            Some(value)
        } else {
            None
        }
    }
    fn get_mut(&mut self, entry_id: EntryId) -> Option<&mut dyn Any> {
        if let Some(value) = self.map.get_mut(&entry_id) {
            Some(value)
        } else {
            None
        }
    }
    fn remove(&mut self, entry_id: EntryId) {
        if let Some(value) = self.map.remove(&entry_id) {
            if let Some(callback) = &self.remove_callback {
                callback(entry_id, value);
            }
        }
    }
}

/// Holds a read guard to the registry. See [`Registry::read()`].
pub struct RegistryReadGuard<'a, T>
where
    T: Send + Sync,
{
    guard: RwLockReadGuard<'a, Inner<T>>,
}

impl<'a, T> RegistryReadGuard<'a, T>
where
    T: Send + Sync,
{
    /// Acquires an iterator over the registry.
    pub fn iter(&'a self) -> std::collections::btree_map::Iter<'a, EntryId, T> {
        self.guard.map.iter()
    }

    /// Acquires a reference to an element from the registry.
    pub fn get(&'a self, key: EntryId) -> Option<&'a T> {
        self.guard.map.get(&key)
    }
}

/// Holds a write guard to the registry. See [`Registry::write()`].
pub struct RegistryWriteGuard<'a, T: 'static>
where
    T: Send + Sync,
{
    guard: RwLockWriteGuard<'a, Inner<T>>,
}

impl<'a, T> RegistryWriteGuard<'a, T>
where
    T: Send + Sync,
{
    /// Acquires an iterator over the registry.
    pub fn iter(&'a self) -> std::collections::btree_map::Iter<'a, EntryId, T> {
        self.guard.map.iter()
    }

    /// Acquires a mutable iterator to the registry.
    pub fn iter_mut(&mut self) -> std::collections::btree_map::IterMut<'_, EntryId, T> {
        self.guard.map.iter_mut()
    }

    /// Acquires a reference to an element from the registry.
    pub fn get(&'a self, key: EntryId) -> Option<&'a T> {
        self.guard.map.get(&key)
    }

    /// Acquires a mutable reference to an element from the registry.
    pub fn get_mut(&'a mut self, key: EntryId) -> Option<&'a mut T> {
        self.guard.map.get_mut(&key)
    }
}

pub(crate) trait RegistryInterface: Send + Sync {
    fn get(&self, entry_id: EntryId) -> Option<&dyn Any>;
    fn get_mut(&mut self, entry_id: EntryId) -> Option<&mut dyn Any>;
    fn remove(&mut self, entry_id: EntryId);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_entry() {
        let r = Registry::<i32>::new();
        let e1 = r.register(11);
        let e2 = r.register(22);

        assert_eq!(*e1.read().unwrap().get(), 11);
        assert_eq!(*e2.read().unwrap().get(), 22);

        *e1.write().unwrap().get_mut() = 33;
        *e2.write().unwrap().get_mut() = 44;

        assert_eq!(*e1.read().unwrap().get(), 33);
        assert_eq!(*e2.read().unwrap().get(), 44);
    }

    #[test]
    fn test_generic_entry() {
        let r1 = Registry::<i32>::new();
        let r2 = Registry::<bool>::new();
        let mut entries = vec![];
        assert_eq!(r1.len(), 0);
        assert_eq!(r2.len(), 0);
        entries.push(r1.register(11).as_generic());
        assert_eq!(r1.len(), 1);
        assert_eq!(r2.len(), 0);
        entries.push(r2.register(false).as_generic());
        assert_eq!(r1.len(), 1);
        assert_eq!(r2.len(), 1);
        assert_eq!(entries.len(), 2);
        drop(entries);
        assert_eq!(r1.len(), 0);
        assert_eq!(r2.len(), 0);
    }

    #[test]
    fn test_length() {
        let r = Registry::<i32>::new();
        assert_eq!(r.len(), 0);
        let e1 = r.register(0);
        let e2 = r.register(0);
        let e3 = r.register(0);
        let e4 = r.register(0);
        assert_eq!(r.len(), 4);
        drop(e1);
        drop(e2);
        assert_eq!(r.len(), 2);
        drop(e3);
        drop(e4);
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn test_is_empty() {
        let r = Registry::<i32>::new();
        assert!(r.is_empty());
        let e = r.register(0);
        assert!(!r.is_empty());
        drop(e);
        assert!(r.is_empty());
    }

    #[test]
    fn test_registry_iter() {
        let r = Registry::<i32>::new();
        {
            let guard = r.read();
            let mut iter = guard.iter();
            assert_eq!(iter.next(), None);
        }

        let e1 = r.register(11);
        {
            let guard = r.read();
            let mut iter = guard.iter();
            assert_eq!(iter.next(), Some((&0, &11)));
            assert_eq!(iter.next(), None);
        }

        let e2 = r.register(22);
        {
            let guard = r.read();
            let mut iter = guard.iter();
            assert_eq!(iter.next(), Some((&0, &11)));
            assert_eq!(iter.next(), Some((&1, &22)));
            assert_eq!(iter.next(), None);
        }

        let e3 = r.register(33);
        {
            let guard = r.read();
            let mut iter = guard.iter();
            assert_eq!(iter.next(), Some((&0, &11)));
            assert_eq!(iter.next(), Some((&1, &22)));
            assert_eq!(iter.next(), Some((&2, &33)));
            assert_eq!(iter.next(), None);
        }
        drop(e2);

        {
            let guard = r.read();
            let mut iter = guard.iter();
            assert_eq!(iter.next(), Some((&0, &11)));
            assert_eq!(iter.next(), Some((&2, &33)));
            assert_eq!(iter.next(), None);
        }
        drop(e1);
        {
            let guard = r.read();
            let mut iter = guard.iter();
            assert_eq!(iter.next(), Some((&2, &33)));
            assert_eq!(iter.next(), None);
        }
        drop(e3);
        {
            let guard = r.read();
            let mut iter = guard.iter();
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn test_registry_iter_mut() {
        let r = Registry::<i32>::new();
        let entries = [
            r.register(11),
            r.register(22),
            r.register(33),
            r.register(44),
        ];

        for (_, v) in r.write().iter_mut() {
            *v *= 2;
        }

        assert_eq!(*entries[0].read().unwrap().get(), 22);
        assert_eq!(*entries[1].read().unwrap().get(), 44);
        assert_eq!(*entries[2].read().unwrap().get(), 66);
        assert_eq!(*entries[3].read().unwrap().get(), 88);
    }
    #[test]
    fn test_attributes() {
        fn is_send_sync<T: Send + Sync>() {}
        fn is_clone<T: Clone>() {}

        is_send_sync::<Registry<i32>>();
        is_send_sync::<Entry<i32>>();

        is_clone::<Registry<i32>>();
    }

    #[test]
    fn test_arc_entry() {
        let r = Registry::<i32>::new();
        let ae = Arc::new(r.register(11));
        let ae2 = ae.clone();
        assert_eq!(r.len(), 1);
        drop(ae);
        assert_eq!(r.len(), 1);
        drop(ae2);
        assert_eq!(r.len(), 0);
    }
    #[test]
    #[should_panic(expected = "Failed to downcast Entry")]
    fn test_generic_entry_read() {
        let r = Registry::<i32>::new();
        let entry = r.register(11).as_generic();
        entry.read().unwrap().get();
    }
    #[test]
    #[should_panic(expected = "Failed to downcast Entry")]
    fn test_generic_entry_write() {
        let r = Registry::<i32>::new();
        let entry = r.register(11).as_generic();
        entry.write().unwrap().get();
    }
    #[test]
    #[should_panic(expected = "Failed to downcast Entry")]
    fn test_generic_entry_write_mut() {
        let r = Registry::<i32>::new();
        let entry = r.register(11).as_generic();
        entry.write().unwrap().get_mut();
    }

    #[test]
    fn test_short_lived_registry() {
        let r = Registry::<i32>::new();
        let entry = r.register(11);
        assert!(entry.write().is_some());
        drop(r);
        assert!(entry.write().is_none());
    }
}
