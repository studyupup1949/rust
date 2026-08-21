//! RAII capability handle for owned namespaces.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use uuid::Uuid;

use crate::registry::{self, OwnedNamespaceSignature, WriteAccess};
use crate::{ContextString, NamespaceString, UuidPoolError};

struct OwnedNamespaceInner {
    namespace: parking_lot::Mutex<NamespaceString>,
    signature: OwnedNamespaceSignature,
    /// Set once this handle group has explicitly released its namespace via
    /// [`OwnedNamespace::remove`], so all clones immediately stop writing instead of
    /// silently re-creating the namespace as unowned through auto-create-on-write.
    invalidated: AtomicBool,
}

impl Drop for OwnedNamespaceInner {
    fn drop(&mut self) {
        // If ownership was already released explicitly, there is nothing left to clean up.
        if self.invalidated.load(Ordering::Acquire) {
            return;
        }
        let namespace = self.namespace.get_mut().clone();
        // Best-effort: only removes data/ownership if the stored signature still matches
        // ours, so a stale handle can never delete a namespace re-claimed by another owner.
        let _ = registry::release_owned_namespace(&namespace, &self.signature);
    }
}

/// A cloneable, opaque write-authorization capability for a namespace claimed via
/// [`crate::interface::reserve_owned_namespace`].
///
/// All clones of an `OwnedNamespace` share the same underlying authorization and namespace
/// data: any clone can write to the namespace, and renaming updates the name observed by
/// every clone. The namespace, its data, and its ownership record are removed automatically
/// once the last clone is dropped, or immediately via [`OwnedNamespace::remove`].
///
/// Free functions in [`crate::interface`] cannot write to a namespace owned by an
/// `OwnedNamespace` handle; they return [`UuidPoolError::UnauthorizedNamespaceWriteAccessError`]
/// instead. Only the handle's own methods (and clones of it) are authorized.
pub struct OwnedNamespace {
    inner: Arc<OwnedNamespaceInner>,
}

impl Clone for OwnedNamespace {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl OwnedNamespace {
    pub(crate) fn new(namespace: NamespaceString, signature: OwnedNamespaceSignature) -> Self {
        Self {
            inner: Arc::new(OwnedNamespaceInner {
                namespace: parking_lot::Mutex::new(namespace),
                signature,
                invalidated: AtomicBool::new(false),
            }),
        }
    }

    fn access(&self) -> WriteAccess<'_> {
        WriteAccess::owned(&self.inner.signature)
    }

    fn current_namespace(&self) -> NamespaceString {
        self.inner.namespace.lock().clone()
    }

    fn ensure_active(&self) -> Result<(), UuidPoolError> {
        if self.inner.invalidated.load(Ordering::Acquire) {
            return Err(UuidPoolError::UnauthorizedNamespaceWriteAccessError(format!(
                "Namespace '{}' ownership has already been released",
                self.current_namespace()
            )));
        }
        Ok(())
    }

    /// Returns the namespace name currently owned by this handle.
    pub fn namespace(&self) -> NamespaceString {
        self.current_namespace()
    }

    /// Reserves a new UUID in this namespace using [`crate::interface::DEFAULT_UUID_BASE`]
    /// and [`crate::interface::DEFAULT_MAX_RETRIES`].
    pub fn reserve_id(&self, context: &str) -> Result<Uuid, UuidPoolError> {
        self.reserve_id_with(context, crate::interface::DEFAULT_UUID_BASE, crate::interface::DEFAULT_MAX_RETRIES)
    }

    /// Reserves a new UUID in this namespace with a custom base.
    pub fn reserve_id_with_base(&self, context: &str, base: u32) -> Result<Uuid, UuidPoolError> {
        self.reserve_id_with(context, base, crate::interface::DEFAULT_MAX_RETRIES)
    }

    /// Reserves a new UUID in this namespace with a custom base and retry count.
    pub fn reserve_id_with(&self, context: &str, base: u32, max_retries: usize) -> Result<Uuid, UuidPoolError> {
        self.ensure_active()?;
        registry::random_uuid(&self.current_namespace(), context, base, max_retries, 0, self.access())
    }

    /// Adds an existing UUID to this namespace and context space.
    pub fn add_id(&self, context: &str, uuid: Uuid) -> Result<(), UuidPoolError> {
        self.ensure_active()?;
        registry::add_uuid_to_pool(&self.current_namespace(), context, &uuid, self.access())
    }

    /// Removes an existing UUID from this namespace and context space.
    pub fn remove_id(&self, context: &str, uuid: Uuid) -> Result<(), UuidPoolError> {
        self.ensure_active()?;
        registry::remove_uuid_from_pool(&self.current_namespace(), context, &uuid, self.access())
    }

    /// Replaces an existing UUID with a new UUID within a context.
    pub fn replace_id(&self, context: &str, old_uuid: Uuid, new_uuid: Uuid) -> Result<(), UuidPoolError> {
        self.ensure_active()?;
        registry::replace_uuid_in_pool(&self.current_namespace(), context, &old_uuid, &new_uuid, self.access())
    }

    /// Clears the given context and all associated UUIDs.
    pub fn clear_context(&self, context: &str) -> Result<(), UuidPoolError> {
        self.ensure_active()?;
        registry::clear_context(&self.current_namespace(), context, self.access())
    }

    /// Clears all contexts (and their UUIDs) within this namespace, retaining the namespace itself.
    pub fn clear_all_contexts(&self) -> Result<(), UuidPoolError> {
        self.ensure_active()?;
        registry::clear_all_contexts(&self.current_namespace(), self.access())
    }

    /// Removes and returns all UUIDs from the given context.
    pub fn drain_context(&self, context: &str) -> Result<Vec<(ContextString, Uuid)>, UuidPoolError> {
        self.ensure_active()?;
        registry::drain_context(&self.current_namespace(), context, self.access())
    }

    /// Removes and returns all contexts (and their UUIDs) within this namespace.
    pub fn drain_all_contexts(&self) -> Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError> {
        self.ensure_active()?;
        registry::drain_all_contexts(&self.current_namespace(), self.access())
    }

    /// Renames this namespace. The new name must be absent or empty, and not owned by
    /// another handle. The new name is observed by every clone of this handle.
    pub fn rename(&self, new_namespace: &str) -> Result<(), UuidPoolError> {
        self.ensure_active()?;
        let mut guard = self.inner.namespace.lock();
        registry::replace_namespace(&guard, new_namespace, self.access())?;
        *guard = new_namespace.to_string();
        Ok(())
    }

    /// Explicitly removes this namespace, its data, and its ownership record.
    ///
    /// Immediately invalidates every clone of this handle: subsequent calls on any clone
    /// return [`UuidPoolError::UnauthorizedNamespaceWriteAccessError`] instead of silently
    /// re-creating the namespace as unowned.
    pub fn remove(self) -> Result<(), UuidPoolError> {
        self.ensure_active()?;
        let namespace = self.current_namespace();
        registry::release_owned_namespace(&namespace, &self.inner.signature)?;
        self.inner.invalidated.store(true, Ordering::Release);
        Ok(())
    }
}
