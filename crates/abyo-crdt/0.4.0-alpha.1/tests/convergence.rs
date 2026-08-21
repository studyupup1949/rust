//! Property-based convergence tests.
//!
//! Generates random op sequences across multiple replicas and verifies that:
//!
//! 1. **Eventual convergence**: replicas that receive the same set of ops
//!    end up with the same visible sequence, regardless of the order ops
//!    were applied (commutativity + associativity).
//! 2. **Idempotency**: applying the same op multiple times has no extra
//!    effect.
//! 3. **Non-interleaving**: contiguous bursts of insertions by one replica
//!    appear contiguously in the merged result.
//! 4. **Causal liveness**: every accepted op is reflected in the visible
//!    sequence (or, if deleted, marked as such).

use abyo_crdt::{List, ListOp};
use proptest::prelude::*;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Synthetic action against one replica's local state.
#[derive(Debug, Clone)]
enum Action {
    Insert { replica: usize, value: u16 },
    Delete { replica: usize },
}

/// Generator for sequences of actions across `n` replicas.
fn arb_actions(n_replicas: usize, max_actions: usize) -> impl Strategy<Value = Vec<Action>> {
    let replica_strat = 0..n_replicas;
    let action_strat = (replica_strat, 0u16..1000u16, any::<bool>(), any::<bool>()).prop_map(
        |(r, v, do_delete, _)| {
            if do_delete {
                Action::Delete { replica: r }
            } else {
                Action::Insert {
                    replica: r,
                    value: v,
                }
            }
        },
    );
    prop::collection::vec(action_strat, 0..=max_actions)
}

/// Run actions sequentially, but give each replica a chance to fall behind
/// (sync-after-each-action vs. defer-and-batch).
///
/// Returns the final replicas after every replica has merged everyone else's
/// log, so they should all agree.
fn simulate(actions: &[Action], n_replicas: usize, seed: u64) -> Vec<List<u16>> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut replicas: Vec<List<u16>> = (0..n_replicas)
        .map(|i| List::<u16>::new(i as u64 + 1))
        .collect();

    for action in actions {
        match action {
            Action::Insert { replica, value } => {
                let r = &mut replicas[*replica];
                let len = r.len();
                let pos = if len == 0 {
                    0
                } else {
                    use rand::Rng as _;
                    rng.gen_range(0..=len)
                };
                r.insert(pos, *value);
            }
            Action::Delete { replica } => {
                let r = &mut replicas[*replica];
                if !r.is_empty() {
                    use rand::Rng as _;
                    let pos = rng.gen_range(0..r.len());
                    r.delete(pos);
                }
            }
        }
    }

    // Final all-pairs merge with shuffled order to stress commutativity.
    let logs: Vec<Vec<ListOp<u16>>> = replicas.iter().map(|r| r.ops().to_vec()).collect();
    for i in 0..replicas.len() {
        let mut indices: Vec<usize> = (0..replicas.len()).filter(|&j| j != i).collect();
        indices.shuffle(&mut rng);
        for j in indices {
            // Apply in shuffled (but causally-respecting via OpId-sorted internally) order.
            let mut to_apply: Vec<ListOp<u16>> = logs[j]
                .iter()
                .filter(|op| !replicas[i].version().contains(op.id()))
                .cloned()
                .collect();
            to_apply.sort_by_key(ListOp::id);
            for op in to_apply {
                replicas[i].apply(op).unwrap();
            }
        }
    }

    replicas
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        max_shrink_iters: 10_000,
        .. ProptestConfig::default()
    })]

    /// All replicas converge to the same visible sequence, regardless of
    /// merge order.
    #[test]
    fn convergence_2_replicas(actions in arb_actions(2, 30), seed in any::<u64>()) {
        let replicas = simulate(&actions, 2, seed);
        let s0 = replicas[0].to_vec();
        for r in &replicas[1..] {
            prop_assert_eq!(&s0, &r.to_vec());
        }
    }

    #[test]
    fn convergence_3_replicas(actions in arb_actions(3, 30), seed in any::<u64>()) {
        let replicas = simulate(&actions, 3, seed);
        let s0 = replicas[0].to_vec();
        for r in &replicas[1..] {
            prop_assert_eq!(&s0, &r.to_vec());
        }
    }

    #[test]
    fn convergence_5_replicas(actions in arb_actions(5, 50), seed in any::<u64>()) {
        let replicas = simulate(&actions, 5, seed);
        let s0 = replicas[0].to_vec();
        for r in &replicas[1..] {
            prop_assert_eq!(&s0, &r.to_vec());
        }
    }

    /// Applying any prefix of remote ops (in OpId order) and then the rest
    /// always converges — i.e. partial sync is safe.
    #[test]
    fn partial_sync_converges(actions in arb_actions(2, 20), seed in any::<u64>()) {
        let replicas = simulate(&actions, 2, seed);
        let final_state = replicas[0].to_vec();

        // Build a fresh replica by replaying ops in OpId order with random
        // batch boundaries.
        let mut all_ops: Vec<ListOp<u16>> = replicas[0].ops().to_vec();
        all_ops.sort_by_key(ListOp::id);

        let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(1));
        let mut fresh = List::<u16>::new(99);
        let mut idx = 0;
        while idx < all_ops.len() {
            use rand::Rng as _;
            let batch = rng.gen_range(1..=all_ops.len() - idx + 1).min(all_ops.len() - idx);
            for op in &all_ops[idx..idx + batch] {
                fresh.apply(op.clone()).unwrap();
            }
            idx += batch;
        }
        prop_assert_eq!(&fresh.to_vec(), &final_state);
    }

    /// Repeated apply of the same op is a no-op.
    #[test]
    fn idempotency(actions in arb_actions(2, 20), seed in any::<u64>()) {
        let replicas = simulate(&actions, 2, seed);
        let original = replicas[0].clone();
        let mut redundant = original.clone();
        // Re-apply every op several times.
        for op in original.ops().to_vec() {
            redundant.apply(op.clone()).unwrap();
            redundant.apply(op).unwrap();
        }
        prop_assert_eq!(redundant.to_vec(), original.to_vec());
    }

    /// Final visible length equals number of inserts minus number of (effective) deletes.
    #[test]
    fn length_accounting(actions in arb_actions(3, 30), seed in any::<u64>()) {
        let replicas = simulate(&actions, 3, seed);
        // Count distinct insert ops and distinct delete-targets in the merged log.
        let log = replicas[0].ops();
        let inserts = log.iter().filter(|op| matches!(op, ListOp::Insert { .. })).count();
        let mut deleted_targets = std::collections::HashSet::new();
        for op in log {
            if let ListOp::Delete { target, .. } = op {
                deleted_targets.insert(*target);
            }
        }
        let expected_len = inserts - deleted_targets.len();
        prop_assert_eq!(replicas[0].len(), expected_len);
    }
}

// ---------------------------------------------------------------------------
// Non-interleaving — explicit, hand-tuned property test
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct BurstConfig {
    n_replicas: usize,
    burst_size: usize,
}

fn arb_burst() -> impl Strategy<Value = BurstConfig> {
    (2usize..=4usize, 1usize..=8usize).prop_map(|(n, sz)| BurstConfig {
        n_replicas: n,
        burst_size: sz,
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, .. ProptestConfig::default() })]

    /// When `n` replicas, sharing a 2-char prefix, each concurrently insert
    /// a contiguous burst of `burst_size` chars at position 1, the merged
    /// result must contain each replica's burst as a contiguous substring.
    #[test]
    fn non_interleaving(cfg in arb_burst(), seed in any::<u64>()) {
        let mut bootstrap = List::<char>::new(0);
        bootstrap.insert(0, 'L');
        bootstrap.insert(1, 'R');

        let mut replicas: Vec<List<char>> =
            (0..cfg.n_replicas).map(|i| {
                let mut r = List::<char>::new((i + 1) as u64);
                r.merge(&bootstrap);
                r
            }).collect();

        // Each replica types `burst_size` distinct chars at position 1.
        // Use replica id as the char — A, B, C, … — so we can spot interleaving.
        for (i, r) in replicas.iter_mut().enumerate() {
            let ch = (b'A' + i as u8) as char;
            for j in 0..cfg.burst_size {
                r.insert(1 + j, ch);
            }
        }

        // Merge everyone with everyone (in random order).
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        for i in 0..replicas.len() {
            let mut indices: Vec<usize> = (0..replicas.len()).filter(|&j| j != i).collect();
            indices.shuffle(&mut rng);
            for j in indices {
                let other = replicas[j].clone();
                replicas[i].merge(&other);
            }
        }

        let final_state: String = replicas[0].iter().collect();
        for r in &replicas[1..] {
            prop_assert_eq!(&final_state, &r.iter().collect::<String>());
        }

        // Each burst must appear as a contiguous substring.
        for i in 0..cfg.n_replicas {
            let ch = (b'A' + i as u8) as char;
            let burst: String = std::iter::repeat(ch).take(cfg.burst_size).collect();
            prop_assert!(
                final_state.contains(&burst),
                "burst {burst:?} for replica {i} got interleaved in {final_state:?}",
            );
        }
        prop_assert!(final_state.starts_with('L'), "lost L: {final_state}");
        prop_assert!(final_state.ends_with('R'), "lost R: {final_state}");
        // Total length: 2 (L,R) + n_replicas * burst_size
        let expected_len = 2 + cfg.n_replicas * cfg.burst_size;
        prop_assert_eq!(final_state.chars().count(), expected_len);
        // Use the seed to silence the unused warning (proptest surfaces it on shrink).
        let _ = seed;
    }
}
