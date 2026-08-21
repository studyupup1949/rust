use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use actix::{Addr, Context, Handler, MailboxError, Message, Supervised};

pub struct Pool<A: actix::Actor> {
    pub(crate) workers: Vec<Addr<A>>,
    pub(crate) current: Arc<AtomicUsize>,
}

impl<A> Pool<A>
where
    A: actix::Actor<Context = Context<A>> + Supervised,
{
    pub fn new<F: 'static + Clone + Fn() -> A>(size: usize, init_fn: F) -> Self {
        Self {
            workers: (0..size)
                .map(|_| {
                    let init_fn = init_fn.clone();
                    actix::Supervisor::start(move |_| init_fn())
                })
                .collect(),
            current: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn next_worker(&self) -> Addr<A> {
        let current = self.current.fetch_add(1, Ordering::SeqCst);
        let index = current % self.workers.len();
        self.workers[index].clone()
    }

    pub fn do_send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: Message + Send + 'static,
        M::Result: Send,
    {
        let actor = self.next_worker();
        actor.do_send(msg);
    }

    pub async fn send<M>(&self, msg: M) -> Result<M::Result, MailboxError>
    where
        A: Handler<M>,
        M: Message + Send + 'static,
        M::Result: Send,
    {
        let actor = self.next_worker();
        actor.send(msg).await
    }
}

#[cfg(test)]
mod tests {
    use actix::{Actor, ActorContext, Message, Supervised};

    use crate::Pool;

    struct TestActor {
        pub name: String,
    }

    impl Actor for TestActor {
        type Context = actix::Context<Self>;
    }
    impl Default for TestActor {
        fn default() -> Self {
            TestActor {
                name: uuid::Uuid::new_v4().to_string(),
            }
        }
    }
    #[derive(Debug, Message)]
    #[rtype(result = "String")]
    struct TestMessage(usize);

    #[derive(Debug, Message)]
    #[rtype(result = "()")]
    struct FailMessage;

    impl actix::Handler<TestMessage> for TestActor {
        type Result = String;
        fn handle(&mut self, msg: TestMessage, _ctx: &mut Self::Context) -> Self::Result {
            format!("Handle test message in test actor: {}-{}", self.name, msg.0)
        }
    }

    impl actix::Handler<FailMessage> for TestActor {
        type Result = ();
        fn handle(&mut self, _msg: FailMessage, ctx: &mut Self::Context) -> Self::Result {
            ctx.stop();
        }
    }

    impl Supervised for TestActor {
        fn restarting(&mut self, _ctx: &mut <Self as Actor>::Context) {
            println!("Restarting actor: {}", self.name);
        }
    }

    #[test]
    fn test_pool() {
        let sys = actix::System::new();

        sys.block_on(async {
            let pool = Pool::new(15, || TestActor::default());

            for i in 0..250 {
                if i % 2 != 0 {
                    let s = pool.send(TestMessage(i)).await;
                    println!("{s:?}");
                    assert!(s.is_ok())
                } else {
                    let _ = pool.send(FailMessage).await;
                }
            }

            actix::System::current().stop();
        });
    }
}
