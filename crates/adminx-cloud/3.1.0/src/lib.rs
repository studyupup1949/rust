// adminx-cloud/src/lib.rs
//
// Cloud storage backends for adminx file attachments. Every provider — AWS S3,
// Google Cloud Storage, Azure Blob — is reached through the same
// [`object_store`] trait, so this crate is one small [`BlobStore`] wrapper plus
// a config builder per provider. Enable only the providers you use:
//
// ```toml
// adminx-cloud = { version = "3", features = ["aws"] }        # just S3
// adminx-cloud = { version = "3", features = ["all"] }        # S3 + GCS + Azure
// ```
//
// ## Use
//
// ```ignore
// // credentials from the standard env vars (AWS_ACCESS_KEY_ID, ...)
// let blobs = adminx_cloud::s3_from_env("my-bucket")?;
// adminx_attachments::init(blobs);   // hand it to the attachment layer
// ```
//
// The attachment metadata still lives in your database (via adminx's `Storage`);
// only the bytes move to the cloud. Everything above the `BlobStore` — the
// upload widget, attach/serve/detach routes, replace-on-reupload, purge-on-
// delete — is unchanged, because the backend is the only thing that swaps.

use adminx_attachments::BlobStore;
use adminx_core::storage::StorageError;
use async_trait::async_trait;
// The convenience `put`/`get`/`delete` live on `ObjectStoreExt` in 0.14; the
// base `ObjectStore` trait only exposes the `_opts` forms.
use object_store::{path::Path as ObjPath, ObjectStore, ObjectStoreExt, PutPayload};
use std::sync::Arc;

/// A [`BlobStore`] backed by any [`object_store::ObjectStore`] — S3, GCS, Azure,
/// or the in-memory / local stores object_store ships. Construct it with one of
/// the provider builders below, or [`from_object_store`] for full control.
pub struct CloudBlobStore {
    inner: Arc<dyn ObjectStore>,
    /// Optional key prefix, so several apps (or environments) can share one
    /// bucket without colliding. Joined ahead of every attachment key.
    prefix: Option<String>,
}

impl CloudBlobStore {
    /// Prefix every key with `prefix` (e.g. `"prod/attachments"`), so this
    /// store's blobs live under their own path in a shared bucket.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        let p = prefix.into();
        self.prefix = if p.is_empty() { None } else { Some(p) };
        self
    }

    /// Resolve an attachment key to an object-store path, applying the prefix and
    /// validating the result.
    fn path(&self, key: &str) -> Result<ObjPath, StorageError> {
        let full = match &self.prefix {
            Some(p) => format!("{}/{}", p.trim_end_matches('/'), key),
            None => key.to_string(),
        };
        ObjPath::parse(&full)
            .map_err(|e| StorageError::Backend(format!("invalid object key {full:?}: {e}")))
    }
}

/// Build a store from an already-constructed [`object_store::ObjectStore`]. The
/// escape hatch when the `*_from_env` builders aren't enough — pass anything
/// object_store can build (custom endpoints, static credentials, retries, ...).
///
/// ```ignore
/// use object_store::aws::AmazonS3Builder;
/// let s3 = AmazonS3Builder::new()
///     .with_bucket_name("bucket")
///     .with_endpoint("https://minio.internal:9000")
///     .with_access_key_id("key")
///     .with_secret_access_key("secret")
///     .with_allow_http(true)
///     .build()?;
/// let blobs = adminx_cloud::from_object_store(std::sync::Arc::new(s3));
/// ```
pub fn from_object_store(store: Arc<dyn ObjectStore>) -> Box<CloudBlobStore> {
    Box::new(CloudBlobStore {
        inner: store,
        prefix: None,
    })
}

#[async_trait]
impl BlobStore for CloudBlobStore {
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StorageError> {
        let path = self.path(key)?;
        self.inner
            .put(&path, PutPayload::from(bytes.to_vec()))
            .await
            .map(|_| ())
            .map_err(map_err)
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let path = self.path(key)?;
        let result = self.inner.get(&path).await.map_err(map_err)?;
        let bytes = result.bytes().await.map_err(map_err)?;
        Ok(bytes.to_vec())
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let path = self.path(key)?;
        match self.inner.delete(&path).await {
            Ok(()) => Ok(()),
            // Idempotent: deleting an absent key is success.
            Err(object_store::Error::NotFound { .. }) => Ok(()),
            Err(e) => Err(map_err(e)),
        }
    }
}

/// Map object_store errors onto adminx's storage error, preserving the
/// not-found distinction the serve path relies on.
fn map_err(e: object_store::Error) -> StorageError {
    match e {
        object_store::Error::NotFound { .. } => StorageError::NotFound,
        other => StorageError::Backend(format!("object store: {other}")),
    }
}

// ===== Provider builders =====
//
// Each reads credentials from that provider's standard environment variables
// (the same ones the provider's own CLI/SDK use), then targets one bucket or
// container. For anything more than that — custom endpoints, static keys — build
// the object_store yourself and use [`from_object_store`].

/// AWS S3 (or S3-compatible: MinIO, Cloudflare R2, ...), credentials from the
/// standard `AWS_*` environment (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`,
/// `AWS_REGION`, `AWS_ENDPOINT`, ...).
#[cfg(feature = "aws")]
pub fn s3_from_env(bucket: impl Into<String>) -> Result<Box<CloudBlobStore>, StorageError> {
    let bucket = bucket.into();
    let store = object_store::aws::AmazonS3Builder::from_env()
        .with_bucket_name(&bucket)
        .build()
        .map_err(|e| StorageError::Backend(format!("S3 init ({bucket}): {e}")))?;
    tracing::info!("adminx-cloud: attachments -> S3 bucket `{bucket}`");
    Ok(from_object_store(Arc::new(store)))
}

/// Google Cloud Storage, credentials from the standard environment
/// (`GOOGLE_SERVICE_ACCOUNT` / `GOOGLE_SERVICE_ACCOUNT_KEY` / Application Default
/// Credentials).
#[cfg(feature = "gcp")]
pub fn gcs_from_env(bucket: impl Into<String>) -> Result<Box<CloudBlobStore>, StorageError> {
    let bucket = bucket.into();
    let store = object_store::gcp::GoogleCloudStorageBuilder::from_env()
        .with_bucket_name(&bucket)
        .build()
        .map_err(|e| StorageError::Backend(format!("GCS init ({bucket}): {e}")))?;
    tracing::info!("adminx-cloud: attachments -> GCS bucket `{bucket}`");
    Ok(from_object_store(Arc::new(store)))
}

/// Azure Blob Storage, credentials from the standard environment
/// (`AZURE_STORAGE_ACCOUNT_NAME`, `AZURE_STORAGE_ACCOUNT_KEY`, ...). `container`
/// is the blob container name.
#[cfg(feature = "azure")]
pub fn azure_from_env(container: impl Into<String>) -> Result<Box<CloudBlobStore>, StorageError> {
    let container = container.into();
    let store = object_store::azure::MicrosoftAzureBuilder::from_env()
        .with_container_name(&container)
        .build()
        .map_err(|e| StorageError::Backend(format!("Azure init ({container}): {e}")))?;
    tracing::info!("adminx-cloud: attachments -> Azure container `{container}`");
    Ok(from_object_store(Arc::new(store)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wrapper's logic is provider-independent: it targets an
    /// `object_store::ObjectStore`, and S3/GCS/Azure are just different
    /// implementations of that trait. So exercising it over object_store's
    /// in-memory store proves the same put/get/delete + prefix + not-found paths
    /// every cloud backend rides on — without credentials.
    #[tokio::test]
    async fn round_trips_and_honours_prefix_over_the_in_memory_store() {
        let mem = Arc::new(object_store::memory::InMemory::new());
        let store = from_object_store(mem.clone()).with_prefix("env1/attachments");

        store.put("posts/1/cover/abc", b"hello").await.unwrap();
        assert_eq!(store.get("posts/1/cover/abc").await.unwrap(), b"hello");

        // The prefix is really applied: the raw object lives under it.
        let raw = mem
            .get(&ObjPath::parse("env1/attachments/posts/1/cover/abc").unwrap())
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();
        assert_eq!(&raw[..], b"hello");

        // Missing key surfaces as NotFound, which the serve path maps to 404.
        assert!(matches!(
            store.get("posts/1/cover/missing").await,
            Err(StorageError::NotFound)
        ));

        // Delete removes it and is idempotent.
        store.delete("posts/1/cover/abc").await.unwrap();
        assert!(matches!(
            store.get("posts/1/cover/abc").await,
            Err(StorageError::NotFound)
        ));
        store.delete("posts/1/cover/abc").await.unwrap(); // second delete: still Ok
    }
}
