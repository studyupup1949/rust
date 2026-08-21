use rug::rand::RandState;
use rug::{Complete, Integer};

use crate::ad_ess::AdEss;
use crate::trellis::Trellis;

use crate::rts::RTS;

use crate::trellis_utils;
use crate::utils;

#[test]
fn incremental_reverse_trellis_vs_traditional() {
    let weights = vec![2, 0, 5, 0, 2];
    let n_max = 5;
    let threshold = 11;
    let adess = AdEss::new(threshold, n_max, &weights);
    let num_bits = adess.num_bits();

    let traditional_reverse_trellis = adess.reverse_trellis();
    println!("Reverse trellis via traditional calculation");
    trellis_utils::pprint_trellis(&traditional_reverse_trellis);

    println!();
    println!("Reverse trellis via incremental calculation");
    let reverse_trellis = trellis_utils::reverse_trellis_upto_num_sequences(
        Integer::u_pow_u(2, num_bits as u32).complete(),
        n_max,
        &adess.trellis.get_weights(),
    )
    .unwrap();
    trellis_utils::pprint_trellis(&reverse_trellis);

    println!(
        "Traditional reverse trellis num sequences: {}",
        traditional_reverse_trellis
            .get_stage(n_max)
            .iter()
            .sum::<Integer>()
    );
    println!(
        "Incremental reverse trellis num sequences: {}",
        reverse_trellis.get_stage(n_max).iter().sum::<Integer>()
    );

    println!();
    for s in 0..n_max + 1 {
        for (idx, (auto, inc)) in traditional_reverse_trellis
            .get_stage(s)
            .iter()
            .zip(&reverse_trellis.get_stage(s))
            .enumerate()
        {
            if auto != inc {
                println!("stage: {}, idx:{}, auto: {}, inc: {}", s, idx, auto, inc);
                panic!("The two reverse trellises differ");
            }
        }
    }
}

#[test]
fn reverse_trellis_lexicographically_bounded() {
    let rt = trellis_utils::reverse_trellis_lexicographically_bounded(
        7,
        4,
        &[0, 1, 3, 6],
        &[5, 1, 3, 1],
    );
    trellis_utils::pprint_trellis(&rt);
    println!();
    let mut paper_example = Trellis::new(7, 4, &[0, 1, 3, 6]);
    paper_example.set(1, 0, Integer::from(1));
    paper_example.set(1, 1, Integer::from(1));
    paper_example.set(2, 0, Integer::from(1));
    paper_example.set(2, 1, Integer::from(2));
    paper_example.set(2, 2, Integer::from(1));
    paper_example.set(2, 3, Integer::from(1));
    paper_example.set(2, 4, Integer::from(1));
    paper_example.set(2, 6, Integer::from(1));
    paper_example.set(2, 7, Integer::from(1));
    paper_example.set(3, 0, Integer::from(1));
    paper_example.set(3, 1, Integer::from(3));
    paper_example.set(3, 2, Integer::from(3));
    paper_example.set(3, 3, Integer::from(4));
    paper_example.set(3, 4, Integer::from(4));
    paper_example.set(3, 5, Integer::from(2));
    paper_example.set(3, 6, Integer::from(3));
    paper_example.set(3, 7, Integer::from(5));
    paper_example.set(4, 0, Integer::from(1));
    paper_example.set(4, 1, Integer::from(4));
    paper_example.set(4, 2, Integer::from(6));
    paper_example.set(4, 3, Integer::from(8));
    paper_example.set(4, 4, Integer::from(11));
    paper_example.set(4, 5, Integer::from(9));
    paper_example.set(4, 6, Integer::from(10));
    paper_example.set(4, 7, Integer::from(15));
    trellis_utils::pprint_trellis(&paper_example);
    println!();

    assert_eq!(rt, paper_example);
}

#[test]
fn amplitude_distribution_paper_example() {
    let adess = AdEss::new(7, 4, &vec![0, 1, 3, 6]);
    let amp_dist = adess.amplitude_distribution();
    let amp_frequencies: Vec<f32> = amp_dist
        .iter()
        .map(|p| {
            p * Integer::u_pow_u(2, adess.num_bits()).complete().to_f32()
                * adess.trellis.n_max as f32
        })
        .collect();
    println!("{:?}", amp_dist);
    println!("{:?}", amp_frequencies);

    assert_eq!(amp_frequencies, vec![114.0, 84.0, 46.0, 12.0]);
}

#[test]
fn average_energy_paper_example() {
    let adess = AdEss::new(7, 4, &vec![0, 1, 3, 6]);
    let mut e_acc = 0;
    let num_sequences_used = 2_i32.pow(adess.num_bits());
    for idx in 0..num_sequences_used {
        let seq = adess.sequence_for_index(&Integer::from(idx));
        let seq_energy: usize = seq.iter().map(|a| a * a).sum();
        e_acc += seq_energy;
    }
    let e_avg = e_acc as f32 / num_sequences_used as f32 / adess.trellis.n_max as f32;
    assert_eq!(adess.average_energy(), e_avg);
}

#[test]
fn average_energy_montecarlo() {
    let n_max = 224;
    let (adess, _) = AdEss::new_for_distribution_num_bits(
        336,
        224,
        &[0.3229397, 0.14510616, 0.02929643, 0.00265771],
        10.0,
    )
    .unwrap();

    let e_avg = adess.average_energy();
    println!("Calculated e_avg: {}", e_avg);

    let mut rand = RandState::new();
    let num_sequences = Integer::u_pow_u(2, adess.num_bits()).complete();
    let mut e_acc = 0f64;
    let montecarlo_n = 1000;
    for _ in 0..montecarlo_n {
        let random_index = Integer::from(num_sequences.random_below_ref(&mut rand));
        let seq = adess.sequence_for_index(&random_index);
        let seq_energy: usize = seq.iter().map(|a| a * a).sum();
        e_acc += seq_energy as f64;
    }
    let e_avg_montecarlo = e_acc / montecarlo_n as f64 / n_max as f64;
    println!("Montecarlo estimated e_avg: {}", e_avg_montecarlo);

    let tolerance = 0.001;
    let upper = e_avg as f64 * (1.0 + tolerance);
    let lower = e_avg as f64 * (1.0 - tolerance);
    assert!(lower < e_avg_montecarlo && e_avg_montecarlo < upper);
}

#[test]
fn cumsum_static() {
    let a = utils::cumsum(&[1, 1, 2, 3]);
    println!("{:?}", a);
    assert_eq!(a, vec![0, 1, 2, 4, 7]);

    let a = utils::cumsum(&[1.0, 1.1, 2.0, 3.0]);
    println!("{:?}", a);
    assert_eq!(a, vec![0.0, 1.0, 2.1, 4.1, 7.1]);
}

#[test]
fn differentiate_static() {
    let a = utils::differeniate(&[1, 1, 2, 3]);
    println!("{:?}", a);
    assert_eq!(a, vec![0, 1, 1]);

    let a = utils::differeniate(&[1.0, 1.1, 2.0, 3.0]);
    println!("{:?}", a);
    assert_eq!(a, vec![1.1 - 1.0, 2.0 - 1.1, 3.0 - 2.0]);
}

#[test]
fn rts_toy_example() {
    let rts = RTS::new(4, 4, &vec![0, 1, 3, 6]);
    let example_sequences = vec![
        vec![1, 1, 1, 1],
        vec![1, 1, 1, 3],
        vec![1, 1, 3, 1],
        vec![1, 3, 1, 1],
        vec![3, 1, 1, 1],
        vec![1, 1, 3, 3],
        vec![1, 3, 1, 3],
        vec![3, 1, 1, 3],
        vec![1, 3, 3, 1],
        vec![3, 1, 3, 1],
        vec![3, 3, 1, 1],
        vec![1, 1, 1, 5],
        vec![1, 3, 3, 3],
        vec![3, 1, 3, 3],
        vec![3, 3, 1, 3],
        vec![1, 1, 5, 1],
        vec![3, 3, 3, 1],
        vec![1, 5, 1, 1],
        vec![5, 1, 1, 1],
    ];
    for (idx, seq) in example_sequences.iter().enumerate() {
        let seq_calc = rts.sequence_for_index(&Integer::from(idx));
        let idx_calc = rts.index_for_sequence(seq);
        println!("{} -> {:?} | {:?} -> {}", idx, seq_calc, seq, idx_calc);
        assert_eq!(seq_calc, *seq);
        assert_eq!(idx_calc, idx);
    }
}

#[test]
fn rts_non_unique_weights_toy_example() {
    let example_sequences = vec![
        vec![1, 1, 1],
        vec![1, 1, 5],
        vec![1, 1, 3],
        vec![1, 5, 1],
        vec![1, 3, 1],
        vec![5, 1, 1],
        vec![3, 1, 1],
        vec![1, 1, 7],
        vec![1, 5, 5],
        vec![1, 3, 5],
        vec![5, 1, 5],
        vec![3, 1, 5],
        vec![1, 5, 3],
        vec![1, 3, 3],
        vec![5, 1, 3],
        vec![3, 1, 3],
        vec![1, 7, 1],
        vec![5, 5, 1],
        vec![3, 5, 1],
        vec![5, 3, 1],
        vec![3, 3, 1],
        vec![7, 1, 1],
    ];
    let rts = RTS::new(5, 3, &vec![0, 1, 1, 2]);
    for (idx, seq) in example_sequences.iter().enumerate() {
        let seq_calc = rts.sequence_for_index(&Integer::from(idx));
        let idx_calc = rts.index_for_sequence(seq);
        println!("{} -> {:?} | {:?} -> {}", idx, seq_calc, seq, idx_calc);
        assert_eq!(seq_calc, *seq);
        assert_eq!(idx_calc, idx);
    }
}

#[test]
fn rts_non_unique_unordered_weights() {
    let rts = RTS::new(7, 4, &vec![2, 0, 5, 2]);
    let num_seq = rts.num_sequences().to_usize().unwrap();
    for idx in 0..num_seq {
        let seq_calc = rts.sequence_for_index(&Integer::from(idx));
        let idx_calc = rts.index_for_sequence(&seq_calc);
        println!("{} -> {:?} -> {}", idx, seq_calc, idx_calc);
        assert_eq!(idx_calc, idx);
    }
}

#[test]
fn rts_multiple_non_unique_weights() {
    let rts = RTS::new(10, 4, &vec![0, 0, 1, 1, 1, 2, 3]);
    trellis_utils::pprint_trellis(&rts.trellis);
    let num_seqences = rts.num_sequences().to_u32().unwrap();
    for idx in 0..num_seqences {
        let seq = rts.sequence_for_index(&Integer::from(idx));
        let idx_calc = rts.index_for_sequence(&seq);
        assert_eq!(idx, idx_calc.to_u32().unwrap());
    }
}

#[test]
fn rts_amplitude_distribution_full_utilization_toy_example() {
    let rts = RTS::new(8, 5, &vec![0, 1, 1, 3]);
    let num_seqences = rts.num_sequences().to_u32().unwrap();
    let n_max = rts.trellis.n_max;
    trellis_utils::pprint_trellis(&rts.trellis);

    let mut num_occurences = vec![0; 4];
    for idx in 0..num_seqences {
        let seq = rts.sequence_for_index(&Integer::from(idx));
        for amplitude in seq {
            num_occurences[(amplitude - 1) / 2] += 1;
        }
    }

    for (p, num) in rts
        .amplitude_distribution_full_utilization()
        .iter()
        .zip(num_occurences)
    {
        let num_from_distribution = p * num_seqences as f32 * n_max as f32;
        println!(
            "calc via amplitude distribution: {}, true: {}",
            num_from_distribution, num
        );
        assert_eq!(num_from_distribution as u32, num)
    }
}

#[test]
fn rts_amplitude_distribution() {
    let rts_list = vec![
        RTS::new(7, 4, &vec![2, 0, 2, 5]),
        RTS::new(2, 3, &vec![2, 0, 2, 5]),
        RTS::new(8, 5, &vec![2, 0, 2, 5]),
        RTS::new(8, 5, &vec![0, 1, 1, 3]),
    ];

    for rts in rts_list {
        let num_sequences = rts.num_sequences();
        let num_used_sequences = Integer::u_pow_u(2, rts.num_bits())
            .complete()
            .to_usize()
            .unwrap();

        let mut amplitude_counts = vec![0; 4];

        for i in 0..num_used_sequences {
            let seq = rts.sequence_for_index(&Integer::from(i));
            for amplitude in seq {
                let amp_idx = (amplitude - 1) / 2;
                amplitude_counts[amp_idx] += 1;
            }
        }

        let amp_dist = rts.amplitude_distribution();
        let rts_amp_counts: Vec<usize> = amp_dist
            .iter()
            .map(|p| (p * num_used_sequences as f32 * rts.trellis.n_max as f32).round() as usize)
            .collect();
        let dist_sum = amp_dist.iter().sum::<f32>();

        println!();
        println!("rts.num_sequences:  {}", num_sequences);
        println!("num_used_sequences: {}", num_used_sequences);
        println!("rts.amplitude_distribution: {:?}", amp_dist);
        println!("sum of amplidude distribution: {}", dist_sum);
        println!(
            "Counts via rts.amplitude_distribution: {:?}",
            rts_amp_counts
        );
        println!(
            "Calculation via loop:                  {:?}",
            amplitude_counts
        );

        assert_eq!(dist_sum, 1.0, "sum of distribution does not equal 1");
        assert_eq!(
            rts_amp_counts, amplitude_counts,
            "rts.amplitude_distribution and loop counting differ"
        );
    }
}

#[test]
fn adess_encoding_decoding() {
    let adess_list = vec![
        AdEss::new(9, 4, &vec![2, 0, 2, 5]),
        AdEss::new(8, 5, &vec![2, 0, 2, 5]),
        AdEss::new(6, 3, &vec![5, 0, 2, 0]),
        AdEss::new(4, 5, &vec![0, 1, 1, 3]),
        AdEss::new(30, 5, &vec![0, 1, 3, 6]),
    ];
    for adess in adess_list {
        let num_seq = adess.num_sequences().to_u32().unwrap();
        for i in 0..num_seq {
            let seq = adess.sequence_for_index(&Integer::from(i));
            let decoded_i = adess.index_for_sequence(&seq).to_u32().unwrap();
            assert_eq!(i, decoded_i);
        }
    }
}

#[test]
fn adess_amplitude_distribution_full_utilization() {
    let adess_list = vec![
        AdEss::new(9, 4, &vec![2, 0, 2, 5]),
        AdEss::new(8, 5, &vec![2, 0, 2, 5]),
        AdEss::new(6, 3, &vec![5, 0, 2, 0]),
        AdEss::new(4, 5, &vec![0, 1, 1, 3]),
        AdEss::new(30, 5, &vec![0, 1, 3, 6]),
    ];
    for adess in adess_list {
        let num_seq = adess.num_sequences().to_usize().unwrap();
        let mut a_counts = vec![0, 0, 0, 0];
        for i in 0..num_seq {
            let seq = adess.sequence_for_index(&Integer::from(i));
            for a in seq {
                let a_idx = (a - 1) / 2;
                a_counts[a_idx] += 1;
            }
        }
        let num_amplitudes = num_seq * adess.trellis.n_max;
        adess
            .amplitude_distribution_full_utilization()
            .iter()
            .map(|p| p * num_amplitudes as f32)
            .zip(a_counts)
            .for_each(|(from_method_call, from_for_loop)| {
                assert_eq!(from_method_call.round() as usize, from_for_loop);
            });
    }
}

#[test]
fn adess_amplitude_distribution() {
    let adess_list = vec![
        AdEss::new(9, 4, &vec![2, 0, 2, 5]),
        AdEss::new(8, 5, &vec![2, 0, 2, 5]), // full utilization
        AdEss::new(6, 3, &vec![5, 0, 2, 0]),
        AdEss::new(4, 5, &vec![0, 1, 1, 3]),  // full utilization
        AdEss::new(30, 5, &vec![0, 1, 3, 6]), // ESS
    ];
    for adess in adess_list {
        let num_bits = adess.num_bits();
        let num_seq = 2usize.pow(num_bits);

        let mut a_counts = vec![0, 0, 0, 0];
        for i in 0..num_seq {
            let seq = adess.sequence_for_index(&Integer::from(i));
            for a in seq {
                let a_idx = (a - 1) / 2;
                a_counts[a_idx] += 1;
            }
        }

        let num_amplitudes = num_seq * adess.trellis.n_max;
        adess
            .amplitude_distribution()
            .iter()
            .map(|p| p * num_amplitudes as f32)
            .zip(a_counts)
            .for_each(|(from_method_call, from_for_loop)| {
                assert_eq!(from_method_call.round() as usize, from_for_loop);
            });
    }
}

#[test]
fn adess_average_energy() {
    let adess_list = vec![
        AdEss::new(9, 4, &vec![2, 0, 2, 5]),
        AdEss::new(8, 5, &vec![2, 0, 2, 5]), // full utilization
        AdEss::new(6, 3, &vec![5, 0, 2, 0]),
        AdEss::new(4, 5, &vec![0, 1, 1, 3]),  // full utilization
        AdEss::new(30, 5, &vec![0, 1, 3, 6]), // ESS
    ];
    for adess in adess_list {
        let num_bits = adess.num_bits();
        let num_seq = 2usize.pow(num_bits);

        let mut energy = 0;
        for i in 0..num_seq {
            let seq = adess.sequence_for_index(&Integer::from(i));
            for a in seq {
                energy += a * a;
            }
        }

        let num_amplitudes = (num_seq * adess.trellis.n_max) as f32;
        let avg_energy = adess.average_energy();

        assert_eq!(energy, (num_amplitudes * avg_energy).round() as usize)
    }
}
