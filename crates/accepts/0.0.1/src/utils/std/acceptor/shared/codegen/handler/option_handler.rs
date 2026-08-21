use core::future::{Future, ready};

use crate::codegen::acceptor::handler::{
    FinalAsyncValueHandlerMut, FinalValueHandlerMut, LinearAsyncValueHandlerMut,
    LinearValueHandlerMut,
};

pub struct OptionHandler;

impl<Value> LinearValueHandlerMut<Option<Value>, Value> for OptionHandler
where
    Value: Clone,
{
    fn apply(receiver: &mut Option<Value>, value: Value, has_next: bool) -> Option<Value> {
        let return_value = has_next.then(|| value.clone());
        receiver.replace(value);
        return_value
    }
}

impl<Value> LinearAsyncValueHandlerMut<Option<Value>, Value> for OptionHandler
where
    Value: Clone,
{
    fn apply<'a>(
        receiver: &'a mut Option<Value>,
        value: Value,
        has_next: bool,
    ) -> impl Future<Output = Option<Value>> + 'a
    where
        Value: 'a,
    {
        let return_value = has_next.then(|| value.clone());
        receiver.replace(value);
        ready(return_value)
    }
}

impl<Value> FinalValueHandlerMut<Option<Value>, Value> for OptionHandler {
    fn apply(receiver: &mut Option<Value>, value: Value) {
        receiver.replace(value);
    }
}

impl<Value> FinalAsyncValueHandlerMut<Option<Value>, Value> for OptionHandler {
    fn apply<'a>(receiver: &'a mut Option<Value>, value: Value) -> impl Future<Output = ()> + 'a {
        receiver.replace(value);
        ready(())
    }
}
