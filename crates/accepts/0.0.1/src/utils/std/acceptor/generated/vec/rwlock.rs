use crate::{
    __internal::alloc::sync::Arc,
    utils::std::acceptor::shared::codegen::{guard_access::RwLockAccess, handler::VecHandler},
};

use std::sync::{PoisonError, RwLock};

use accepts_macros::auto_impl_dyn_internal;

accepts_macros::generate_linear_acceptor_internal! {

    #[must_use = "RwLockVecAcceptor must be used to forward vector values guarded by an RwLock"]
    #[derive(Debug)]
    pub struct RwLockVecAcceptor<Value: Clone>{
        accepts_impls: [Sync],
        handler: Mut(VecHandler<RwLock<__Inner> => Vec<__Item>>, RwLockAccess<PoisonError<()>>{ forward_source = Ref }){ source_ident = vec },
    }

    #[must_use = "RwLockVecFinalAcceptor must be used to flush the final vector value from an RwLock"]
    #[derive(Debug)]
    pub struct RwLockVecFinalAcceptor<Value>{
        accepts_impls: [Sync],
        tail_accepts: Final,
        handler: Mut(VecHandler<RwLock<__Inner> => Vec<__Item>>, RwLockAccess<PoisonError<()>>{ forward_source = Ref }){ source_ident = vec },
    }

    #[must_use = "ArcRwLockVecAcceptor must be used to forward vector values guarded by an Arc<RwLock<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcRwLockVecAcceptor<Value: Clone>{
        accepts_impls: [Sync],
        handler: Mut(VecHandler<Arc<__Inner> => RwLock<__Inner> => Vec<__Item>>, RwLockAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = vec },
    }

    #[must_use = "ArcRwLockVecFinalAcceptor must be used to flush the final vector value from an Arc<RwLock<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcRwLockVecFinalAcceptor<Value>{
        accepts_impls: [Sync],
        tail_accepts: Final,
        handler: Mut(VecHandler<Arc<__Inner> => RwLock<__Inner> => Vec<__Item>>, RwLockAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = vec },
    }

    #[must_use = "ArcRwLockVecAsyncAcceptor must be used to forward async vector values guarded by an Arc<RwLock<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcRwLockVecAsyncAcceptor<Value: Clone>{
        accepts_impls: [Async(#[auto_impl_dyn_internal])],
        handler: Mut(VecHandler<Arc<__Inner> => RwLock<__Inner> => Vec<__Item>>, RwLockAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = vec },
    }

    #[must_use = "ArcRwLockVecFinalAsyncAcceptor must be used to flush the final async vector value from an Arc<RwLock<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcRwLockVecFinalAsyncAcceptor<Value>{
        accepts_impls: [Async(#[auto_impl_dyn_internal])],
        tail_accepts: Final,
        handler: Mut(VecHandler<Arc<__Inner> => RwLock<__Inner> => Vec<__Item>>, RwLockAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = vec },
    }

}
