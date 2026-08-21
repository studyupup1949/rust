use crate::{
    __internal::alloc::sync::Arc,
    utils::std::acceptor::shared::codegen::{guard_access::MutexAccess, handler::VecHandler},
};

use std::sync::{Mutex, PoisonError};

use accepts_macros::auto_impl_dyn_internal;

accepts_macros::generate_linear_acceptor_internal! {

    #[must_use = "MutexVecAcceptor must be used to forward vector values guarded by a Mutex"]
    #[derive(Debug)]
    pub struct MutexVecAcceptor<Value: Clone>{
        accepts_impls: [Sync],
        handler: Mut(VecHandler<Mutex<__Inner> => Vec<__Item>>, MutexAccess<PoisonError<()>>{ forward_source = Ref }){ source_ident = vec },
    }

    #[must_use = "MutexVecFinalAcceptor must be used to flush the final vector value from a Mutex"]
    #[derive(Debug)]
    pub struct MutexVecFinalAcceptor<Value>{
        accepts_impls: [Sync],
        tail_accepts: Final,
        handler: Mut(VecHandler<Mutex<__Inner> => Vec<__Item>>, MutexAccess<PoisonError<()>>{ forward_source = Ref }){ source_ident = vec },
    }

    #[must_use = "ArcMutexVecAcceptor must be used to forward vector values guarded by an Arc<Mutex<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcMutexVecAcceptor<Value: Clone>{
        accepts_impls: [Sync],
        handler: Mut(VecHandler<Arc<__Inner> => Mutex<__Inner> => Vec<__Item>>, MutexAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = vec },
    }

    #[must_use = "ArcMutexVecFinalAcceptor must be used to flush the final vector value from an Arc<Mutex<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcMutexVecFinalAcceptor<Value>{
        accepts_impls: [Sync],
        tail_accepts: Final,
        handler: Mut(VecHandler<Arc<__Inner> => Mutex<__Inner> => Vec<__Item>>, MutexAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = vec },
    }

    #[must_use = "ArcMutexVecAsyncAcceptor must be used to forward async vector values guarded by an Arc<Mutex<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcMutexVecAsyncAcceptor<Value: Clone>{
        accepts_impls: [Async(#[auto_impl_dyn_internal])],
        handler: Mut(VecHandler<Arc<__Inner> => Mutex<__Inner> => Vec<__Item>>, MutexAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = vec },
    }

    #[must_use = "ArcMutexVecFinalAsyncAcceptor must be used to flush the final async vector value from an Arc<Mutex<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcMutexVecFinalAsyncAcceptor<Value>{
        accepts_impls: [Async(#[auto_impl_dyn_internal])],
        tail_accepts: Final,
        handler: Mut(VecHandler<Arc<__Inner> => Mutex<__Inner> => Vec<__Item>>, MutexAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = vec },
    }

}
