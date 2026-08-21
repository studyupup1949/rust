use core::fmt;

use accepts::{Accepts, AsyncAccepts};

/// Prints values using `Display` formatting.
#[must_use = "DisplayPrinter must be used to emit output"]
#[derive(Debug, Clone)]
pub struct DisplayPrinter;

impl<Value> Accepts<Value> for DisplayPrinter
where
    Value: fmt::Display,
{
    fn accept(&self, value: Value) {
        println!("{value}");
    }
}

impl<Value> AsyncAccepts<Value> for DisplayPrinter
where
    Value: fmt::Display + Send,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl core::future::Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async move { println!("{value}") }
    }
}
