use super::{entry::Entry, registry::Registry};

pub trait EventObserver<Args>: Send + Sync {
    fn notify(&self, args: &Args);
}

#[derive(Clone)]
pub struct Event<Args>
where
    Args: Send + Sync + 'static,
{
    observers: Registry<Box<dyn EventObserver<Args>>>,
}

impl<O, Args> EventObserver<Args> for O
where
    O: Fn(&Args) + Send + Sync,
{
    fn notify(&self, args: &Args) {
        self(args)
    }
}

impl<Args> Default for Event<Args>
where
    Args: Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Args> Event<Args>
where
    Args: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            observers: Registry::new(),
        }
    }

    pub fn register_observer<O>(&self, observer: O) -> Entry
    where
        O: EventObserver<Args> + 'static,
    {
        self.observers.register(Box::new(observer)).as_generic()
    }

    pub fn dispatch(&self, mut args: Args) {
        for (_, observer) in self.observers.read().iter() {
            (**observer).notify(&mut args);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_send_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        fn is_clone<T: Clone>() {}

        is_send_sync::<Event<i32>>();
        is_clone::<Event<i32>>();
    }
}
