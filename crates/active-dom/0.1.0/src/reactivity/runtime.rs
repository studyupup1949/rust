use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalId(pub usize);

/// Runtime
/// 
/// 
#[derive(Default)]
pub struct Runtime {
    pub signals: Rc<RefCell<Vec<Box<dyn Any>>>>,

    pub effects: Rc<RefCell<Vec<Box<dyn Fn()>>>>,
    pub running_effect: Rc<RefCell<Option<EffectId>>>,
    pub subscribers: Rc<RefCell<HashMap<SignalId, HashSet<EffectId>>>>,
}

impl Runtime {
    /// Run Effect
    /// 
    /// 
    pub fn run_effect(&'static self, effect_id: EffectId) {
        let previous_effect = {
            let mut running_effect = self.running_effect.borrow_mut();
            let previous_effect = running_effect.take();

            // set running effect to current effect
            *running_effect = Some(effect_id);

            previous_effect
        };

        {
            let effects = { self.effects.borrow() };
            effects[effect_id.0]();
        }

        {
            *self.running_effect.borrow_mut() = previous_effect;
        }
    }
}
