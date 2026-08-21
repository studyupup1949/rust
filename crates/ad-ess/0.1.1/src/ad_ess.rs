use rug::Complete;
use rug::Integer;
use rug::Rational;

use crate::trellis::Trellis;
use crate::trellis_utils;
use crate::utils::{cumsum, entropy, kl_divergence};

/// Arbitrary-Distribution ESS (AD-ESS)
///
/// This struct contains methods to [encode](AdEss::sequence_for_index) and
/// [decode](AdEss::index_for_sequence) using AD-ESS.
/// Additional methods calculate the [amplitude distribution](AdEss::amplitude_distribution),
/// [average energy](AdEss::average_energy) or other metrics.
pub struct AdEss {
    pub trellis: Trellis,
}

impl AdEss {
    /// Returns a new [AdEss] instance given weights
    ///
    /// The trellis is calculated with `n_max` stages using the weights `weights` and holds
    /// sequences with a sum weight up to `threshold`.
    pub fn new(threshold: usize, n_max: usize, weights: &[usize]) -> AdEss {
        let trellis = Trellis::new(threshold, n_max, weights);
        let mut instance = AdEss { trellis };
        instance.calc_forward_trellis();
        instance
    }

    /// Returns a new [AdEss] instance given a distribution
    ///
    /// The trellis is calculated with `n_max` stages using weights computed via
    /// [AdEss::calc_weights()] and holds sequences with a sum weight up to `threshold`.
    ///
    /// `distribution` and `res_factor` are passed to [AdEss::calc_weights()].
    ///
    /// A new [AdEss] instance and the target distribution [AdEss::get_distribution()] are returned.
    pub fn new_for_distribution_threshold(
        threshold: usize,
        n_max: usize,
        distribution: &[f32],
        res_factor: f32,
    ) -> Result<(AdEss, Vec<f32>), &'static str> {
        let weights = AdEss::calc_weights(distribution, res_factor)?;
        let adess = AdEss::new(threshold, n_max, &weights);
        let p_goal = adess.get_distribution(res_factor);

        Ok((adess, p_goal))
    }

    /// Returns an [AdEss] instance which encodes at least `num_bits` bits
    ///
    /// The smallest possible trellis that encodes `num_bits` bits is used, in
    /// some cases this trellis is capable of encoding more than `num_bits` bits.
    ///
    /// The trellis is calculated with `n_max` stages using weights computed via
    /// [AdEss::calc_weights()], `distribution` and `res_factor` are passed to [AdEss::calc_weights()].
    ///
    /// A new [AdEss] instance and the target distribution [AdEss::get_distribution()] are returned.
    pub fn new_for_distribution_num_bits(
        num_bits: usize,
        n_max: usize,
        distribution: &[f32],
        res_factor: f32,
    ) -> Result<(AdEss, Vec<f32>), &'static str> {
        let weights = AdEss::calc_weights(distribution, res_factor)?;

        let num_sequences = Integer::u_pow_u(2, num_bits as u32).complete();
        let reverse_trellis =
            trellis_utils::reverse_trellis_upto_num_sequences(num_sequences, n_max, &weights)?;
        let threshold = reverse_trellis.threshold;

        let adess = AdEss::new(threshold, n_max, &weights);
        let p_goal = adess.get_distribution(res_factor);

        Ok((adess, p_goal))
    }

    /// Returns an [AdEss] whith a threshold chosen to maximizes the lower bound on mutual information
    ///
    /// According to formulas (13) and (14) in <https://doi.org/10.1109/LWC.2018.2890595>
    ///
    /// The trellis is calculated with `n_max` stages using weights computed via
    /// [AdEss::calc_weights()], `distribution` and `res_factor` are passed to [AdEss::calc_weights()].
    ///
    /// - `search_width`: number of weight levels to check below and above the initial estimated
    ///   optimal threshold
    /// - `rev_trellis_calculation_fraction`: the fraction of the reverse trellis that should be
    ///   calculated. If the calculated fraction is to small, the optimal threshold can not be found.
    pub fn new_for_distribution_optimal_threshold(
        n_max: usize,
        distribution: &Vec<f32>,
        res_factor: f32,
        search_width: usize,
        rev_trellis_calculation_fraction: f32,
    ) -> Result<(AdEss, Vec<f32>), &'static str> {
        let threshold = AdEss::optimal_threshold(
            n_max,
            distribution,
            res_factor,
            search_width,
            rev_trellis_calculation_fraction,
        )?;

        let result =
            AdEss::new_for_distribution_threshold(threshold, n_max, distribution, res_factor)?;

        Ok(result)
    }

    /// Compute weights from a probability distribution
    ///
    /// `distribution` is a slice/vec of (amplitude) probabilities, i.e., `sum(distribution) == 1`
    ///
    /// The `res_factor` controls a trade off between trellis size and distribution quantisation.
    /// High `res_factor` leads to fine quantisation but a potentially large trellis.
    pub fn calc_weights(distribution: &[f32], res_factor: f32) -> Result<Vec<usize>, &'static str> {
        let weights: Vec<f32> = distribution
            .iter()
            .map(|p| -p.log2() * res_factor)
            .collect();
        let min_weight = weights.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let weights = weights.iter().map(|weight| weight - min_weight);
        // + 0.5 to convert floor to integer rounding
        let weights: Vec<usize> = weights.map(|weight| (weight + 0.5) as usize).collect();
        Ok(weights)
    }

    /// Fill `self.trellis` with values
    fn calc_forward_trellis(&mut self) {
        for n in (0..self.trellis.n_max + 1).rev() {
            for wl in self.trellis.get_weight_levels() {
                if n == self.trellis.n_max {
                    // number of possible sequences for end nodes is 1
                    self.trellis.set(n, wl, Integer::from(1));
                } else {
                    // number of possible paths for a node is the sum of the number
                    // of possible sequences of all successor nodes
                    for (_, next_wl) in self.trellis.get_successors(wl) {
                        self.trellis.add(n, wl, self.trellis.get(n + 1, next_wl));
                    }
                }
                // debugging output
                //println!("n: {}, wl: {}, value: {}", n, wl, self.trellis.get(n, wl));
            }
        }
    }
    /// Returns the amplitude value for a given weight index
    fn weight_idx_to_amplitude(weight_index: usize) -> usize {
        weight_index * 2 + 1
    }
    /// Replaces the amplitudes in a seqence by their weight indexes
    fn amplitude_seq_to_weight_idx_seq(amplitude_sequence: &[usize]) -> Vec<usize> {
        amplitude_sequence.iter().map(|a| (a - 1) / 2).collect()
    }

    /// Returns a reverse trellis with the given parameters
    ///
    /// A reverse trellis is a trellis like the ESS trellis but trellis values are calculated in
    /// the reversed direction, i.e., not from last stage to first but starting at the first stage.
    /// For details see <https://doi.org/10.1109/JLT.2022.3201901>.
    ///
    /// The trellis is calculated with `n_max` stages using the weights `weights` and holds
    /// sequences with a sum weight up to `threshold`.
    pub fn calc_reverse_trellis(threshold: usize, n_max: usize, weights: &[usize]) -> Trellis {
        let mut rev_trellis = Trellis::new(threshold, n_max, weights);
        rev_trellis.set(0, 0, Integer::from(1));

        for n in 0..rev_trellis.n_max {
            for wl in rev_trellis.get_weight_levels() {
                let current_wl_val = rev_trellis.get(n, wl);
                for (_, next_wl) in rev_trellis.get_successors(wl) {
                    rev_trellis.add(n + 1, next_wl, current_wl_val.clone());
                }
            }
        }
        rev_trellis
    }
    /// Calculates the threshold that maximizes the lower bound on mutual-information
    ///
    /// According to formulas (13) and (14) in <https://doi.org/10.1109/LWC.2018.2890595>
    ///
    /// - `n_max`: number of stages in the trellis
    /// - `distribution`: (amplitude) probability mass function as a [Vec]
    /// - `res_factor`: trade-off between trellis size and distribution quantisation (-> see
    ///   [AdEss::calc_weights()]
    /// - `search_width`: number of weight levels to check below and above the initial estimated
    ///   optimal threshold
    /// - `rev_trellis_calculation_fraction`: the fraction of the reverse trellis that should be
    ///   calculated. If the calculated fraction is to small, the optimal threshold can not be found.
    pub fn optimal_threshold(
        n_max: usize,
        distribution: &Vec<f32>,
        res_factor: f32,
        search_width: usize,
        rev_trellis_calculation_fraction: f32,
    ) -> Result<usize, &'static str> {
        // this code could be significantly improved using
        // `trellis_utils::reverse_trellis_upto_num_sequences` the function
        // argument `rev_trellis_calculation_fraction` would no longer be necessary
        println!("WARNING: Code has not been checked with non-unique weights!");
        let weights = AdEss::calc_weights(distribution, res_factor)?;

        let max_possible_wl = (weights.iter().max().unwrap() * n_max) as f32;
        let rev_trellis_threshold = (max_possible_wl * rev_trellis_calculation_fraction) as usize;
        let rev_trellis = AdEss::calc_reverse_trellis(rev_trellis_threshold, n_max, &weights);

        let code_sizes = rev_trellis
            .get_stage(n_max)
            .iter()
            .fold(vec![], |mut total, wl_val| {
                if total.is_empty() {
                    total.push(wl_val.clone());
                } else {
                    total.push(Integer::from(wl_val + &total[total.len() - 1]));
                }
                total
            });
        let weight_levels = rev_trellis.get_weight_levels();

        let estimated_optimal_size =
            Integer::from_f64(2.0_f64.powf(n_max as f64 * entropy(distribution) as f64)).unwrap();
        let estimated_optimal_wl_idx = code_sizes.iter().position(|x| x >= &estimated_optimal_size);
        let estimated_optimal_wl_idx = match estimated_optimal_wl_idx {
            Some(wl_idx) => wl_idx,
            None => return Err("The calculated fraction of the reverse trellis is to small!"),
        };

        let mut max_mi_losses = vec![];
        let mut tested_wl_idxs = vec![];
        let search_start_wl_idx = estimated_optimal_wl_idx - search_width;
        let search_end_wl_idx = estimated_optimal_wl_idx + search_width;
        for (wl_idx, &threshold) in weight_levels
            .iter()
            .enumerate()
            .take(search_end_wl_idx)
            .skip(search_start_wl_idx)
        {
            let (adess, _) =
                AdEss::new_for_distribution_threshold(threshold, n_max, distribution, res_factor)?;

            let amp_distr = adess.amplitude_distribution();
            let n = n_max as f32;
            let log2_code_size = (adess.num_sequences().significant_bits() - 1) as f32;
            let amplitude_kl = kl_divergence(&amp_distr, distribution);

            // upper bound on reduction in mutual information
            let max_mi_loss = entropy(&amp_distr) - log2_code_size / n + amplitude_kl;
            max_mi_losses.push(max_mi_loss);
            tested_wl_idxs.push(wl_idx);
        }

        let min_max_mi_loss = max_mi_losses.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let min_loss_idx = max_mi_losses
            .iter()
            .position(|&loss| loss == min_max_mi_loss);
        let min_loss_idx = match min_loss_idx {
            Some(idx) => idx,
            None => return Err("Failed finding minimum mutual information loss!"),
        };
        let optimal_wl_idx = tested_wl_idxs[min_loss_idx];
        let optimal_threshold = weight_levels[optimal_wl_idx];

        Ok(optimal_threshold)
    }
    /// Returns the number of sequences that can be encoded / decoded
    pub fn num_sequences(&self) -> Integer {
        self.trellis.get(0, 0)
    }
    /// Returns the number of bits that can be encoded / decoded
    pub fn num_bits(&self) -> u32 {
        self.num_sequences().significant_bits() - 1
    }
    /// Returns the weights used by the internal trellis
    pub fn get_weights(&self) -> Vec<usize> {
        self.trellis.get_weights()
    }
    /// Returns the distribution [AdEss] is optimizing for
    pub fn get_distribution(&self, res_factor: f32) -> Vec<f32> {
        let exps: Vec<f32> = self
            .get_weights()
            .iter()
            .map(|weight| (*weight as f32 / -res_factor).exp2())
            .collect();
        let exps_sum = exps.iter().sum::<f32>();
        let p_goal: Vec<f32> = exps.iter().map(|exp| exp / exps_sum).collect();

        p_goal
    }
    /// Returns the reverse trellis for this [AdEss]
    pub fn reverse_trellis(&self) -> Trellis {
        AdEss::calc_reverse_trellis(
            self.trellis.threshold,
            self.trellis.n_max,
            &self.trellis.get_weights(),
        )
    }
    /// Returns the amplitude sequence for a given `index` (encode)
    ///
    /// Calculations based on algorithm 1 in section III-C of <https://doi.org/10.1109/TWC.2019.2951139>.
    pub fn sequence_for_index(&self, index: &Integer) -> Vec<usize> {
        assert!(index < &self.num_sequences(), "Index out of range!");

        let mut amplitude_sequence = Vec::new();

        let mut current_wl = 0;
        let mut num_sequences_left_below = Integer::from(0);
        for n in 0..self.trellis.n_max {
            for (w_idx, next_wl) in self.trellis.get_successors(current_wl) {
                let next_wl_value = self.trellis.get(n + 1, next_wl);

                // it is impossible to leave all sequences possible with `next_wl` below
                // when using `next_wl` as the next weight level
                let just_unreachable_index =
                    Integer::from(&num_sequences_left_below + &next_wl_value);

                if index < &just_unreachable_index {
                    // we can reach the target index via next_wl
                    amplitude_sequence.push(AdEss::weight_idx_to_amplitude(w_idx));
                    current_wl = next_wl;
                    break;
                } else {
                    // target index is not reachable if we use `next_wl` as next weight level
                    // -> to reach the target index, we have to use a higher next weight level
                    // thus we leave below all sequences possible with the current `next_wl`
                    num_sequences_left_below += next_wl_value
                }
            }
        }
        amplitude_sequence
    }
    /// Returns the index for a given `amplitude_sequence` (decode)
    ///
    /// Calculations based on algorithm 2 in section III-C of <https://doi.org/10.1109/TWC.2019.2951139>.
    pub fn index_for_sequence(&self, amplitude_sequence: &[usize]) -> Integer {
        let weight_idx_seq = AdEss::amplitude_seq_to_weight_idx_seq(amplitude_sequence);
        let weights = self.trellis.get_weights();

        // the index of the sequence, before the number of lower sequences is added
        let mut index = Integer::from(0);

        // compute the sequence of traversed weight levels
        let wl_seq = weight_idx_seq.iter().fold(vec![0], |mut acc, w_idx| {
            acc.push(weights[*w_idx] + acc[acc.len() - 1]);
            acc
        });

        // add number of lower sequences to the index
        for n in 0..self.trellis.n_max {
            // sum number of possible sequences where the next weight would have lower order than
            // the real next weight
            for (w_idx, next_wl) in self.trellis.get_successors(wl_seq[n]) {
                if next_wl <= wl_seq[n + 1] && w_idx != weight_idx_seq[n] {
                    index += self.trellis.get(n + 1, next_wl);
                } else {
                    break;
                }
            }
        }
        index
    }
    /// Counts the occurences of the amplitude associated to `weight_idx` in stage `stage`
    fn count_weight_in_stage(&self, weight_idx: usize, stage: usize) -> Integer {
        let num_bits = self.num_bits();
        let num_sequences_used = Integer::u_pow_u(2, num_bits).complete();
        let first_abandoned_sequence = self.sequence_for_index(&num_sequences_used); // Short: FAS
        let weights = self.trellis.get_weights();
        let fas_weight_idxs: Vec<usize> = first_abandoned_sequence // FAS weight indexes
            .iter()
            .map(|a| (a - 1) / 2) // amplitude -> weight index
            .collect();
        let fas_weights: Vec<usize> = fas_weight_idxs
            .iter()
            .map(|&w_idx| weights[w_idx])
            .collect();
        let fas_wls = cumsum(&fas_weights); // FAS weight levels

        let n_max = self.trellis.n_max;

        // occurences in sequences that split out of the FAS at earlier stages
        let from_earlier_splits: Integer = if stage > 0 {
            (0..stage)
                .map(|n| {
                    self.trellis
                        .get_successors(fas_wls[n])
                        .iter()
                        .take_while(|(w_idx, _)| w_idx != &fas_weight_idxs[n])
                        .map(|(_, wl)| self.trellis.get_or_0(n + 2, *wl + weights[weight_idx]))
                        .sum::<Integer>()
                })
                .sum()
        } else {
            Integer::from(0)
        };

        // occurences in sequences that split out of the FAS at this stage
        let from_split_at_stage = if weights[weight_idx] < fas_weights[stage] ||
        // this condition depends on the internal ordering used by [Trellis::get_predecessor]
        // if code is changed there this code might break
        (weights[weight_idx] == fas_weights[stage] && weight_idx < fas_weight_idxs[stage])
        {
            self.trellis
                .get_or_0(stage + 1, fas_wls[stage] + weights[weight_idx])
        } else {
            Integer::from(0)
        };

        // occurences in sequences that split out of the FAS at later stages
        let from_later_splits = if weight_idx == fas_weight_idxs[stage] {
            (stage + 1..n_max)
                .map(|n| {
                    self.trellis
                        .get_successors(fas_wls[n])
                        .iter()
                        .take_while(|(w_idx, _)| w_idx != &fas_weight_idxs[n])
                        .map(|(_, wl)| self.trellis.get_or_0(n + 1, *wl))
                        .sum::<Integer>()
                })
                .sum()
        } else {
            Integer::from(0)
        };

        from_earlier_splits + from_split_at_stage + from_later_splits
    }
    /// Returns the amplitude distribution as a [Vec]
    ///
    /// The amplitude distribution is valid if only sequences with indexes
    /// representable with [self.num_bits] bits are used.
    pub fn amplitude_distribution(&self) -> Vec<f32> {
        let num_bits = self.num_bits();
        let num_sequences_used = Integer::u_pow_u(2, num_bits).complete();

        if num_sequences_used == self.num_sequences() {
            return self.amplitude_distribution_full_utilization();
        }

        let n_max = self.trellis.n_max;

        let weight_frequencies: Vec<f32> = (0..self.trellis.get_weights().len())
            .map(|weight_idx| {
                (0..n_max)
                    .map(|stage| self.count_weight_in_stage(weight_idx, stage))
                    .sum::<Integer>() // sum occurences over all stages
            })
            .map(|weight_occurences| {
                // convert number of occurences to relative frequency
                Rational::from((weight_occurences, &num_sequences_used * n_max)).to_f32()
            })
            .collect();

        weight_frequencies
    }
    /// Returns the amplitude distribution as a [Vec]
    ///
    /// The amplitude distribution is valid if all sequences in the trellis
    /// are used equiprobably.
    pub fn amplitude_distribution_full_utilization(&self) -> Vec<f32> {
        let num_sequences = self.num_sequences();
        let mut distribution = vec![0f32; self.trellis.get_weights().len()];

        for (w_idx, wl) in self.trellis.get_successors(0) {
            distribution[w_idx] =
                Rational::from((self.trellis.get(1, wl), &num_sequences)).to_f32();
        }
        distribution
    }

    /// Returns the average energy
    ///
    /// Assumes only indexes representable with [self.num_bits] bits are used.
    pub fn average_energy(&self) -> f32 {
        let amplitude_distribution = self.amplitude_distribution();
        amplitude_distribution
            .iter()
            .enumerate()
            .map(|(w_idx, p)| (AdEss::weight_idx_to_amplitude(w_idx) as f32, p))
            .map(|(a, p)| a * a * p) // expected value of energy == squared amplitude * probability
            .sum::<f32>()
    }
}
