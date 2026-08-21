use core::future::Future;
use std::sync::PoisonError;

use tokio::sync::{Mutex, MutexGuard};

use crate::codegen::acceptor::guard_access::{AcquireError, AsyncMutGuardAccess};

#[allow(dead_code)]
pub struct TokioMutexAccess;

impl<T> AsyncMutGuardAccess<Mutex<T>, T> for TokioMutexAccess {
    type Guard<'a>
        = MutexGuard<'a, T>
    where
        Self: 'a,
        T: 'a;

    type Error = PoisonError<()>;

    fn acquire<'a>(
        source: &'a Mutex<T>,
    ) -> impl Future<Output = Result<Self::Guard<'a>, AcquireError<Self::Error, Self::Guard<'a>>>> + 'a
    where
        Self: 'a,
    {
        async { Ok(source.lock().await) }
    }
}
