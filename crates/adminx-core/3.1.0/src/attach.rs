// adminx-core/src/attach.rs
//
// The file-attachment seam. A resource can declare `file_fields()`; the panel
// then renders an upload widget on the detail page and the web adapters expose
// attach / serve / detach routes. The actual bytes live behind a pluggable
// `Attachments` backend registered once — exactly like `storage`, `authz` and
// `audit`. Core never names object storage, S3, or a filesystem.
//
// Uploads ride a *dedicated* endpoint, not the create/edit form, so the existing
// URL-encoded form pipeline is untouched: a file is attached to a record that
// already exists.

use crate::error::CoreError;
use crate::response::ApiResponse;
use crate::storage::StorageError;
use async_trait::async_trait;
use once_cell::sync::OnceCell;

/// A file field a resource exposes on its detail page. Declared from
/// [`Resource::file_fields`](crate::resource::Resource::file_fields).
#[derive(Debug, Clone)]
pub struct FileField {
    /// Stable key for the field, used in the attach/serve URLs (e.g. `avatar`).
    pub name: String,
    /// Human label shown above the widget.
    pub label: String,
    /// Restrict what the file picker offers (the `accept` attribute), e.g.
    /// `"image/*"`. Empty means anything.
    pub accept: String,
}

impl FileField {
    pub fn new(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            accept: String::new(),
        }
    }

    /// Only offer images in the picker. Advisory — the browser hint, not a
    /// server-side content check.
    pub fn images(mut self) -> Self {
        self.accept = "image/*".into();
        self
    }

    pub fn accept(mut self, accept: impl Into<String>) -> Self {
        self.accept = accept.into();
        self
    }
}

/// A file arriving on an upload request. The adapter builds this from the
/// multipart body; `bytes` is the whole file held in memory (fine for the admin
/// use-case — logos, avatars, small docs — a streaming path can come later).
#[derive(Clone)]
pub struct UploadedFile {
    pub filename: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

/// One stored attachment, as the detail page and the serve route need it.
#[derive(Debug, Clone)]
pub struct Attachment {
    pub field: String,
    pub filename: String,
    pub content_type: String,
    pub byte_size: u64,
    /// Opaque key the backend uses to fetch the bytes.
    pub storage_key: String,
}

/// A pluggable attachment backend: stores the bytes somewhere and records the
/// metadata so it can be listed and served back.
#[async_trait]
pub trait Attachments: Send + Sync {
    /// Store (or replace) the file attached to `owner_type`/`owner_id` under
    /// `field`, returning the stored metadata.
    async fn put(
        &self,
        owner_type: &str,
        owner_id: &str,
        field: &str,
        file: UploadedFile,
    ) -> Result<Attachment, StorageError>;

    /// The attachment stored under a single field, if any.
    async fn get(
        &self,
        owner_type: &str,
        owner_id: &str,
        field: &str,
    ) -> Result<Option<Attachment>, StorageError>;

    /// Every attachment for a record, across all fields.
    async fn list(
        &self,
        owner_type: &str,
        owner_id: &str,
    ) -> Result<Vec<Attachment>, StorageError>;

    /// Fetch the raw bytes for a stored attachment.
    async fn read(&self, storage_key: &str) -> Result<Vec<u8>, StorageError>;

    /// Remove one field's attachment (bytes and metadata). A no-op if absent.
    async fn delete(
        &self,
        owner_type: &str,
        owner_id: &str,
        field: &str,
    ) -> Result<(), StorageError>;

    /// Remove *all* attachments for a record. Called when the record itself is
    /// deleted, so blobs don't outlive their owner.
    async fn delete_all(&self, owner_type: &str, owner_id: &str) -> Result<(), StorageError>;
}

static ATTACHMENTS: OnceCell<Box<dyn Attachments>> = OnceCell::new();

/// Register the global attachment backend. Set-once, matching the other seams.
pub fn set_attachments(backend: Box<dyn Attachments>) {
    if ATTACHMENTS.set(backend).is_err() {
        tracing::warn!("adminx attachments backend already initialized; ignoring reset");
    }
}

/// The registered backend, if any.
pub fn attachments() -> Option<&'static dyn Attachments> {
    ATTACHMENTS.get().map(|b| b.as_ref())
}

/// Whether attachment support is on. The detail page checks this before
/// rendering upload widgets, and `crud::delete` before trying to purge.
pub fn is_enabled() -> bool {
    ATTACHMENTS.get().is_some()
}

/// The error returned when a file operation is attempted with no backend
/// registered — a misconfiguration (a resource declared `file_fields()` but the
/// app never called `adminx_storage::init`).
fn not_configured() -> ApiResponse {
    CoreError::Internal(
        "file attachments are not configured; register a backend with \
         adminx_storage::init(..)"
            .into(),
    )
    .into()
}

/// Store an uploaded file, returning a ready `ApiResponse`. The adapter calls
/// this after parsing multipart; auth is the caller's responsibility (the
/// resource checks `Update` before invoking).
pub async fn store(
    owner_type: &str,
    owner_id: &str,
    field: &str,
    file: UploadedFile,
) -> Result<Attachment, ApiResponse> {
    let backend = attachments().ok_or_else(not_configured)?;
    backend
        .put(owner_type, owner_id, field, file)
        .await
        .map_err(|e| CoreError::from(e).into())
}

/// List a record's attachments for display. Returns empty (never errors to the
/// caller) so the detail page renders even when the backend hiccups.
pub async fn list(owner_type: &str, owner_id: &str) -> Vec<Attachment> {
    let Some(backend) = attachments() else {
        return Vec::new();
    };
    match backend.list(owner_type, owner_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("adminx: failed to list attachments: {e}");
            Vec::new()
        }
    }
}

/// Purge every attachment for a record. Best-effort: a failure is logged, not
/// propagated, so a storage hiccup can't block a delete the user asked for.
pub async fn purge(owner_type: &str, owner_id: &str) {
    if let Some(backend) = attachments() {
        if let Err(e) = backend.delete_all(owner_type, owner_id).await {
            tracing::error!("adminx: failed to purge attachments for {owner_type}/{owner_id}: {e}");
        }
    }
}
