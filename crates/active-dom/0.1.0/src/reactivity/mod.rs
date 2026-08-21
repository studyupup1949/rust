use std::marker::PhantomData;
pub use runtime::Runtime;
use runtime::{EffectId, SignalId};
use signal::Signal;

mod runtime;
mod signal;

/// Create Signal
/// 
/// 
pub fn create_signal<T>(ctx: &'static Runtime, value: T) -> Signal<T>
where
    T: 'static + Clone + Copy,
{
    let mut signals = ctx.signals.borrow_mut();
    signals.push(Box::new(value));

    let signal_id = SignalId(signals.len() - 1);

    Signal {
        ctx,
        id: signal_id,
        ty: PhantomData::<T>,
    }
}

/// Create Effect
/// 
/// 
pub fn create_effect(ctx: &'static Runtime, func: impl Fn() + 'static) {
    let effect_id = {
        let mut effects = ctx.effects.borrow_mut();
        effects.push(Box::new(func));

        EffectId(effects.len() - 1)
    };

    ctx.run_effect(effect_id);
}


