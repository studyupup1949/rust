//! Property-based convergence tests for the [`Text`] CRDT (v0.3-alpha
//! Peritext-style rich text).

#![allow(clippy::needless_range_loop)]

use abyo_crdt::{MarkSet, Text};
use proptest::prelude::*;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Debug, Clone)]
enum TextAction {
    Insert {
        replica: usize,
        value: char,
    },
    Delete {
        replica: usize,
    },
    SetMark {
        replica: usize,
        name: u8,
        on: bool,
        range_offset: u8,
        range_len: u8,
    },
}

fn arb_text_actions(n_replicas: usize, max: usize) -> impl Strategy<Value = Vec<TextAction>> {
    let strat = (
        0..n_replicas,
        any::<u32>(),
        0u8..3,
        any::<bool>(),
        0u8..16,
        0u8..16,
    )
        .prop_map(|(r, kind, name, on, ro, rl)| match kind % 5 {
            0..=2 => TextAction::Insert {
                replica: r,
                // Restrict to a small alphabet so we get repetition.
                value: char::from(b'a' + (kind as u8 % 26)),
            },
            3 => TextAction::Delete { replica: r },
            _ => TextAction::SetMark {
                replica: r,
                name,
                on,
                range_offset: ro,
                range_len: rl,
            },
        });
    prop::collection::vec(strat, 0..=max)
}

fn simulate(actions: &[TextAction], n_replicas: usize, seed: u64) -> Vec<Text> {
    let mut replicas: Vec<Text> = (0..n_replicas).map(|i| Text::new(i as u64 + 1)).collect();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    for action in actions {
        match action {
            TextAction::Insert { replica, value } => {
                use rand::Rng as _;
                let r = &mut replicas[*replica];
                let len = r.len();
                let pos = if len == 0 { 0 } else { rng.gen_range(0..=len) };
                r.insert(pos, *value);
            }
            TextAction::Delete { replica } => {
                use rand::Rng as _;
                let r = &mut replicas[*replica];
                if !r.is_empty() {
                    let pos = rng.gen_range(0..r.len());
                    r.delete(pos);
                }
            }
            TextAction::SetMark {
                replica,
                name,
                on,
                range_offset,
                range_len,
            } => {
                let r = &mut replicas[*replica];
                let len = r.len();
                if len == 0 {
                    continue;
                }
                let start = (*range_offset as usize) % len;
                let max_len = len - start;
                let span_len = if max_len == 0 {
                    0
                } else {
                    (*range_len as usize) % max_len
                };
                let end = start + span_len;
                let name_str = match name {
                    0 => "bold",
                    1 => "italic",
                    _ => "underline",
                };
                if start < end {
                    r.set_mark(start..end, name_str, *on);
                }
            }
        }
    }

    let snapshot: Vec<Text> = replicas.clone();
    for i in 0..replicas.len() {
        let mut order: Vec<usize> = (0..replicas.len()).filter(|&j| j != i).collect();
        order.shuffle(&mut rng);
        for j in order {
            replicas[i].merge(&snapshot[j]);
        }
    }
    replicas
}

fn render(t: &Text) -> Vec<(char, Vec<String>)> {
    t.iter_with_marks()
        .map(|(c, m): (char, MarkSet)| (c, m.iter().map(String::from).collect()))
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, .. ProptestConfig::default() })]

    #[test]
    fn text_2_replicas_converge(actions in arb_text_actions(2, 30), seed in any::<u64>()) {
        let replicas = simulate(&actions, 2, seed);
        let r0 = render(&replicas[0]);
        prop_assert_eq!(&r0, &render(&replicas[1]));
    }

    #[test]
    fn text_4_replicas_converge(actions in arb_text_actions(4, 50), seed in any::<u64>()) {
        let replicas = simulate(&actions, 4, seed);
        let r0 = render(&replicas[0]);
        for i in 1..replicas.len() {
            prop_assert_eq!(&r0, &render(&replicas[i]));
        }
    }

    #[test]
    fn text_idempotent(actions in arb_text_actions(3, 30), seed in any::<u64>()) {
        let replicas = simulate(&actions, 3, seed);
        let original = replicas[0].clone();
        let mut redundant = original.clone();
        for op in original.ops().to_vec() {
            redundant.apply(op.clone()).unwrap();
            redundant.apply(op).unwrap();
        }
        prop_assert_eq!(render(&redundant), render(&original));
    }
}
