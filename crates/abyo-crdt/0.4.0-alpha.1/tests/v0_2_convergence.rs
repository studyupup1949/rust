//! Property-based convergence tests for the v0.2 CRDTs (Map, Counter, Set).
//!
//! Generates randomized op sequences and verifies that all replicas
//! converge to the same final state, regardless of the order ops are
//! applied.

#![allow(clippy::needless_range_loop)]

use abyo_crdt::{Counter, Map, Set};
use proptest::prelude::*;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// ---------------------------------------------------------------------------
// Map convergence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum MapAction {
    Set { replica: usize, key: u8, value: i32 },
    Remove { replica: usize, key: u8 },
}

fn arb_map_actions(n_replicas: usize, max: usize) -> impl Strategy<Value = Vec<MapAction>> {
    let strat =
        (0..n_replicas, 0u8..16, any::<i32>(), any::<bool>()).prop_map(|(r, k, v, do_remove)| {
            if do_remove {
                MapAction::Remove { replica: r, key: k }
            } else {
                MapAction::Set {
                    replica: r,
                    key: k,
                    value: v,
                }
            }
        });
    prop::collection::vec(strat, 0..=max)
}

fn map_simulate(actions: &[MapAction], n_replicas: usize, seed: u64) -> Vec<Map<u8, i32>> {
    let mut replicas: Vec<Map<u8, i32>> = (0..n_replicas)
        .map(|i| Map::<u8, i32>::new(i as u64 + 1))
        .collect();
    for action in actions {
        match action {
            MapAction::Set {
                replica,
                key,
                value,
            } => {
                replicas[*replica].set(*key, *value);
            }
            MapAction::Remove { replica, key } => {
                replicas[*replica].remove(*key);
            }
        }
    }
    // Random-order all-pairs merge.
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let snapshot: Vec<Map<u8, i32>> = replicas.clone();
    for i in 0..replicas.len() {
        let mut indices: Vec<usize> = (0..replicas.len()).filter(|&j| j != i).collect();
        indices.shuffle(&mut rng);
        for j in indices {
            replicas[i].merge(&snapshot[j]);
        }
    }
    replicas
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, .. ProptestConfig::default() })]

    #[test]
    fn map_converges_2_replicas(actions in arb_map_actions(2, 30), seed in any::<u64>()) {
        let replicas = map_simulate(&actions, 2, seed);
        let r0: Vec<_> = {
            let mut v: Vec<_> = replicas[0].iter().map(|(k,v)| (*k, *v)).collect();
            v.sort_unstable();
            v
        };
        for r in &replicas[1..] {
            let mut other: Vec<_> = r.iter().map(|(k,v)| (*k, *v)).collect();
            other.sort_unstable();
            prop_assert_eq!(&r0, &other);
        }
    }

    #[test]
    fn map_converges_4_replicas(actions in arb_map_actions(4, 50), seed in any::<u64>()) {
        let replicas = map_simulate(&actions, 4, seed);
        let r0: Vec<_> = {
            let mut v: Vec<_> = replicas[0].iter().map(|(k,v)| (*k, *v)).collect();
            v.sort_unstable();
            v
        };
        for r in &replicas[1..] {
            let mut other: Vec<_> = r.iter().map(|(k,v)| (*k, *v)).collect();
            other.sort_unstable();
            prop_assert_eq!(&r0, &other);
        }
    }
}

// ---------------------------------------------------------------------------
// Counter convergence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct CounterAction {
    replica: usize,
    delta: i32,
}

fn arb_counter_actions(n_replicas: usize, max: usize) -> impl Strategy<Value = Vec<CounterAction>> {
    let strat = (0..n_replicas, -100i32..100i32)
        .prop_map(|(replica, delta)| CounterAction { replica, delta });
    prop::collection::vec(strat, 0..=max)
}

fn counter_simulate(actions: &[CounterAction], n_replicas: usize, seed: u64) -> Vec<Counter> {
    let mut replicas: Vec<Counter> = (0..n_replicas)
        .map(|i| Counter::new(i as u64 + 1))
        .collect();
    for action in actions {
        replicas[action.replica].add(i64::from(action.delta));
    }
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let snapshot: Vec<Counter> = replicas.clone();
    for i in 0..replicas.len() {
        let mut indices: Vec<usize> = (0..replicas.len()).filter(|&j| j != i).collect();
        indices.shuffle(&mut rng);
        for j in indices {
            replicas[i].merge(&snapshot[j]);
        }
    }
    replicas
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, .. ProptestConfig::default() })]

    #[test]
    fn counter_converges(actions in arb_counter_actions(4, 60), seed in any::<u64>()) {
        let replicas = counter_simulate(&actions, 4, seed);
        let v0 = replicas[0].value();
        for r in &replicas[1..] {
            prop_assert_eq!(v0, r.value());
        }
        // Sanity: final value must equal the sum of all deltas.
        let expected: i128 = actions.iter().map(|a| i128::from(a.delta)).sum();
        prop_assert_eq!(v0, expected);
    }

    #[test]
    fn counter_idempotent(actions in arb_counter_actions(3, 30), seed in any::<u64>()) {
        let replicas = counter_simulate(&actions, 3, seed);
        let original = replicas[0].clone();
        let mut redundant = original.clone();
        for op in original.ops().to_vec() {
            redundant.apply(op).unwrap();
            redundant.apply(op).unwrap();
        }
        prop_assert_eq!(redundant.value(), original.value());
    }
}

// ---------------------------------------------------------------------------
// Set convergence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum SetAction {
    Add { replica: usize, value: u8 },
    Remove { replica: usize, value: u8 },
}

fn arb_set_actions(n_replicas: usize, max: usize) -> impl Strategy<Value = Vec<SetAction>> {
    let strat = (0..n_replicas, 0u8..16, any::<bool>()).prop_map(|(r, v, do_remove)| {
        if do_remove {
            SetAction::Remove {
                replica: r,
                value: v,
            }
        } else {
            SetAction::Add {
                replica: r,
                value: v,
            }
        }
    });
    prop::collection::vec(strat, 0..=max)
}

fn set_simulate(actions: &[SetAction], n_replicas: usize, seed: u64) -> Vec<Set<u8>> {
    let mut replicas: Vec<Set<u8>> = (0..n_replicas)
        .map(|i| Set::<u8>::new(i as u64 + 1))
        .collect();
    for action in actions {
        match action {
            SetAction::Add { replica, value } => {
                replicas[*replica].add(*value);
            }
            SetAction::Remove { replica, value } => {
                replicas[*replica].remove(value);
            }
        }
    }
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let snapshot: Vec<Set<u8>> = replicas.clone();
    for i in 0..replicas.len() {
        let mut indices: Vec<usize> = (0..replicas.len()).filter(|&j| j != i).collect();
        indices.shuffle(&mut rng);
        for j in indices {
            replicas[i].merge(&snapshot[j]);
        }
    }
    replicas
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, .. ProptestConfig::default() })]

    #[test]
    fn set_converges(actions in arb_set_actions(4, 60), seed in any::<u64>()) {
        let replicas = set_simulate(&actions, 4, seed);
        let r0: Vec<u8> = {
            let mut v: Vec<u8> = replicas[0].iter().copied().collect();
            v.sort_unstable();
            v
        };
        for r in &replicas[1..] {
            let mut other: Vec<u8> = r.iter().copied().collect();
            other.sort_unstable();
            prop_assert_eq!(&r0, &other);
        }
    }

    #[test]
    fn set_idempotent(actions in arb_set_actions(3, 30), seed in any::<u64>()) {
        let replicas = set_simulate(&actions, 3, seed);
        let original = replicas[0].clone();
        let mut redundant = original.clone();
        for op in original.ops().to_vec() {
            redundant.apply(op.clone()).unwrap();
            redundant.apply(op).unwrap();
        }
        let mut a: Vec<u8> = original.iter().copied().collect();
        a.sort_unstable();
        let mut b: Vec<u8> = redundant.iter().copied().collect();
        b.sort_unstable();
        prop_assert_eq!(a, b);
    }
}
