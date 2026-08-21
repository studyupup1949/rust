use core::future::Future;
use tokio::sync::mpsc::{Sender, error::SendError};

use crate::codegen::acceptor::handler::{
    FinalAsyncValueErrorHandlerRef, FinalValueErrorHandlerRef, LinearAsyncValueErrorHandlerRef,
    LinearValueErrorHandlerRef,
};

pub struct MpscSenderHandler;

impl<Value> LinearValueErrorHandlerRef<Sender<Value>, Value, SendError<Value>> for MpscSenderHandler
where
    Value: Clone,
{
    fn apply(
        receiver: &Sender<Value>,
        value: Value,
        has_next: bool,
    ) -> Result<Option<Value>, SendError<Value>> {
        let return_value = has_next.then(|| value.clone());
        receiver.blocking_send(value)?;
        Ok(return_value)
    }
}

impl<Value> LinearAsyncValueErrorHandlerRef<Sender<Value>, Value, SendError<Value>>
    for MpscSenderHandler
where
    Value: Clone,
{
    fn apply<'a>(
        receiver: &'a Sender<Value>,
        value: Value,
        has_next: bool,
    ) -> impl Future<Output = Result<Option<Value>, SendError<Value>>> + 'a
    where
        Option<Value>: 'a,
        SendError<Value>: 'a,
    {
        async move {
            let return_value = has_next.then(|| value.clone());
            receiver.send(value).await?;
            Ok(return_value)
        }
    }
}

impl<Value> FinalValueErrorHandlerRef<Sender<Value>, Value, SendError<Value>>
    for MpscSenderHandler
{
    fn apply(receiver: &Sender<Value>, value: Value) -> Result<(), SendError<Value>> {
        receiver.blocking_send(value)?;
        Ok(())
    }
}

impl<Value> FinalAsyncValueErrorHandlerRef<Sender<Value>, Value, SendError<Value>>
    for MpscSenderHandler
{
    fn apply<'a>(
        receiver: &'a Sender<Value>,
        value: Value,
    ) -> impl Future<Output = Result<(), SendError<Value>>> + 'a
    where
        SendError<Value>: 'a,
    {
        async {
            receiver.send(value).await?;
            Ok(())
        }
    }
}
