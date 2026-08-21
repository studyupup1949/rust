use std::marker::PhantomData;
use super::{Runtime, runtime::SignalId};

/// Signal
/// 
/// Reactive signal.
#[derive(Clone, Copy)]
pub struct Signal<T> {
    pub ctx: &'static Runtime,
    pub id: SignalId,
    pub ty: PhantomData<T>,
}

impl<T> Signal<T>
where
    T: 'static + Clone + Copy,
{
    /// Get
    /// 
    /// Get signal value.
    pub fn get(&self) -> T {
        let signals = { self.ctx.signals.borrow() };
        let value = &signals[self.id.0];
        let value = value.downcast_ref::<T>().unwrap();

        self.add_subscriber();

        value.clone()
    }

    /// Set
    /// 
    /// Set signal value.
    pub fn set(&self, value: T) {
        {
            let mut signals = self.ctx.signals.borrow_mut();
            signals[self.id.0] = Box::new(value);
        }

        self.notify_subscribers();
    }

    /// Add Subscriber
    /// 
    /// 
    fn add_subscriber(&self) {
        let running_effect = { self.ctx.running_effect.borrow() };

        if let Some(effect_id) = running_effect.to_owned() {
            self.ctx
                .subscribers
                .borrow_mut()
                .entry(self.id)
                .or_default()
                .insert(effect_id);
        }
    }

    /// Notify Subscribers
    /// 
    /// 
    fn notify_subscribers(&self) {
        let subscribers = {
            let subscribers = self.ctx.subscribers.borrow();
            subscribers.get(&self.id).unwrap().clone()
        };

        for effect_id in subscribers {
            self.ctx.run_effect(effect_id);
        }
    }
}
