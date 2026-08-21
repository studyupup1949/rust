use crate::{
    entry::{Entry, EntryId},
    event::{Event, EventObserver},
    registry::{Registry, RegistryReadGuard, RegistryWriteGuard},
};
/// Event types emitted by a traced registry.
#[derive(Clone, Debug)]
pub enum TracedRegistryEvent {
    Register,
    UnRegister,
}

/// [`Registry`] is a vector whose registered elements' lifetimes are controlled by the non-copyable [`Entry`] object.
/// With [`TracedRegistry`] you can register observers via [`TracedRegistry::register_observer()`], which are called whenever an element
/// is registered or unregistered.
pub struct TracedRegistry<T>
where
    T: Send + Sync + Clone + 'static,
{
    registry: Registry<T>,
    event: Event<(TracedRegistryEvent, EntryId, T)>,
}

impl<T> Clone for TracedRegistry<T>
where
    T: Send + Clone + Sync,
{
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            event: self.event.clone(),
        }
    }
}

impl<T> Default for TracedRegistry<T>
where
    T: Send + Sync + Clone + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TracedRegistry<T>
where
    T: Send + Sync + Clone + 'static,
{
    /// Creates a new traced registry.
    pub fn new() -> Self {
        let registry = Registry::new();
        let event = Event::new();
        let event2 = event.clone();
        registry.set_remove_callback(move |entry_id, value| {
            event2.dispatch((TracedRegistryEvent::UnRegister, entry_id, value));
        });
        Self { registry, event }
    }

    /// Registers an element in the [`Registry`].
    ///
    /// # Returns
    /// [`Entry`] which controls the lifetime of the registered element.
    #[must_use = "Entry will be immediately revoked if not used"]
    pub fn register(&self, value: T) -> Entry<T> {
        let entry = self.registry.register(value.clone());
        self.event
            .dispatch((TracedRegistryEvent::Register, entry.get_id(), value));

        entry
    }

    /// Registers an observer to the [`Registry`].
    ///
    /// # Returns
    /// [`Entry`] which controls the lifetime of the observer.
    #[must_use = "Entry will be immediately revoked if not used"]
    pub fn register_observer<O>(&self, observer: O) -> Entry
    where
        O: EventObserver<(TracedRegistryEvent, EntryId, T)> + 'static,
    {
        self.event.register_observer(observer)
    }

    /// Returns the number of elements in the registry.
    pub fn len(&self) -> usize {
        self.registry.len()
    }

    /// Returns true if the registry contains no elements.
    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }

    /// Creates a [`RegistryReadGuard`] which can be used to read the contents of the registry.
    pub fn read(&self) -> RegistryReadGuard<'_, T> {
        self.registry.read()
    }

    /// Creates a [`RegistryWriteGuard`] which can be used to write the contents of the registry.
    pub fn write(&self) -> RegistryWriteGuard<'_, T> {
        self.registry.write()
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

        is_send_sync::<TracedRegistry<i32>>();
        is_clone::<TracedRegistry<i32>>();
    }

    // Define a simple struct for testing
    #[derive(Debug, PartialEq, Clone)]
    struct TestData {
        value: i32,
    }

    #[test]
    fn test_register_and_notify_multiple_observers() {
        // Create a new TracedRegistry
        let registry = TracedRegistry::new();

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
        let _entry1 = registry.register_observer(observer_function1);
        let _entry2 = registry.register_observer(observer_function2);

        // Register a value
        let entry = registry.register(TestData { value: 42 });

        // Check that both observer functions were called once
        assert_eq!(counter1.load(Ordering::Relaxed), 1);
        assert_eq!(counter2.load(Ordering::Relaxed), 1);

        // Remove the registered value
        drop(entry);

        // Check that both observer functions were called twice
        assert_eq!(counter1.load(Ordering::Relaxed), 2);
        assert_eq!(counter2.load(Ordering::Relaxed), 2);
    }
}
