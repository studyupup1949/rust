use core::future::{Future, ready};
use std::sync::{PoisonError, RwLock, RwLockWriteGuard};

use crate::codegen::acceptor::guard_access::{AcquireError, AsyncMutGuardAccess, MutGuardAccess};

pub struct RwLockAccess;

impl<T> MutGuardAccess<RwLock<T>, T> for RwLockAccess {
    type Guard<'a>
        = RwLockWriteGuard<'a, T>
    where
        Self: 'a,
        T: 'a;

    type Error = PoisonError<()>;

    fn acquire<'a>(
        source: &'a RwLock<T>,
    ) -> Result<Self::Guard<'a>, AcquireError<Self::Error, Self::Guard<'a>>> {
        source.write().map_err(|error| {
            let guard = error.into_inner();
            AcquireError::new(PoisonError::new(()), Some(guard))
        })
    }
}

impl<T> AsyncMutGuardAccess<RwLock<T>, T> for RwLockAccess {
    type Guard<'a>
        = RwLockWriteGuard<'a, T>
    where
        Self: 'a,
        T: 'a;

    type Error = PoisonError<()>;

    fn acquire<'a>(
        source: &'a RwLock<T>,
    ) -> impl Future<Output = Result<Self::Guard<'a>, AcquireError<Self::Error, Self::Guard<'a>>>> + 'a
    where
        Self: 'a,
    {
        ready(source.write().map_err(|error| {
            let guard = error.into_inner();
            AcquireError::new(PoisonError::new(()), Some(guard))
        }))
    }
}
