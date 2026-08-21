//! Exhaustive interleaving check: verify List CRDT convergence under
//! every possible ordering of a small operation set, using stateright's
//! `Model` trait.
//!
//! Unlike `convergence.rs` (proptest, randomized sampling), this is a
//! BFS/DFS over the *complete* finite state space. If it passes, no
//! interleaving in the bounded model breaks convergence.

#![cfg(feature = "serde")]
#![allow(clippy::similar_names)]

use abyo_crdt::{List, ListOp};
use stateright::{Checker as _, Model};

/// State: opaque snapshots of three replicas' lists, plus the set of ops
/// each has yet to deliver to the others.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct ConvergenceModel {
    /// Initial scripted ops to give each replica.
    scripts: Vec<Vec<(usize, u8)>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct ModelState {
    /// Serialized snapshot of each replica's `List<u8>`.
    replicas: Vec<Vec<u8>>,
    /// (sender, receiver) pairs of ops that haven't been delivered yet.
    /// Each entry is the FULL log of `sender` at the time the message was emitted;
    /// `receiver` will diff against its version vector and apply the missing ones.
    pending: Vec<(usize, usize, Vec<u8>)>,
    /// Which scripted op each replica has consumed so far (cursor into scripts[i]).
    cursors: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum Action {
    /// Replica `i` applies its next scripted local op AND broadcasts.
    LocalOp(usize),
    /// Deliver the pending message at `idx`.
    Deliver(usize),
}

fn list_from(snapshot: &[u8]) -> List<u8> {
    bincode::deserialize(snapshot).expect("deserialize")
}

fn list_to(list: &List<u8>) -> Vec<u8> {
    bincode::serialize(list).expect("serialize")
}

impl Model for ConvergenceModel {
    type State = ModelState;
    type Action = Action;

    fn init_states(&self) -> Vec<Self::State> {
        let n = self.scripts.len();
        let replicas: Vec<Vec<u8>> = (0..n)
            .map(|i| list_to(&List::<u8>::new(i as u64 + 1)))
            .collect();
        vec![ModelState {
            replicas,
            pending: Vec::new(),
            cursors: vec![0; n],
        }]
    }

    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        for i in 0..state.replicas.len() {
            if state.cursors[i] < self.scripts[i].len() {
                actions.push(Action::LocalOp(i));
            }
        }
        for idx in 0..state.pending.len() {
            actions.push(Action::Deliver(idx));
        }
    }

    fn next_state(&self, last: &Self::State, action: Self::Action) -> Option<Self::State> {
        let mut next = last.clone();
        match action {
            Action::LocalOp(i) => {
                let mut list = list_from(&next.replicas[i]);
                let (pos, val) = self.scripts[i][next.cursors[i]];
                let len = list.len();
                let p = if len == 0 { 0 } else { pos.min(len) };
                list.insert(p, val);
                next.cursors[i] += 1;
                let log = bincode::serialize(list.ops()).expect("ser ops");
                next.replicas[i] = list_to(&list);
                // Broadcast: queue a delivery for every other replica.
                for j in 0..next.replicas.len() {
                    if j != i {
                        next.pending.push((i, j, log.clone()));
                    }
                }
            }
            Action::Deliver(idx) => {
                let (_sender, receiver, log_bytes) = next.pending.remove(idx);
                let mut list = list_from(&next.replicas[receiver]);
                let ops: Vec<ListOp<u8>> = bincode::deserialize(&log_bytes).expect("deser ops");
                for op in ops {
                    let _ = list.apply(op);
                }
                next.replicas[receiver] = list_to(&list);
            }
        }
        Some(next)
    }

    fn properties(&self) -> Vec<stateright::Property<Self>> {
        vec![stateright::Property::always(
            "all replicas in the quiescent state are equal",
            |_, state: &Self::State| {
                // Quiescence: no pending messages, no remaining script ops.
                if !state.pending.is_empty() {
                    return true; // not yet quiescent
                }
                for i in 0..state.cursors.len() {
                    let _ = i;
                }
                let lists: Vec<Vec<u8>> = state
                    .replicas
                    .iter()
                    .map(|s| list_from(s).to_vec())
                    .collect();
                let r0 = &lists[0];
                lists.iter().all(|r| r == r0)
            },
        )]
    }
}

#[test]
fn stateright_convergence_2_replicas() {
    let model = ConvergenceModel {
        scripts: vec![vec![(0, b'a'), (1, b'b')], vec![(0, b'X')]],
    };
    let checker = model.checker().target_max_depth(20).spawn_bfs();
    checker.join().assert_properties();
}

#[test]
fn stateright_convergence_3_replicas() {
    let model = ConvergenceModel {
        scripts: vec![vec![(0, b'a')], vec![(0, b'b')], vec![(0, b'c')]],
    };
    let checker = model.checker().target_max_depth(15).spawn_bfs();
    checker.join().assert_properties();
}
