use actix::prelude::*;

use std::{collections::HashSet, hash::Hash, mem::replace, time::Duration, time::Instant};

type CallbackFn<T> = Box<dyn Fn(HashSet<T>) + Send>;

pub struct Aggregator<T: Eq + Hash> {
    buf: HashSet<T>,
    cb: CallbackFn<T>,
    debounce: Duration,
    max_delay: Duration,
    handle: Option<(SpawnHandle, Instant)>,
}

impl<T> Aggregator<T>
where
    T: Eq + Hash + Unpin + 'static,
{
    pub fn new(
        debounce: Duration,
        max_delay: Duration,
        callback_fn: CallbackFn<T>,
    ) -> Aggregator<T> {
        Self {
            buf: HashSet::new(),
            cb: callback_fn,
            debounce,
            max_delay,
            handle: None,
        }
    }

    pub fn extend(&mut self, payload: Vec<T>, ctx: &mut <Self as Actor>::Context) {
        self.buf.extend(payload);
        self.flush_later(ctx);
    }

    pub fn flush_later(&mut self, ctx: &mut <Self as Actor>::Context) {
        if let Some((handle, start)) = replace(&mut self.handle, None) {
            ctx.cancel_future(handle);
            if start.elapsed() >= self.max_delay {
                ctx.notify(AggregatorCmd::Flush);
            } else {
                self.handle = Some((ctx.notify_later(AggregatorCmd::Flush, self.debounce), start))
            }
        } else {
            self.handle = Some((
                ctx.notify_later(AggregatorCmd::Flush, self.debounce),
                Instant::now(),
            ));
        }
    }

    pub fn flush(&mut self) {
        self.handle.take();
        if !self.buf.is_empty() {
            (self.cb)(replace(&mut self.buf, HashSet::new()))
        }
    }
}

impl<T> Actor for Aggregator<T>
where
    T: Eq + Hash + Unpin + 'static,
{
    type Context = Context<Self>;
}

impl<T> Supervised for Aggregator<T>
where
    T: Eq + Hash + Unpin + 'static,
{
    fn restarting(&mut self, _: &mut Self::Context) {}
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum AggregatorCmd<T: Eq + Hash> {
    NewData(Vec<T>),
    Flush,
}

impl<T> Handler<AggregatorCmd<T>> for Aggregator<T>
where
    T: Eq + Hash + Unpin + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: AggregatorCmd<T>, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            AggregatorCmd::NewData(payload) => self.extend(payload, ctx),
            AggregatorCmd::Flush => self.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
