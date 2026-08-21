use crate::{
    __internal::alloc::sync::Arc,
    utils::std::acceptor::shared::codegen::{guard_access::MutexAccess, handler::OptionHandler},
};

use std::sync::{Mutex, PoisonError};

use accepts_macros::auto_impl_dyn_internal;

accepts_macros::generate_linear_acceptor_internal! {

    #[must_use = "MutexOptionAcceptor must be used to forward option values guarded by a Mutex"]
    #[derive(Debug)]
    pub struct MutexOptionAcceptor<Value: Clone>{
        accepts_impls: [Sync],
        handler: Mut(OptionHandler<Mutex<__Inner> => Option<__Item>>, MutexAccess<PoisonError<()>>{ forward_source = Ref }){ source_ident = vec },
    }

    #[must_use = "MutexOptionFinalAcceptor must be used to flush the final option value from a Mutex"]
    #[derive(Debug)]
    pub struct MutexOptionFinalAcceptor<Value>{
        accepts_impls: [Sync],
        tail_accepts: Final,
        handler: Mut(OptionHandler<Mutex<__Inner> => Option<__Item>>, MutexAccess<PoisonError<()>>{ forward_source = Ref }){ source_ident = vec },
    }

    #[must_use = "ArcMutexOptionAcceptor must be used to forward option values guarded by an Arc<Mutex<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcMutexOptionAcceptor<Value: Clone>{
        accepts_impls: [Sync],
        handler: Mut(OptionHandler<Arc<__Inner> => Mutex<__Inner> => Option<__Item>>, MutexAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = vec },
    }

    #[must_use = "ArcMutexOptionFinalAcceptor must be used to flush the final option value from an Arc<Mutex<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcMutexOptionFinalAcceptor<Value>{
        accepts_impls: [Sync],
        tail_accepts: Final,
        handler: Mut(OptionHandler<Arc<__Inner> => Mutex<__Inner> => Option<__Item>>, MutexAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = vec },
    }

    #[must_use = "ArcMutexOptionAsyncAcceptor must be used to forward async option values guarded by an Arc<Mutex<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcMutexOptionAsyncAcceptor<Value: Clone>{
        accepts_impls: [Async(#[auto_impl_dyn_internal])],
        handler: Mut(OptionHandler<Arc<__Inner> => Mutex<__Inner> => Option<__Item>>, MutexAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = vec },
    }

    #[must_use = "ArcMutexOptionFinalAsyncAcceptor must be used to flush the final async option value from an Arc<Mutex<_>>"]
    #[derive(Debug, Clone)]
    pub struct ArcMutexOptionFinalAsyncAcceptor<Value>{
        accepts_impls: [Async(#[auto_impl_dyn_internal])],
        tail_accepts: Final,
        handler: Mut(OptionHandler<Arc<__Inner> => Mutex<__Inner> => Option<__Item>>, MutexAccess<PoisonError<()>>{ forward_source = Clone }){ source_ident = vec },
    }

}
