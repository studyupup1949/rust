use core::future::{Future, ready};

use crate::{Accepts, AsyncAccepts};

impl<Value> Accepts<Value> for () {
    fn accept(&self, _: Value) {}
}

impl<Value> AsyncAccepts<Value> for () {
    fn accept_async<'a>(&'a self, _: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        ready(())
    }
}
