//! This module implements `BackendProvider` in local memory.

use std::collections::HashMap;

use chrono::Utc;
use log::debug;

use crate::{backend::BackendError, limit::Limit, RequestId};

use super::{BackendProvider, Bucket};

/// Stores rate limits buckets in-memory via HashMap.
#[derive(Clone, Debug, Default)]
pub struct MemoryBackendProvider {
    usages: HashMap<RequestId, Bucket>,
}

impl BackendProvider for MemoryBackendProvider {
    async fn validate_request(
        &mut self,
        id: &RequestId,
        limit: Limit,
    ) -> Result<(), super::BackendError> {
        let bucket = self.usages.entry(id.clone()).or_insert(Bucket {
            remaining: limit.amount,
            created_at: Utc::now(),
        });

        if (Utc::now() - bucket.created_at).num_seconds() >= limit.ttl {
            debug!("Bucket {} has expired. Creating a new one.", id);
            self.usages.insert(
                id.clone(),
                Bucket {
                    remaining: limit.amount - 1,
                    created_at: Utc::now(),
                },
            );
            return Ok(());
        }

        if bucket.remaining <= 0 {
            return Err(BackendError::RateLimited);
        }

        bucket.remaining -= 1;
        Ok(())
    }
}
