use core::future::{Future, ready};
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::codegen::acceptor::guard_access::{AcquireError, AsyncMutGuardAccess, MutGuardAccess};

pub struct MutexAccess;

impl<T> MutGuardAccess<Mutex<T>, T> for MutexAccess {
    type Guard<'a>
        = MutexGuard<'a, T>
    where
        Self: 'a,
        T: 'a;

    type Error = PoisonError<()>;

    fn acquire<'a>(
        source: &'a Mutex<T>,
    ) -> Result<Self::Guard<'a>, AcquireError<Self::Error, Self::Guard<'a>>> {
        source.lock().map_err(|error| {
            let guard = error.into_inner();
            AcquireError::new(PoisonError::new(()), Some(guard))
        })
    }
}

impl<T> AsyncMutGuardAccess<Mutex<T>, T> for MutexAccess {
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
        ready(source.lock().map_err(|error| {
            let guard = error.into_inner();
            AcquireError::new(PoisonError::new(()), Some(guard))
        }))
    }
}
