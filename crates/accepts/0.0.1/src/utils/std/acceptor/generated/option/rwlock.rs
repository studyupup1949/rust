use crate::{
    __internal::alloc::sync::Arc,
    utils::std::acceptor::shared::codegen::{guard_access::RwLockAccess, handler::OptionHandler},
};

use std::sync::{PoisonError, RwLock};

use accepts_macros::auto_impl_dyn_internal;

accepts_macros::generate_linear_acceptor_internal! {

    #[must_use = "RwLockOptionAcceptor must be used to forward option values guarded by an RwLock"]
    #[derive(Debug)]
    pub struct RwLockOptionAcceptor<Value: Clone>{
        accepts_impls: [Sync],
        handler: Mut(OptionHandler<RwLock<__Inner> => Option<__Item>>, RwLockAccess<PoisonError<()>>{ forward_source = Ref }){ source_ident = rw_lock },
    }

    #[must_use = "RwLockOptionFinalAcceptor must be used to flush the final option value from an RwLock"]
    #[derive(Debug)]
    pub struct RwLockOptionFinalAcceptor<Value>{
        accepts_impls: [Sync],
        tail_accepts: Final,
        handler: Mut(OptionHandler<RwLock<__Inner> => Option<__Item>>, RwLockAccess<PoisonError<()>>{ forward_source = Ref }){ source_ident = rw_lock },
    }

    #[must_use = "ArcRwLockOptionAcceptor must be used to forward option values guarded by an Arc<RwLock<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcRwLockOptionAcceptor<Value: Clone>{
        accepts_impls: [Sync],
        handler: Mut(OptionHandler<Arc<__Inner> => RwLock<__Inner> => Option<__Item>>, RwLockAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = rw_lock },
    }

    #[must_use = "ArcRwLockOptionFinalAcceptor must be used to flush the final option value from an Arc<RwLock<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcRwLockOptionFinalAcceptor<Value>{
        accepts_impls: [Sync],
        tail_accepts: Final,
        handler: Mut(OptionHandler<Arc<__Inner> => RwLock<__Inner> => Option<__Item>>, RwLockAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = rw_lock },
    }

    #[must_use = "ArcRwLockOptionAsyncAcceptor must be used to forward async option values guarded by an Arc<RwLock<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcRwLockOptionAsyncAcceptor<Value: Clone>{
        accepts_impls: [Async(#[auto_impl_dyn_internal(cfg(feature = "alloc"))])],
        handler: Mut(OptionHandler<Arc<__Inner> => RwLock<__Inner> => Option<__Item>>, RwLockAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = rw_lock },
    }

    #[must_use = "ArcRwLockOptionFinalAsyncAcceptor must be used to flush the final async option value from an Arc<RwLock<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcRwLockOptionFinalAsyncAcceptor<Value>{
        accepts_impls: [Async(#[auto_impl_dyn_internal])],
        tail_accepts: Final,
        handler: Mut(OptionHandler<Arc<__Inner> => RwLock<__Inner> => Option<__Item>>, RwLockAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = rw_lock },
    }

}
