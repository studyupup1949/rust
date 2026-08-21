# adminx-cloud

[![crates.io](https://img.shields.io/crates/v/adminx-cloud.svg)](https://crates.io/crates/adminx-cloud)
[![docs.rs](https://img.shields.io/docsrs/adminx-cloud)](https://docs.rs/adminx-cloud)
[![license: MIT](https://img.shields.io/crates/l/adminx-cloud.svg)](https://github.com/srotas-space/adminx/blob/main/LICENSE)

> **AWS S3 · Google Cloud Storage · Azure Blob** backends for
> [adminx](https://crates.io/crates/adminx) file attachments.

Store attachment bytes in the cloud instead of on local disk. Every provider is
reached through one [`object_store`](https://crates.io/crates/object_store)
trait, so this crate is a single thin `BlobStore` wrapper plus a config builder
per provider — and you enable only the clouds you use.

Everything above the storage layer is unchanged: the upload widget, the
attach / serve / detach routes, replace-on-reupload and purge-on-delete all work
exactly the same, because only the backend swaps.

---

## Install

Pick your providers with feature flags — the heavy cloud HTTP stacks compile
only for the ones you turn on:

```toml
# just S3 (also covers S3-compatible: MinIO, Cloudflare R2, ...)
adminx-cloud = { version = "3", features = ["aws"] }

# everything
adminx-cloud = { version = "3", features = ["all"] }
```

| Feature | Backend |
| --- | --- |
| `aws` | AWS S3 / S3-compatible |
| `gcp` | Google Cloud Storage |
| `azure` | Azure Blob Storage |
| `all` | all three |

## Use

One line, in place of `init_local`:

```rust,ignore
// AWS S3 — credentials from AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY / AWS_REGION
adminx::attachments::init(adminx_cloud::s3_from_env("my-bucket")?);

// Google Cloud Storage — GOOGLE_SERVICE_ACCOUNT / Application Default Credentials
adminx::attachments::init(adminx_cloud::gcs_from_env("my-bucket")?);

// Azure Blob — AZURE_STORAGE_ACCOUNT_NAME / AZURE_STORAGE_ACCOUNT_KEY
adminx::attachments::init(adminx_cloud::azure_from_env("my-container")?);
```

The attachment **metadata** still lives in your database (via adminx's
`Storage`); only the bytes go to the cloud.

### Sharing a bucket

Prefix every key so several apps or environments can share one bucket:

```rust,ignore
let blobs = adminx_cloud::s3_from_env("shared-bucket")?
    .with_prefix("prod/attachments");
adminx::attachments::init(blobs);
```

### Custom endpoints and static credentials

The `*_from_env` builders cover the common case. For anything more — a MinIO or
R2 endpoint, static keys, custom retry/timeout — build the `object_store`
yourself and wrap it:

```rust,ignore
use object_store::aws::AmazonS3Builder;

let s3 = AmazonS3Builder::new()
    .with_bucket_name("bucket")
    .with_endpoint("https://minio.internal:9000")
    .with_access_key_id("key")
    .with_secret_access_key("secret")
    .with_allow_http(true)
    .build()?;

adminx::attachments::init(adminx_cloud::from_object_store(std::sync::Arc::new(s3)));
```

## How it fits

```
adminx-core            the Attachments seam + resource routes
adminx-attachments     BlobStore trait + AttachmentStore + LocalFsStore
adminx-cloud    ←────  implements BlobStore over object_store (this crate)
```

`adminx-cloud` produces a `Box<dyn BlobStore>` that
[`adminx_attachments::init`](https://docs.rs/adminx-attachments) accepts. That's
the whole integration surface.

## A note on testing

The provider builders can't be exercised without live credentials, so they're
verified to compile against the real object_store API. The wrapper's own
logic — put / get / delete, key prefixing, the not-found mapping the serve path
relies on — is provider-independent (S3, GCS and Azure are just different
`object_store` implementations) and is tested over object_store's in-memory
store, which needs no credentials.

## License

MIT
