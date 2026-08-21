use rug::Integer;

use crate::trellis::Trellis;
use crate::utils;

pub fn reverse_trellis_upto_num_sequences(
    num_sequences: Integer,
    n_max: usize,
    weights: &[usize],
) -> Result<Trellis, &'static str> {
    let mut reverse_trellis = Trellis::new_expandable(n_max, weights);
    let weight_levels = reverse_trellis.get_weight_levels();

    // calculate values for higher weight levels
    let mut expand_values: Vec<Integer> = vec![];
    let mut current_num_sequences = Integer::from(0);
    for &wl in weight_levels.iter() {
        let predecessors = reverse_trellis.get_predecessors(wl);
        let predecessor_wls: Vec<usize> =
            predecessors.iter().map(|(_, pred_wl)| *pred_wl).collect();
        for stage in 0..n_max + 1 {
            let node_value: Integer = if wl == 0 && stage == 0 {
                // node at (0, 0) has value 1
                Integer::from(1)
            } else {
                // reverse trellis node value equals sum of its predecessors
                predecessor_wls
                    .iter()
                    .map(|&predecessor_wl| {
                        if stage == 0 {
                            // No predecessors in first stage
                            Integer::from(0)
                        } else if predecessor_wl == wl {
                            // value for predecessor not yet stored in the reverse trellis
                            expand_values
                                .last()
                                .expect("already added one element")
                                .clone()
                        } else {
                            reverse_trellis.get(stage - 1, predecessor_wl)
                        }
                    })
                    .sum()
            };
            expand_values.push(node_value);
        }
        reverse_trellis.expand_with(&mut expand_values)?;

        current_num_sequences += reverse_trellis.get(n_max, wl);
        if current_num_sequences >= num_sequences {
            return Ok(reverse_trellis);
        }
    }

    Err("`num_sequences` is to large")
}

pub fn reverse_trellis_lexicographically_bounded(
    threshold: usize,
    n_max: usize,
    weights: &[usize],
    first_abandoned_sequence: &[usize],
) -> Trellis {
    let mut reverse_trellis = Trellis::new(threshold, n_max, weights);
    let abandoned_seq_wls = utils::cumsum(
        &first_abandoned_sequence
            .iter()
            .map(|a| (a - 1) / 2) // amplitude -> weight index
            .map(|w_idx| weights[w_idx]) // weight index -> weight
            .collect::<Vec<usize>>(),
    );

    let weight_levels = reverse_trellis.get_weight_levels();
    for n in 0..n_max {
        for &wl in &weight_levels {
            let next_stage = n + 1;
            for (_, next_wl) in reverse_trellis.get_successors(wl) {
                if wl == abandoned_seq_wls[n] && next_wl < abandoned_seq_wls[next_stage] {
                    reverse_trellis.add(next_stage, next_wl, reverse_trellis.get(n, wl) + 1);
                } else {
                    reverse_trellis.add(next_stage, next_wl, reverse_trellis.get(n, wl));
                }
            }
        }
    }

    reverse_trellis
}

pub fn pprint_trellis(trellis: &Trellis) {
    fn integer_to_str(integer: &Integer) -> String {
        format!(" {:>5}", integer.to_string())
    }

    let weight_levels: Vec<usize> = trellis
        .get_weight_levels()
        .into_iter()
        .take(trellis.get_num_weight_levels())
        .rev()
        .collect();

    let wl_strs = weight_levels.iter().map(|wl| {
        (0..trellis.n_max + 1)
            .map(|stage| integer_to_str(&trellis.get(stage, *wl)))
            .collect::<String>()
    });

    for (wl_str, wl) in wl_strs.zip(&weight_levels) {
        println!("{wl:<5}| {wl_str}");
    }
}
