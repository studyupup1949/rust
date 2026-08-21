use tokio::{sync::mpsc, task::JoinHandle};

#[async_trait::async_trait]
pub trait Actor {
    type Message;

    async fn run(self, state: State<Self>);
}

pub struct State<A>
where
    A: Actor + ?Sized,
{
    pub message_receiver: mpsc::Receiver<A::Message>,
    pub control_receiver: mpsc::Receiver<Control>,
}

pub struct Handle<A: Actor> {
    message_sender: mpsc::Sender<A::Message>,
}

impl<A: Actor> Clone for Handle<A> {
    fn clone(&self) -> Self {
        Self {
            message_sender: self.message_sender.clone(),
        }
    }
}

impl<A: Actor> Handle<A> {
    pub async fn send(&self, message: A::Message) {
        if let Err(e) = self.message_sender.send(message).await {
            tracing::error!("Failed to send message to handle: {:?}", e);
        }
    }
}

pub enum Control {
    Shutdown,
}

pub struct Runner {
    control_senders: Vec<mpsc::Sender<Control>>,
    join_handles: Vec<JoinHandle<()>>,
}

impl Runner {
    const BACKPRESSURE: usize = 10;

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            control_senders: Vec::new(),
            join_handles: Vec::new(),
        }
    }

    pub fn run<A>(&mut self, actor: A) -> Handle<A>
    where
        A: Actor + Send + 'static,
        A::Message: Send + 'static,
    {
        let (message_sender, message_receiver) = mpsc::channel(Self::BACKPRESSURE);
        let (control_sender, control_receiver) = mpsc::channel(1);
        let join_handle = tokio::spawn(async move {
            actor
                .run(State {
                    message_receiver,
                    control_receiver,
                })
                .await;
        });
        self.control_senders.push(control_sender);
        self.join_handles.push(join_handle);

        Handle::<A> { message_sender }
    }

    pub async fn shutdown(self) {
        for control_sender in self.control_senders {
            if let Err(e) = control_sender.send(Control::Shutdown).await {
                tracing::error!("Failed to send shutdown control message: {:?}", e);
            }
        }
        for join_handle in self.join_handles {
            if let Err(e) = join_handle.await {
                tracing::error!("Failed to join actor: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::oneshot;

    use super::*;

    struct Counter {
        count: usize,
    }

    impl Counter {
        fn new() -> Self {
            Self { count: 0 }
        }
    }

    enum CounterMessage {
        IncrementAndGet { response: oneshot::Sender<usize> },
    }

    #[async_trait::async_trait]
    impl Actor for Counter {
        type Message = CounterMessage;

        async fn run(mut self, mut state: State<Self>) {
            loop {
                tokio::select! {
                    Some(ctrl) = state.control_receiver.recv() => {
                        match ctrl {
                            Control::Shutdown => break,
                        }
                    }
                    Some(message) = state.message_receiver.recv() => {
                        match message {
                            CounterMessage::IncrementAndGet { response } => {
                                self.count += 1;
                                let _ = response.send(self.count);
                            }
                        }
                    }
                    else => break,
                }
            }
        }
    }

    struct CounterForwarder {
        counter_handle: Handle<Counter>,
    }

    impl CounterForwarder {
        fn new(counter_handle: Handle<Counter>) -> Self {
            Self { counter_handle }
        }
    }

    #[async_trait::async_trait]
    impl Actor for CounterForwarder {
        type Message = CounterMessage;

        async fn run(mut self, mut state: State<Self>) {
            loop {
                tokio::select! {
                    Some(ctrl) = state.control_receiver.recv() => {
                        match ctrl {
                            Control::Shutdown => break,
                        }
                    }
                    Some(message) = state.message_receiver.recv() => {
                        self.counter_handle.send(message).await;
                    }
                    else => break,
                }
            }
        }
    }

    #[tokio::test]
    async fn actors() {
        let mut runner = Runner::new();
        let counter_handle = runner.run(Counter::new());
        let counter_forwarder_handle = runner.run(CounterForwarder::new(counter_handle.clone()));

        let (sender, receiver) = oneshot::channel();
        counter_forwarder_handle
            .send(CounterMessage::IncrementAndGet { response: sender })
            .await;
        let count = receiver.await.unwrap();
        assert_eq!(count, 1);

        runner.shutdown().await;
    }
}
