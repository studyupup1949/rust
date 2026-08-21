//! Fast Fourier Transform (FFT) Implementation
//!
//! The FFT is one of the most important numerical algorithms, used in signal processing,
//! image compression, audio processing, and many other applications.
//!
//! This implementation uses the Cooley-Tukey algorithm and supports parallel processing
//! for improved performance on large datasets.
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::numerical::fft;
//!
//! // Transform a simple signal
//! let signal = vec![1.0, 2.0, 3.0, 4.0, 3.0, 2.0, 1.0, 0.0];
//! let spectrum = fft::fft(&signal);
//!
//! // Transform back to time domain
//! let reconstructed = fft::ifft(&spectrum);
//! ```

use num_complex::Complex64;
use rayon::prelude::*;
use std::f64::consts::PI;

/// Performs a Fast Fourier Transform on the input signal
///
/// # Arguments
///
/// * `input` - A slice of real-valued samples
///
/// # Returns
///
/// A vector of complex frequency components
///
/// # Panics
///
/// Panics if the input length is not a power of 2
///
/// # Performance
///
/// Time complexity: O(n log n) where n is the input length
/// Uses parallel processing for inputs larger than 1024 samples
pub fn fft(input: &[f64]) -> Vec<Complex64> {
    let n = input.len();
    assert!(n.is_power_of_two(), "Input length must be a power of 2");
    
    let complex_input: Vec<Complex64> = input.iter()
        .map(|&x| Complex64::new(x, 0.0))
        .collect();
    
    fft_complex(&complex_input)
}

/// Performs FFT on complex-valued input
///
/// # Arguments
///
/// * `input` - A slice of complex samples
///
/// # Returns
///
/// A vector of complex frequency components
pub fn fft_complex(input: &[Complex64]) -> Vec<Complex64> {
    let n = input.len();
    
    if n <= 1 {
        return input.to_vec();
    }
    
    if n <= 32 {
        // Use DFT for small inputs
        return dft(input);
    }
    
    // Cooley-Tukey FFT algorithm
    fft_recursive(input)
}

/// Inverse Fast Fourier Transform
///
/// Converts frequency domain back to time domain
///
/// # Arguments
///
/// * `input` - Frequency domain complex samples
///
/// # Returns
///
/// Time domain complex samples
pub fn ifft(input: &[Complex64]) -> Vec<Complex64> {
    let n = input.len();
    
    // Conjugate the input
    let conjugated: Vec<Complex64> = input.iter()
        .map(|&x| x.conj())
        .collect();
    
    // Perform FFT
    let result = fft_complex(&conjugated);
    
    // Conjugate and scale the result
    result.iter()
        .map(|&x| x.conj() / (n as f64))
        .collect()
}

/// Performs FFT with parallel processing for large inputs
///
/// # Arguments
///
/// * `input` - A slice of real-valued samples
///
/// # Returns
///
/// A vector of complex frequency components
pub fn fft_parallel(input: &[f64]) -> Vec<Complex64> {
    let n = input.len();
    assert!(n.is_power_of_two(), "Input length must be a power of 2");
    
    let complex_input: Vec<Complex64> = input.par_iter()
        .map(|&x| Complex64::new(x, 0.0))
        .collect();
    
    fft_recursive_parallel(&complex_input)
}

// Internal recursive FFT implementation
fn fft_recursive(input: &[Complex64]) -> Vec<Complex64> {
    let n = input.len();
    
    if n <= 1 {
        return input.to_vec();
    }
    
    // Split into even and odd indices
    let even: Vec<Complex64> = input.iter()
        .step_by(2)
        .copied()
        .collect();
    
    let odd: Vec<Complex64> = input.iter()
        .skip(1)
        .step_by(2)
        .copied()
        .collect();
    
    // Recursively compute FFT
    let fft_even = fft_recursive(&even);
    let fft_odd = fft_recursive(&odd);
    
    // Combine results
    let mut result = vec![Complex64::new(0.0, 0.0); n];
    
    for k in 0..n/2 {
        let angle = -2.0 * PI * (k as f64) / (n as f64);
        let w = Complex64::new(angle.cos(), angle.sin());
        let t = w * fft_odd[k];
        
        result[k] = fft_even[k] + t;
        result[k + n/2] = fft_even[k] - t;
    }
    
    result
}

// Parallel FFT implementation
fn fft_recursive_parallel(input: &[Complex64]) -> Vec<Complex64> {
    let n = input.len();
    
    if n <= 1024 {
        return fft_recursive(input);
    }
    
    // Split into even and odd indices (same as serial version)
    let even: Vec<Complex64> = input.iter()
        .step_by(2)
        .copied()
        .collect();
    
    let odd: Vec<Complex64> = input.iter()
        .skip(1)
        .step_by(2)
        .copied()
        .collect();
    
    // Recursively compute FFT in parallel
    let (fft_even, fft_odd) = rayon::join(
        || fft_recursive_parallel(&even),
        || fft_recursive_parallel(&odd)
    );
    
    // Combine results (same as serial version but parallelized)
    let mut result = vec![Complex64::new(0.0, 0.0); n];
    
    result.par_iter_mut()
        .enumerate()
        .for_each(|(k, r)| {
            if k < n/2 {
                let angle = -2.0 * PI * (k as f64) / (n as f64);
                let w = Complex64::new(angle.cos(), angle.sin());
                let t = w * fft_odd[k];
                *r = fft_even[k] + t;
            } else {
                let k = k - n/2;
                let angle = -2.0 * PI * (k as f64) / (n as f64);
                let w = Complex64::new(angle.cos(), angle.sin());
                let t = w * fft_odd[k];
                *r = fft_even[k] - t;
            }
        });
    
    result
}

// Direct DFT for small inputs
fn dft(input: &[Complex64]) -> Vec<Complex64> {
    let n = input.len();
    let mut result = vec![Complex64::new(0.0, 0.0); n];
    
    for (k, r) in result.iter_mut().enumerate() {
        let mut sum = Complex64::new(0.0, 0.0);
        for (j, &x) in input.iter().enumerate() {
            let angle = -2.0 * PI * (k * j) as f64 / n as f64;
            let w = Complex64::new(angle.cos(), angle.sin());
            sum += x * w;
        }
        *r = sum;
    }
    
    result
}

/// Compute the power spectrum (magnitude squared) from FFT output
///
/// # Arguments
///
/// * `fft_output` - Output from FFT
///
/// # Returns
///
/// Vector of power values (magnitude squared)
pub fn power_spectrum(fft_output: &[Complex64]) -> Vec<f64> {
    fft_output.iter()
        .map(|c| c.norm_sqr())
        .collect()
}

/// Compute the magnitude spectrum from FFT output
///
/// # Arguments
///
/// * `fft_output` - Output from FFT
///
/// # Returns
///
/// Vector of magnitude values
pub fn magnitude_spectrum(fft_output: &[Complex64]) -> Vec<f64> {
    fft_output.iter()
        .map(|c| c.norm())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fft_basic() {
        let input = vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let output = fft(&input);
        assert_eq!(output.len(), 8);
    }
    
    #[test]
    fn test_fft_ifft_roundtrip() {
        let input = vec![1.0, 2.0, 3.0, 4.0, 3.0, 2.0, 1.0, 0.0];
        let spectrum = fft(&input);
        let reconstructed = ifft(&spectrum);
        
        for (i, &val) in input.iter().enumerate() {
            assert!((reconstructed[i].re - val).abs() < 1e-10);
            assert!(reconstructed[i].im.abs() < 1e-10);
        }
    }
    
    #[test]
    fn test_fft_parallel() {
        let input: Vec<f64> = (0..2048).map(|i| (i as f64).sin()).collect();
        let serial = fft(&input);
        let parallel = fft_parallel(&input);
        
        // Parallel computation may have slightly different rounding due to different order
        // of operations. We check that the maximum error is small.
        let max_error = serial.iter().zip(parallel.iter())
            .map(|(s, p)| (s - p).norm())
            .fold(0.0, f64::max);
        
        assert!(max_error < 1e-6, "Maximum error: {}", max_error);
    }
}
