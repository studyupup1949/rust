use rug::Integer;

use ad_ess::ad_ess::AdEss;
use ad_ess::trellis_utils;
use ad_ess::utils::{entropy, information, kl_divergence};

fn main() {
    let reverse_trellis =
        trellis_utils::reverse_trellis_upto_num_sequences(Integer::from(65), 4, &[0, 1, 3, 6])
            .unwrap();
    trellis_utils::pprint_trellis(&reverse_trellis);

    let original_distribution = vec![0.28, 0.3, 0.2, 0.22];
    println!("{original_distribution:?}");
    let n_max = 95;
    let factor = 10.0;

    let (adess, _) =
        AdEss::new_for_distribution_num_bits(40, n_max, &original_distribution, factor).unwrap();
    println!("Num bits: {}", adess.num_bits());

    let optimal_threshold =
        AdEss::optimal_threshold(n_max, &original_distribution, factor, 10, 0.5).unwrap();
    println!("optimal threshold: {optimal_threshold}");

    profile_adess(optimal_threshold, n_max, &original_distribution, factor);
}

fn profile_adess(threshold: usize, n_max: usize, original_distribution: &Vec<f32>, factor: f32) {
    println!();
    println!("##########################################");
    println!("Profile AD_ESS");
    println!("##########################################");
    println!();
    println!("Threshold: {threshold}");
    println!();

    let (adess, distribution) =
        AdEss::new_for_distribution_threshold(threshold, n_max, original_distribution, factor)
            .unwrap();

    println!("Goal distribution: {distribution:?}");
    println!("  Information: {:?} bit", information(&distribution));
    println!("  Trellis weights: {:?}", adess.trellis.get_weights());
    println!("  Entropy: {} bit", entropy(&distribution));
    println!();

    println!(
        "Amplitude distribution: {:?}",
        adess.amplitude_distribution()
    );
    println!(
        "  Information: {:?} bit",
        information(&adess.amplitude_distribution())
    );
    println!(
        "  Entropy: {} bit",
        entropy(&adess.amplitude_distribution())
    );
    println!();
    println!(
        "KL-divergence (goal to final):     {}",
        kl_divergence(&distribution, &adess.amplitude_distribution())
    );
    println!(
        "KL-divergence (original to final): {}",
        kl_divergence(original_distribution, &adess.amplitude_distribution())
    );
    println!();
    println!(
        "Storage complexity: {:?}",
        adess.trellis.get_storage_dimensions()
    );
    println!(
        "Num sequences: 2^{:?}",
        adess.num_sequences().to_f64().log2()
    );
    println!(
        "Shaping rate: {} bit/amplitude",
        adess.num_bits() as f32 / n_max as f32
    );
    println!();

    println!(
        "Mutual information loss: {} bit/channel use",
        entropy(original_distribution) - adess.num_bits() as f32 / n_max as f32
            + kl_divergence(&adess.amplitude_distribution(), original_distribution)
    );
}
