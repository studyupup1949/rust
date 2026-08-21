use crate::{
    entry::{Entry, EntryId},
    event::{Event, EventObserver},
    registry_map::{RegistryMap, RegistryMapError, RegistryMapReadGuard, RegistryMapWriteGuard},
};

/// Event types emitted by a traced registry map.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TracedRegistryMapEvent {
    Register,
    UnRegister,
}

/// [`RegistryMap`] is a map whose registered elements' lifetimes are controlled by the non-copyable [`Entry`] object.
/// With [`TracedRegistryMap`] you can register observers via [`TracedRegistryMap::register_observer()`], which are called whenever an element
/// is registered or unregistered.
pub struct TracedRegistryMap<K, T>
where
    T: Send + Sync + Clone + 'static,
    K: Ord + Send + Sync + Clone + 'static,
{
    registry_map: RegistryMap<K, T>,
    event: Event<(TracedRegistryMapEvent, EntryId, K, T)>,
}

impl<K, T> Clone for TracedRegistryMap<K, T>
where
    T: Send + Clone + Sync,
    K: Ord + Send + Sync + Clone,
{
    fn clone(&self) -> Self {
        Self {
            registry_map: self.registry_map.clone(),
            event: self.event.clone(),
        }
    }
}

impl<K, T> Default for TracedRegistryMap<K, T>
where
    T: Send + Sync + Clone + 'static,
    K: Ord + Send + Sync + Clone + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, T> TracedRegistryMap<K, T>
where
    T: Send + Sync + Clone + 'static,
    K: Ord + Send + Sync + Clone + 'static,
{
    /// Creates a new traced registry map.
    pub fn new() -> Self {
        let registry_map = RegistryMap::new();
        let event = Event::new();
        let event2 = event.clone();
        registry_map.set_remove_callback(move |entry_id, key, value| {
            event2.dispatch((TracedRegistryMapEvent::UnRegister, entry_id, key, value));
        });
        Self {
            registry_map,
            event,
        }
    }

    /// Registers an element in the [`RegistryMap`].
    ///
    /// # Returns
    /// [`Entry`] which controls the lifetime of the registered element. If the key already exists, `Err(RegistryMapError::KeyAlreadyExists)` is returned.
    #[must_use = "Entry will be immediately revoked if not used"]
    pub fn register(&self, key: K, value: T) -> Result<Entry<T>, RegistryMapError> {
        let entry = self.registry_map.register(key.clone(), value.clone())?;
        self.event
            .dispatch((TracedRegistryMapEvent::Register, entry.get_id(), key, value));

        Ok(entry)
    }

    /// Registers an observer to the [`RegistryMap`].
    ///
    /// # Returns
    /// [`Entry`] which controls the lifetime of the observer.
    #[must_use = "Entry will be immediately revoked if not used"]
    pub fn register_observer<O>(&self, observer: O) -> Entry
    where
        O: EventObserver<(TracedRegistryMapEvent, EntryId, K, T)> + 'static,
    {
        self.event.register_observer(observer)
    }

    /// Returns the number of elements in the registry map.
    pub fn len(&self) -> usize {
        self.registry_map.len()
    }

    /// Returns true if the registry map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.registry_map.is_empty()
    }

    /// Creates a [`RegistryMapReadGuard`] which can be used to read the contents of the registry map.
    pub fn read(&self) -> RegistryMapReadGuard<'_, K, T> {
        self.registry_map.read()
    }

    /// Creates a [`RegistryMapWriteGuard`] which can be used to write the contents of the registry map.
    pub fn write(&self) -> RegistryMapWriteGuard<'_, K, T> {
        self.registry_map.write()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_attributes() {
        fn is_send_sync<T: Send + Sync>() {}
        fn is_clone<T: Clone>() {}

        is_send_sync::<TracedRegistryMap<i32, i32>>();
        is_clone::<TracedRegistryMap<i32, i32>>();
    }

    // Define a simple struct for testing
    #[derive(Debug, PartialEq, Clone)]
    struct TestData {
        value: i32,
    }

    #[test]
    fn test_register_and_notify_multiple_observers() {
        // Create a new TracedRegistryMap
        let registry_map = TracedRegistryMap::new();

        // Create variables to track the number of times each observer is called
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));
        let counter1_clone = Arc::clone(&counter1);
        let counter2_clone = Arc::clone(&counter2);

        // Define observer functions that increment the counters
        let observer_function1 = move |_: &_| {
            counter1_clone.fetch_add(1, Ordering::Relaxed);
        };
        let observer_function2 = move |_: &_| {
            counter2_clone.fetch_add(1, Ordering::Relaxed);
        };

        // Register the observers
        let _entry1 = registry_map.register_observer(observer_function1);
        let _entry2 = registry_map.register_observer(observer_function2);

        // Register a value
        let entry = registry_map.register(42, TestData { value: 100 }).unwrap();

        // Check that both observer functions were called once
        assert_eq!(counter1.load(Ordering::Relaxed), 1);
        assert_eq!(counter2.load(Ordering::Relaxed), 1);

        // Remove the registered value
        drop(entry);

        // Check that both observer functions were called twice
        assert_eq!(counter1.load(Ordering::Relaxed), 2);
        assert_eq!(counter2.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_duplicate_key_error() {
        let registry_map = TracedRegistryMap::<i32, i32>::new();
        let _entry1 = registry_map.register(42, 100).unwrap();
        let result = registry_map.register(42, 200);
        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryMapError::KeyAlreadyExists)));
    }
}
