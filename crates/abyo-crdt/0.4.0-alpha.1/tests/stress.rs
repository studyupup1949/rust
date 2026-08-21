//! Long-running stress tests. Ignored by default — run with
//! `cargo test --release --test stress -- --ignored`.

#![allow(clippy::needless_range_loop)]

use abyo_crdt::List;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Simulate a session of `n_ops` random operations across `n_replicas`
/// replicas, with random sync points, and verify final convergence + invariants.
fn simulate_session(seed: u64, n_replicas: usize, n_ops: usize) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut replicas: Vec<List<u32>> = (0..n_replicas)
        .map(|i| List::<u32>::new(i as u64 + 1))
        .collect();

    for op_idx in 0..n_ops {
        let r_idx = rng.gen_range(0..n_replicas);

        // 30% chance to sync from a random other replica before acting.
        if rng.gen_bool(0.3) {
            let other_idx = rng.gen_range(0..n_replicas);
            if other_idx != r_idx {
                let other = replicas[other_idx].clone();
                replicas[r_idx].merge(&other);
            }
        }

        // 80% inserts, 20% deletes (when non-empty).
        let r = &mut replicas[r_idx];
        if !r.is_empty() && rng.gen_bool(0.2) {
            let pos = rng.gen_range(0..r.len());
            r.delete(pos);
        } else {
            let pos = rng.gen_range(0..=r.len());
            r.insert(pos, op_idx as u32);
        }
    }

    // All-pairs final merge.
    let snapshots: Vec<List<u32>> = replicas.clone();
    for i in 0..replicas.len() {
        for j in 0..replicas.len() {
            if i != j {
                replicas[i].merge(&snapshots[j]);
            }
        }
    }

    // All replicas must agree.
    let s0 = replicas[0].to_vec();
    for (idx, r) in replicas.iter().enumerate() {
        assert_eq!(s0, r.to_vec(), "replica {idx} diverged");
    }
}

#[test]
fn stress_2_replicas_1k_ops() {
    for seed in 0..20 {
        simulate_session(seed, 2, 1_000);
    }
}

#[test]
fn stress_5_replicas_5k_ops() {
    for seed in 0..5 {
        simulate_session(seed, 5, 5_000);
    }
}

#[test]
#[ignore = "runs >1 minute"]
fn stress_10_replicas_50k_ops() {
    simulate_session(42, 10, 50_000);
}

#[test]
fn long_text_no_concurrency() {
    // Build a 5000-char document by appending. Verifies no stack overflow
    // from deep right-chains and confirms result matches input.
    let mut list = List::<char>::new(1);
    let text: String = ('a'..='z').cycle().take(5_000).collect();
    for (i, c) in text.chars().enumerate() {
        list.insert(i, c);
    }
    let result: String = list.iter().collect();
    assert_eq!(result, text);
}

#[test]
fn long_text_with_random_inserts() {
    // Build a 2000-char document by inserting at random positions. Verifies
    // tree balance under realistic workloads.
    let mut rng = ChaCha8Rng::seed_from_u64(7);
    let mut list = List::<u32>::new(1);
    for i in 0..2_000 {
        let pos = rng.gen_range(0..=list.len());
        list.insert(pos, i);
    }
    assert_eq!(list.len(), 2_000);
}

#[test]
#[ignore = "100K inserts — runs >30s in debug, ~1s in release"]
fn stress_100k_appends() {
    let mut list = List::<u32>::new(1);
    for i in 0..100_000u32 {
        list.insert(i as usize, i);
    }
    assert_eq!(list.len(), 100_000);
    // Spot-check a few positions.
    assert_eq!(list.get(0), Some(&0));
    assert_eq!(list.get(50_000), Some(&50_000));
    assert_eq!(list.get(99_999), Some(&99_999));
}

#[test]
#[ignore = "100K random inserts — runs ~3s in release"]
fn stress_100k_random_inserts() {
    let mut rng = ChaCha8Rng::seed_from_u64(11);
    let mut list = List::<u32>::new(1);
    for i in 0..100_000u32 {
        let pos = if list.is_empty() {
            0
        } else {
            rng.gen_range(0..=list.len())
        };
        list.insert(pos, i);
    }
    assert_eq!(list.len(), 100_000);
}

#[test]
fn deletes_heavy_workload() {
    // 5000 inserts followed by 4000 random deletes.
    use abyo_crdt::List;
    let mut rng = ChaCha8Rng::seed_from_u64(13);
    let mut list = List::<u32>::new(1);
    for i in 0..5_000 {
        let pos = if list.is_empty() {
            0
        } else {
            rng.gen_range(0..=list.len())
        };
        list.insert(pos, i);
    }
    assert_eq!(list.len(), 5_000);
    for _ in 0..4_000 {
        let pos = rng.gen_range(0..list.len());
        list.delete(pos);
    }
    assert_eq!(list.len(), 1_000);
    // Iterate should still work without traversing tombstones individually.
    let collected: Vec<u32> = list.iter().copied().collect();
    assert_eq!(collected.len(), 1_000);
}

#[test]
fn deletes_at_tail_then_gc() {
    // Tombstoned LEAVES (no children) are GC-eligible. We get those by
    // appending then deleting from the END.
    //
    // (A right-chain document where the head is deleted creates
    // tombstones-with-children, which the simple leaf-only GC can't
    // remove. That's a known limitation — `gc` is conservative by
    // design; full reparenting GC is a v0.5+ task.)
    let mut list = List::<u32>::new(1);
    for i in 0..1_000 {
        list.insert(i as usize, i);
    }
    // Delete the last 500 — these are right-chain leaves cascading up.
    for _ in 0..500 {
        list.delete(list.len() - 1);
    }
    assert_eq!(list.len(), 500);
    let frontier = list.version().clone();
    let mut total_gc = 0;
    loop {
        let n = list.gc(&frontier);
        if n == 0 {
            break;
        }
        total_gc += n;
    }
    assert_eq!(total_gc, 500, "all tail tombstones should cascade-GC");
    assert_eq!(list.len(), 500);
}

#[test]
fn mixed_workload_with_undo() {
    use abyo_crdt::ListOp;
    let mut rng = ChaCha8Rng::seed_from_u64(2026);
    let mut list = List::<u32>::new(1);
    let mut undo: Vec<ListOp<u32>> = Vec::new();

    for i in 0..2_000u32 {
        match rng.gen_range(0..10) {
            0..=6 => {
                let pos = if list.is_empty() {
                    0
                } else {
                    rng.gen_range(0..=list.len())
                };
                let op = list.insert(pos, i);
                undo.push(op);
            }
            7..=8 if !list.is_empty() => {
                let pos = rng.gen_range(0..list.len());
                let op = list.delete(pos);
                undo.push(op);
            }
            _ if !undo.is_empty() => {
                // Undo a recent op.
                let op = undo.pop().unwrap();
                list.apply_inverse(&op);
            }
            _ => {}
        }
    }
    // Convergence sanity: serialize+deserialize should round-trip.
    #[cfg(feature = "serde")]
    {
        let bytes = bincode::serialize(&list).unwrap();
        let restored: List<u32> = bincode::deserialize(&bytes).unwrap();
        assert_eq!(list.to_vec(), restored.to_vec());
    }
}
