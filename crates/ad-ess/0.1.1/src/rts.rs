use rug::Complete;
use rug::Integer;
use rug::Rational;

use crate::trellis::Trellis;
use crate::trellis_utils;
use crate::utils;

pub struct RTS {
    pub trellis: Trellis,
}

impl RTS {
    /// Returns an [RTS] instance which encodes at least `num_bits` bits
    ///
    /// The smallest possible trellis that encodes `num_bits` bits is used, in
    /// some cases this trellis is capable of encoding more than `num_bits` bits.
    pub fn new(num_bits: usize, n_max: usize, weights: &[usize]) -> RTS {
        let trellis = trellis_utils::reverse_trellis_upto_num_sequences(
            Integer::u_pow_u(2, num_bits as u32).complete(),
            n_max,
            weights,
        )
        .unwrap();
        RTS { trellis }
    }

    /// Returns the amplitude value for a given weight index
    fn weight_idx_to_amplitude(weight_index: usize) -> usize {
        weight_index * 2 + 1
    }
    /// Returns the weight index for a given amplitude value
    fn amplitude_to_weight_idx(amplitude: usize) -> usize {
        (amplitude - 1) / 2
    }
    /// Replaces the amplitudes in a sequence by their weight indexes
    fn amplitude_seq_to_weight_idx_seq(amplitude_sequence: &[usize]) -> Vec<usize> {
        amplitude_sequence.iter().map(|a| (a - 1) / 2).collect()
    }
}

impl RTS {
    /// Returns the number of sequences that can be encoded / decoded
    pub fn num_sequences(&self) -> Integer {
        let n_max = self.trellis.n_max;
        self.trellis.get_stage(n_max).iter().sum()
    }
    /// Returns the number of bits that can be encoded / decoded
    pub fn num_bits(&self) -> u32 {
        self.num_sequences().significant_bits() - 1
    }
    /// Returns the weights used by the internal trellis
    pub fn get_weights(&self) -> Vec<usize> {
        self.trellis.get_weights()
    }
    /// Returns the distribution [RTS] is optimizing for
    pub fn get_distribution(&self, res_factor: f32) -> Vec<f32> {
        utils::distribution_from_weights(&self.get_weights(), res_factor)
    }
    /// Returns the amplitude sequence for a given index
    pub fn sequence_for_index(&self, index: &Integer) -> Vec<usize> {
        assert!(index < &self.num_sequences(), "Index out of range!");

        let n_max = self.trellis.n_max;
        let mut wl_path = vec![0usize; n_max + 1];

        let mut lower_nodes_sum = Integer::from(0);
        for (wl_idx, node_value) in self.trellis.get_stage(n_max).iter().enumerate() {
            lower_nodes_sum += node_value;
            if &lower_nodes_sum > index {
                wl_path[n_max] = self.trellis.get_weight_levels()[wl_idx];
                lower_nodes_sum -= node_value;
                break;
            }
        }
        let mut local_index = index - lower_nodes_sum;
        let mut weight_idx_seq = vec![0usize; n_max];
        for stage in (0..n_max).rev() {
            lower_nodes_sum = Integer::from(0);
            // caching predecessors may improve speed
            for (w_idx, pred_wl) in self.trellis.get_predecessors(wl_path[stage + 1]) {
                let node_value = self.trellis.get(stage, pred_wl);

                lower_nodes_sum += &node_value;
                if lower_nodes_sum > local_index {
                    wl_path[stage] = pred_wl;
                    weight_idx_seq[stage] = w_idx;
                    lower_nodes_sum -= &node_value;
                    break;
                }
            }
            local_index -= lower_nodes_sum;
        }
        weight_idx_seq
            .iter()
            .map(|&weight_idx| RTS::weight_idx_to_amplitude(weight_idx))
            .collect()
    }
    /// Returns the index for a given amplitude sequence
    pub fn index_for_sequence(&self, amplitude_sequence: &[usize]) -> Integer {
        let n_max = self.trellis.n_max;

        let weight_idx_seq = RTS::amplitude_seq_to_weight_idx_seq(amplitude_sequence);
        let weights = self.trellis.get_weights();
        let weight_seq: Vec<usize> = weight_idx_seq.iter().map(|&w_idx| weights[w_idx]).collect();
        let wl_path = utils::cumsum(&weight_seq);

        let num_lower_end_nodes = self.trellis.get_weight_level_index(wl_path[n_max]);

        let mut index: Integer = self
            .trellis
            .get_stage(self.trellis.n_max)
            .iter()
            .take(num_lower_end_nodes)
            .sum();

        for (idx, (&weight_idx, wl_transition)) in weight_idx_seq
            .iter()
            .zip(wl_path.windows(2))
            .enumerate()
            .rev()
        {
            let stage = idx + 1;
            if let &[predecessor_wl, wl] = wl_transition {
                self.trellis
                    .get_predecessors(wl)
                    .iter()
                    .take_while(|(possible_weight_idx, possible_predecessor_wl)| {
                        *possible_predecessor_wl <= predecessor_wl
                            && *possible_weight_idx != weight_idx
                    })
                    .for_each(|(_, possible_predecessor_wl)| {
                        index += self.trellis.get(stage - 1, *possible_predecessor_wl);
                    });
            } else {
                panic!("`window(2)` produced a window of length != 2");
            }
        }

        index
    }
    fn count_amplitude_in_stage(
        &self,
        amplitude: usize,
        stage: usize,
        first_abandoned_seq: &[usize],
    ) -> Integer {
        let the_w_idx = RTS::amplitude_to_weight_idx(amplitude);
        let the_weight = self.trellis.get_weight(the_w_idx);
        let the_stage = stage;

        let n_max = self.trellis.n_max;

        let fas_w_idxs = RTS::amplitude_seq_to_weight_idx_seq(first_abandoned_seq);
        let fas_weights: Vec<usize> = fas_w_idxs
            .iter()
            .map(|&w_idx| self.trellis.get_weight(w_idx))
            .collect();
        let fas_wls = utils::cumsum(&fas_weights);

        // calculation is split according to the stage in which the considered sequences join the
        // first abandoned sequence (FAS)
        let mut amplitude_count = Integer::from(0);

        // sequences that never join the FAS
        amplitude_count += self
            .trellis
            .get_weight_levels()
            .iter()
            .take_while(|wl| *wl < fas_wls.last().unwrap())
            .skip_while(|wl| **wl < the_weight) // ensure `wl - the_weight` is positive
            .map(|wl| self.trellis.get_or_0(n_max - 1, *wl - the_weight))
            .sum::<Integer>();

        // sequences that join the FAS between stages `the_stage` + 2 and `n_max`
        amplitude_count += (the_stage + 2..n_max + 1)
            .flat_map(|stage| {
                let ref_fas_w_idxs = &fas_w_idxs;
                self.trellis
                    .get_predecessors(fas_wls[stage])
                    .into_iter()
                    .take_while(move |(w_idx, _)| *w_idx != ref_fas_w_idxs[stage - 1])
                    // ensure `predecessor_wl - the_weight >= 0`
                    .skip_while(|(_, predecessor_wl)| *predecessor_wl < the_weight)
                    .map(move |(_, predecessor_wl)| {
                        self.trellis
                            .get_or_0(stage - 2, predecessor_wl - the_weight)
                    })
            })
            .sum::<Integer>();

        // sequences that join the FAS in stage `the_stage` + 1
        if (the_weight > fas_weights[the_stage]
            // this condition depends on the internal ordering used by [Trellis::get_predecessor]
            // if code is changed there this code might break
            || (the_weight == fas_weights[the_stage] && the_w_idx > fas_w_idxs[the_stage]))
        // ensure `fas_wls[the_stage + 1] - the_weight >= 0`
        && fas_wls[the_stage + 1] >= the_weight
        {
            amplitude_count += self
                .trellis
                .get_or_0(the_stage, fas_wls[the_stage + 1] - the_weight)
        }

        // sequences that join the FAS in stage `the_stage` or before
        if the_w_idx == fas_w_idxs[the_stage] {
            amplitude_count += (1..the_stage + 1)
                .flat_map(|stage| {
                    let ref_fas_w_idxs = &fas_w_idxs;
                    self.trellis
                        .get_predecessors(fas_wls[stage])
                        .into_iter()
                        .take_while(move |(w_idx, _)| *w_idx != ref_fas_w_idxs[stage - 1])
                        .map(move |(_, predecessor_wl)| {
                            self.trellis.get_or_0(stage - 1, predecessor_wl)
                        })
                })
                .sum::<Integer>();
        }

        amplitude_count
    }
    /// Returns the amplitude distribution as a [Vec]
    ///
    /// The amplitude distribution is valid if only sequences with indexes
    /// representable with [self.num_bits] bits are used.
    pub fn amplitude_distribution(&self) -> Vec<f32> {
        let num_sequences_used = Integer::u_pow_u(2, self.num_bits()).complete();
        if num_sequences_used == self.num_sequences() {
            return self.amplitude_distribution_full_utilization();
        }

        let first_abandoned_seq = self.sequence_for_index(&num_sequences_used);
        let n_max = self.trellis.n_max;

        let num_weights = self.trellis.get_weights().len();
        let amplitudes = (0..num_weights).map(RTS::weight_idx_to_amplitude);

        let amplitude_counts = amplitudes.map(|amplitude| {
            (0..n_max)
                .map(|stage| self.count_amplitude_in_stage(amplitude, stage, &first_abandoned_seq))
                .sum::<Integer>()
        });

        amplitude_counts
            .map(|amplitude_count| {
                Rational::from((&amplitude_count, &num_sequences_used * n_max)).to_f32()
            })
            .collect()
    }
    /// Returns the amplitude distribution as a [Vec]
    ///
    /// The amplitude distribution is valid if all sequences in the trellis
    /// are used equiprobably.
    pub fn amplitude_distribution_full_utilization(&self) -> Vec<f32> {
        let n_max = self.trellis.n_max;
        let weight_levels = self.trellis.get_weight_levels();
        let threshold = self.trellis.threshold;
        let num_sequences = self.num_sequences();

        self.trellis
            .get_weights()
            .iter()
            .map(|weight| {
                let num_weight_occurences: Integer = weight_levels
                    .iter()
                    .take_while(|wl| *wl + *weight <= threshold)
                    .map(|wl| self.trellis.get(n_max - 1, *wl))
                    .sum();

                Rational::from((&num_weight_occurences, &num_sequences)).to_f32()
            })
            .collect()
    }
    // /// Returns the average energy
    // ///
    // /// Assumes only indexes representable with [self.num_bits] bits are used.
    // pub fn average_energy(&self) -> f32 {
    // let amplitude_distribution = self.amplitude_distribution();
    // amplitude_distribution
    // .iter()
    // .enumerate()
    // .map(|(w_idx, p)| (RTS::weight_idx_to_amplitude(w_idx) as f32, p))
    // .map(|(a, p)| a * a * p) // expected value of energy == squared amplitude * probability
    // .sum::<f32>()
    // }
}
