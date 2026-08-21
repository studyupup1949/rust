use crate::registry::RegistryInterface;
use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Deref,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak},
};

/// Entry index type
pub type EntryId = u32;

/// Entry controls the lifetime of an entry in the registry. When the entry has its original
/// type definition, you can also use it to access the stored object. See [`crate::registry::Registry::register()`].
pub struct Entry<T = ()> {
    iface: Weak<RwLock<dyn RegistryInterface + 'static>>,
    id: EntryId,
    phantom: PhantomData<T>,
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{:?}", self.id)
    }
}

impl<T> Entry<T>
where
    T: Send + Sync,
{
    pub(crate) fn new(iface: Weak<RwLock<dyn RegistryInterface>>, id: EntryId) -> Self {
        Self {
            iface,
            id,
            phantom: PhantomData,
        }
    }

    /// Removes the type definition from the Entry. This method is useful when you want to store different
    /// kinds of [`Entry`] in one collection.
    pub fn as_generic(self) -> Entry {
        let maybe_uninit = MaybeUninit::new(self);
        let ptr = maybe_uninit.as_ptr();
        unsafe {
            // Note: Converting Entry<T> to Entry without calling drop. Drop will be called by type erased Entry later on...
            Entry {
                iface: std::ptr::read(&(*ptr).iface),
                id: std::ptr::read(&(*ptr).id),
                phantom: PhantomData,
            }
        }
    }

    /// Grants mutable access to the entry. It locks the shared [`RwLock`] of the [`crate::registry::Registry`]. Blocks the current thread until the
    /// lock can be acquired!
    /// # Return
    /// [`None`] if the [`crate::registry::Registry`] no longer exists.
    pub fn write(&self) -> Option<EntryWriteGuard<T>> {
        let registry = self.iface.upgrade()?;
        let ptr = self.iface.as_ptr();
        // Note: The acquired pointer will be valid as long as a strong reference is alive.
        // Using a pointer is required because RwLock.write() would partially borrow the registry making it impossible
        // to create an object containing both a strong pointer and a lock guard.
        let reference = unsafe { &*ptr };
        Some(EntryWriteGuard::<T> {
            _registry: registry,
            guard: reference.write().unwrap(),
            entry_id: self.id,
            phantom: PhantomData,
        })
    }

    /// Grants shared read access to the entry. It locks the shared [`RwLock`] of the [`crate::registry::Registry`]. Blocks the current thread until the
    /// lock can be acquired!
    /// # Return
    /// [`None`] if the [`crate::registry::Registry`] no longer exists.
    pub fn read(&self) -> Option<EntryReadGuard<T>> {
        let registry = self.iface.upgrade()?;
        let ptr = self.iface.as_ptr();
        // Note: The acquired pointer will be valid as long as a strong reference is alive.
        // Using a pointer is required because RwLock.read() would partially borrow the registry making it impossible
        // to create an object containing both a strong pointer and a lock guard.
        let reference = unsafe { &*ptr };
        Some(EntryReadGuard::<T> {
            _registry: registry,
            guard: reference.read().unwrap(),
            entry_id: self.id,
            phantom: PhantomData,
        })
    }

    /// Gets the underlying id of the entry.
    pub fn get_id(&self) -> EntryId {
        self.id
    }

    /// Leaks the entry. \
    /// ⚠️ In production environments you should never use this method. It's only meant for quick prototyping or debugging.
    pub unsafe fn leak(self) {
        std::mem::forget(self);
    }
}

impl<T> Drop for Entry<T> {
    #[inline(always)]
    fn drop(&mut self) {
        if let Some(arc) = self.iface.upgrade() {
            if let Ok(mut guard) = arc.write() {
                guard.remove(self.id);
            }
        }
    }
}

/// Holds a write guard to the entry. See [`Entry::write()`].
pub struct EntryWriteGuard<'a, T> {
    _registry: Arc<RwLock<dyn RegistryInterface>>,
    guard: RwLockWriteGuard<'a, dyn RegistryInterface + 'static>,
    entry_id: EntryId,
    phantom: PhantomData<T>,
}

impl<T: 'static> EntryWriteGuard<'_, T> {
    /// Acquires a reference to the entry.
    pub fn get(&self) -> &T {
        self.guard
            .get(self.entry_id)
            .expect("Entry not found in the Registry")
            .downcast_ref::<T>()
            .expect("Failed to downcast Entry")
    }

    /// Acquires a mutable reference to the entry.
    pub fn get_mut(&mut self) -> &mut T {
        self.guard
            .get_mut(self.entry_id)
            .expect("Entry not found in the Registry")
            .downcast_mut::<T>()
            .expect("Failed to downcast Entry")
    }
}

/// Holds a read guard to the entry. See [`Entry::read()`].
pub struct EntryReadGuard<'a, T> {
    _registry: Arc<RwLock<dyn RegistryInterface>>,
    guard: RwLockReadGuard<'a, dyn RegistryInterface + 'static>,
    entry_id: EntryId,
    phantom: PhantomData<T>,
}

impl<'a, T: 'static> Deref for EntryReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard
            .get(self.entry_id)
            .expect("Entry not found in the Registry")
            .downcast_ref::<T>()
            .expect("Failed to downcast Entry")
    }
}

impl<T: 'static> EntryReadGuard<'_, T> {
    /// Acquires a reference to the entry.
    pub fn get(&self) -> &T {
        self.guard
            .get(self.entry_id)
            .expect("Entry not found in the Registry")
            .downcast_ref::<T>()
            .expect("Failed to downcast Entry")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(size_of::<Entry>(), 24);
        assert_eq!(size_of::<EntryReadGuard<()>>(), 48);
    }
}
