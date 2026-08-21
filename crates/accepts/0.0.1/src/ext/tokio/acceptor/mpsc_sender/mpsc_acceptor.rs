use tokio::sync::mpsc::{Sender, error::SendError};

use accepts_macros::auto_impl_dyn_internal;

use super::super::shared::codegen::handler::MpscSenderHandler;

accepts_macros::generate_linear_acceptor_internal! {

    #[must_use = "MpscSenderAcceptor must be used to forward values to the Tokio mpsc sender"]
    #[derive(Debug, Clone)]
    pub struct MpscSenderAcceptor<Value: Clone>{
        accepts_impls: [Sync],
        handler: Ref(MpscSenderHandler<Sender<__Item>, SendError<Value>>){ source_ident = sender }
    }

    #[must_use = "MpscSenderAsyncAcceptor must be used to forward values asynchronously to the Tokio mpsc sender"]
    #[derive(Debug, Clone)]
    pub struct MpscSenderAsyncAcceptor<Value: Clone>{
        accepts_impls: [Async(#[auto_impl_dyn_internal])],
        handler: Ref(MpscSenderHandler<Sender<__Item>, SendError<Value>>){ source_ident = sender }
    }

    #[must_use = "MpscSenderFinalAcceptor must be used to forward the final value to the Tokio mpsc sender"]
    #[derive(Debug, Clone)]
    pub struct MpscSenderFinalAcceptor<Value: Clone>{
        accepts_impls: [Sync],
        tail_accepts: Final,
        handler: Ref(MpscSenderHandler<Sender<__Item>, SendError<Value>>){ source_ident = sender }
    }

    #[must_use = "MpscSenderFinalAsyncAcceptor must be used to forward the final async value to the Tokio mpsc sender"]
    #[derive(Debug, Clone)]
    pub struct MpscSenderFinalAsyncAcceptor<Value: Clone>{
        accepts_impls: [Async(#[auto_impl_dyn_internal])],
        tail_accepts: Final,
        handler: Ref(MpscSenderHandler<Sender<__Item>, SendError<Value>>){ source_ident = sender }
    }

}
