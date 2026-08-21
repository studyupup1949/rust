//! Base-aware ALU dispatcher.
//!
//! Before an operation, the dispatcher inspects the denominator signatures of
//! the operands to decide *which channels are actually needed*. When adding
//! `1/6 + 1/4` the natural base involves only the primes `{2, 3}`; the channels
//! for 5, 7, 11, … are idle. On a GPU, idle channels consume no power — so the
//! dispatcher is the hardware analog of lazy evaluation: work happens only where
//! the number's arithmetic structure demands it.

use crate::primes::lcm;
use crate::rational::RnsRational;
use crate::rns::Channels;

/// A plan describing which channels an operation needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchPlan {
    /// Indices (into `channels`) of the channels that must be active.
    pub active_channels: Vec<usize>,
    /// LCM of the operands' natural bases.
    pub natural_base: u64,
    /// Bitmask of active channels (bit `i` set ⇒ channel `i` active).
    pub channel_mask: u64,
}

/// Routes operations to the minimal set of RNS channels.
pub struct Dispatcher {
    channels: Channels,
}

impl Dispatcher {
    pub fn new(channels: Channels) -> Self {
        Dispatcher { channels }
    }

    /// The channels this dispatcher manages.
    pub fn channels(&self) -> &Channels {
        &self.channels
    }

    fn plan_for(&self, natural_base: u64) -> DispatchPlan {
        let mut active_channels = Vec::new();
        let mut channel_mask = 0u64;
        for c in 0..self.channels.len() {
            let prime = self.channels.modulus(c);
            if natural_base % prime == 0 {
                active_channels.push(c);
                if c < 64 {
                    channel_mask |= 1u64 << c;
                }
            }
        }
        DispatchPlan {
            active_channels,
            natural_base,
            channel_mask,
        }
    }

    /// Plan an addition: the natural base is the LCM of the operands' bases.
    pub fn plan_add(&self, a: &RnsRational, b: &RnsRational) -> DispatchPlan {
        self.plan_for(lcm(a.natural_base(), b.natural_base()))
    }

    /// Plan a multiplication.
    pub fn plan_mul(&self, a: &RnsRational, b: &RnsRational) -> DispatchPlan {
        self.plan_for(lcm(a.natural_base(), b.natural_base()))
    }

    /// Execute an addition (the result is exact regardless of the plan; the plan
    /// is the hardware-scheduling hint).
    pub fn execute_add(&self, a: &RnsRational, b: &RnsRational) -> RnsRational {
        a.add(b)
    }

    /// Execute a multiplication.
    pub fn execute_mul(&self, a: &RnsRational, b: &RnsRational) -> RnsRational {
        a.mul(b)
    }

    /// Fraction of channels that are active for this plan (in `[0, 1]`).
    pub fn channel_efficiency(&self, plan: &DispatchPlan) -> f64 {
        plan.active_channels.len() as f64 / self.channels.len() as f64
    }

    /// Fraction of channels left idle for this plan.
    pub fn channel_idle_fraction(&self, plan: &DispatchPlan) -> f64 {
        1.0 - self.channel_efficiency(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixth_plus_quarter_uses_two_channels() {
        let channels = Channels::standard(16);
        let dispatcher = Dispatcher::new(channels.clone());
        let sixth = RnsRational::from_fraction(1, 6, channels.clone());
        let quarter = RnsRational::from_fraction(1, 4, channels);
        let plan = dispatcher.plan_add(&sixth, &quarter);
        // Natural base involves only primes {2, 3}.
        assert_eq!(plan.active_channels.len(), 2);
        assert_eq!(plan.active_channels, vec![0, 1]); // channels for 2 and 3
        assert!((dispatcher.channel_efficiency(&plan) - 2.0 / 16.0).abs() < 1e-12);
    }

    #[test]
    fn integers_use_no_channels() {
        let channels = Channels::standard(16);
        let dispatcher = Dispatcher::new(channels.clone());
        let a = RnsRational::from_int(3, channels.clone());
        let b = RnsRational::from_int(5, channels);
        let plan = dispatcher.plan_add(&a, &b);
        assert!(plan.active_channels.is_empty());
        assert_eq!(dispatcher.channel_efficiency(&plan), 0.0);
    }

    #[test]
    fn execution_is_exact() {
        let channels = Channels::standard(16);
        let dispatcher = Dispatcher::new(channels.clone());
        let sixth = RnsRational::from_fraction(1, 6, channels.clone());
        let quarter = RnsRational::from_fraction(1, 4, channels.clone());
        // 1/6 + 1/4 = 5/12
        assert_eq!(
            dispatcher.execute_add(&sixth, &quarter),
            RnsRational::from_fraction(5, 12, channels)
        );
    }
}
