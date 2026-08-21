//! Example demonstrating FFT and signal processing

use advanced_algorithms::numerical::fft;
use std::f64::consts::PI;

fn main() {
    println!("=== FFT Demo ===\n");
    
    // Create a signal: combination of two sine waves
    let sample_rate = 1000.0; // Hz
    let duration = 1.0; // seconds
    let n_samples = (sample_rate * duration) as usize;
    let n_samples_pow2 = n_samples.next_power_of_two();
    
    println!("Generating signal with {} samples...", n_samples_pow2);
    
    // Signal = 50 Hz + 120 Hz
    let freq1 = 50.0;
    let freq2 = 120.0;
    
    let mut signal = Vec::with_capacity(n_samples_pow2);
    for i in 0..n_samples_pow2 {
        let t = i as f64 / sample_rate;
        let value = (2.0 * PI * freq1 * t).sin() + 0.5 * (2.0 * PI * freq2 * t).sin();
        signal.push(value);
    }
    
    // Perform FFT
    println!("Performing FFT...");
    let spectrum = fft::fft(&signal);
    
    // Compute power spectrum
    let power = fft::power_spectrum(&spectrum);
    
    // Find dominant frequencies
    println!("\nFinding dominant frequencies:");
    let mut freq_power: Vec<(f64, f64)> = power.iter()
        .enumerate()
        .take(n_samples_pow2 / 2) // Only positive frequencies
        .map(|(i, &p)| {
            let freq = i as f64 * sample_rate / n_samples_pow2 as f64;
            (freq, p)
        })
        .collect();
    
    freq_power.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    println!("\nTop 5 frequencies:");
    for (i, (freq, power)) in freq_power.iter().take(5).enumerate() {
        println!("  {}. {:.1} Hz - Power: {:.2e}", i + 1, freq, power);
    }
    
    // Inverse FFT to reconstruct signal
    println!("\nPerforming inverse FFT...");
    let reconstructed = fft::ifft(&spectrum);
    
    // Verify reconstruction
    let max_error = signal.iter()
        .zip(reconstructed.iter())
        .map(|(&orig, recon)| (orig - recon.re).abs())
        .fold(0.0f64, |a, b| a.max(b));
    
    println!("Maximum reconstruction error: {:.2e}", max_error);
    
    // Demonstrate parallel FFT for large signals
    println!("\n=== Parallel FFT Demo ===");
    let large_signal: Vec<f64> = (0..16384)
        .map(|i| (2.0 * PI * 100.0 * i as f64 / sample_rate).sin())
        .collect();
    
    println!("Processing {} samples with parallel FFT...", large_signal.len());
    let parallel_spectrum = fft::fft_parallel(&large_signal);
    
    println!("Parallel FFT completed!");
    println!("Spectrum length: {}", parallel_spectrum.len());
}
