use core::future::{Future, ready};

use crate::codegen::acceptor::handler::{
    FinalAsyncValueHandlerMut, FinalValueHandlerMut, LinearAsyncValueHandlerMut,
    LinearValueHandlerMut,
};

pub struct VecHandler;

impl<Value> LinearValueHandlerMut<Vec<Value>, Value> for VecHandler
where
    Value: Clone,
{
    fn apply(receiver: &mut Vec<Value>, value: Value, has_next: bool) -> Option<Value> {
        let return_value = has_next.then(|| value.clone());
        receiver.push(value);
        return_value
    }
}

impl<Value> LinearAsyncValueHandlerMut<Vec<Value>, Value> for VecHandler
where
    Value: Clone,
{
    fn apply<'a>(
        receiver: &'a mut Vec<Value>,
        value: Value,
        has_next: bool,
    ) -> impl Future<Output = Option<Value>> + 'a
    where
        Option<Value>: 'a,
    {
        let return_value = has_next.then(|| value.clone());
        receiver.push(value);
        ready(return_value)
    }
}

impl<Value> FinalValueHandlerMut<Vec<Value>, Value> for VecHandler {
    fn apply(receiver: &mut Vec<Value>, value: Value) {
        receiver.push(value);
    }
}

impl<Value> FinalAsyncValueHandlerMut<Vec<Value>, Value> for VecHandler {
    fn apply<'a>(receiver: &'a mut Vec<Value>, value: Value) -> impl Future<Output = ()> + 'a {
        receiver.push(value);
        ready(())
    }
}
