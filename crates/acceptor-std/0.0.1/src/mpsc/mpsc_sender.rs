use std::sync::mpsc::{SendError, Sender};

use accepts::{Accepts, AsyncAccepts};

/// Sends values through an `mpsc::Sender` and forwards the `Result` downstream.
#[derive(Debug, Clone)]
pub struct MpscSender<Value, NextAccepts> {
    sender: Sender<Value>,
    next_acceptor: NextAccepts,
}

impl<Value, NextAccepts> MpscSender<Value, NextAccepts> {
    pub fn new(sender: Sender<Value>, next_acceptor: NextAccepts) -> Self {
        Self {
            sender,
            next_acceptor,
        }
    }
}

impl<Value, NextAccepts> Accepts<Value> for MpscSender<Value, NextAccepts>
where
    NextAccepts: Accepts<Result<(), SendError<Value>>>,
{
    fn accept(&self, value: Value) {
        let result = self.sender.send(value);
        self.next_acceptor.accept(result);
    }
}

impl<Value, NextAccepts> AsyncAccepts<Value> for MpscSender<Value, NextAccepts>
where
    NextAccepts: AsyncAccepts<Result<(), SendError<Value>>>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl core::future::Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async move {
            let result = self.sender.send(value);
            self.next_acceptor.accept_async(result).await;
        }
    }
}
