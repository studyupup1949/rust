use core::fmt;

use accepts::{Accepts, AsyncAccepts};

/// Prints values using `Debug` formatting.
#[must_use = "DebugPrinter must be used to emit output"]
#[derive(Debug, Clone)]
pub struct DebugPrinter;

impl<Value> Accepts<Value> for DebugPrinter
where
    Value: fmt::Debug,
{
    fn accept(&self, value: Value) {
        println!("{value:?}");
    }
}

impl<Value> AsyncAccepts<Value> for DebugPrinter
where
    Value: fmt::Debug + Send,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl core::future::Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async move {
            println!("{value:?}");
        }
    }
}
