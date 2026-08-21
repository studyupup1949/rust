use crate::error::AcoustixError;
use num_complex::Complex;
use rustfft::FftPlanner;
use std::f32::consts::PI;

/// Computes the Segmental Signal-to-Noise Ratio (SegSNR) in dB between a reference signal and a test signal.
///
/// Excludes silent frames where the reference signal energy falls below `silence_threshold`.
/// Individual frame SNRs are clipped between `min_snr` (default -10.0) and `max_snr` (default 35.0).
pub fn segmental_snr(
    ref_sig: &[f32],
    test_sig: &[f32],
    frame_len: usize,
    overlap: usize,
    min_snr: f32,
    max_snr: f32,
    silence_threshold: f32,
) -> Result<f32, AcoustixError> {
    if ref_sig.is_empty() || test_sig.is_empty() {
        return Err(AcoustixError::EmptySignal(
            "Input signals cannot be empty".to_string(),
        ));
    }
    if frame_len == 0 || overlap >= frame_len {
        return Err(AcoustixError::InvalidFraming { frame_len, overlap });
    }

    let len = ref_sig.len().min(test_sig.len());
    let step = frame_len - overlap;

    let mut snr_sum = 0.0;
    let mut active_frames = 0;

    let mut start = 0;
    while start + frame_len <= len {
        let end = start + frame_len;
        let ref_frame = &ref_sig[start..end];
        let test_frame = &test_sig[start..end];

        // Compute signal energy (mean square)
        let ref_energy: f32 = ref_frame.iter().map(|&x| x * x).sum::<f32>() / frame_len as f32;

        // Only calculate SNR if the reference frame is active (non-silent)
        if ref_energy >= silence_threshold {
            let noise_energy: f32 = ref_frame
                .iter()
                .zip(test_frame.iter())
                .map(|(&x, &y)| {
                    let diff = x - y;
                    diff * diff
                })
                .sum::<f32>()
                / frame_len as f32;

            let frame_snr = if noise_energy <= 1e-10 {
                max_snr
            } else {
                let snr = 10.0 * (ref_energy / noise_energy).log10();
                snr.clamp(min_snr, max_snr)
            };

            snr_sum += frame_snr;
            active_frames += 1;
        }

        start += step;
    }

    if active_frames == 0 {
        Ok(min_snr) // If all frames are silent, return minimum SNR
    } else {
        Ok(snr_sum / active_frames as f32)
    }
}

/// Generates a Hamming window of a given length.
fn hamming_window(len: usize) -> Vec<f32> {
    if len <= 1 {
        return vec![1.0];
    }
    (0..len)
        .map(|i| 0.54 - 0.46 * (2.0 * PI * i as f32 / (len - 1) as f32).cos())
        .collect()
}

/// Computes the Log Spectral Distance (LSD) in dB between a reference signal and a test signal.
pub fn log_spectral_distance(
    ref_sig: &[f32],
    test_sig: &[f32],
    frame_len: usize,
    overlap: usize,
) -> Result<f32, AcoustixError> {
    if ref_sig.is_empty() || test_sig.is_empty() {
        return Err(AcoustixError::EmptySignal(
            "Input signals cannot be empty".to_string(),
        ));
    }
    if frame_len == 0 || overlap >= frame_len {
        return Err(AcoustixError::InvalidFraming { frame_len, overlap });
    }

    let len = ref_sig.len().min(test_sig.len());
    let step = frame_len - overlap;

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(frame_len);

    let window = hamming_window(frame_len);
    let mut lsd_sum = 0.0;
    let mut total_frames = 0;

    // The number of unique frequency bins up to Nyquist frequency is frame_len / 2 + 1
    let num_bins = frame_len / 2 + 1;
    let eps = 1e-10_f32;

    let mut start = 0;
    while start + frame_len <= len {
        let end = start + frame_len;

        let mut ref_buf: Vec<Complex<f32>> = ref_sig[start..end]
            .iter()
            .zip(window.iter())
            .map(|(&x, &w)| Complex::new(x * w, 0.0))
            .collect();

        let mut test_buf: Vec<Complex<f32>> = test_sig[start..end]
            .iter()
            .zip(window.iter())
            .map(|(&y, &w)| Complex::new(y * w, 0.0))
            .collect();

        fft.process(&mut ref_buf);
        fft.process(&mut test_buf);

        let mut bin_diff_sq_sum = 0.0;
        for k in 0..num_bins {
            let ref_psd = ref_buf[k].norm_sqr();
            let test_psd = test_buf[k].norm_sqr();

            let ref_log = 10.0 * (ref_psd + eps).log10();
            let test_log = 10.0 * (test_psd + eps).log10();

            let diff = ref_log - test_log;
            bin_diff_sq_sum += diff * diff;
        }

        let frame_lsd = (bin_diff_sq_sum / num_bins as f32).sqrt();
        lsd_sum += frame_lsd;
        total_frames += 1;

        start += step;
    }

    if total_frames == 0 {
        Ok(0.0)
    } else {
        Ok(lsd_sum / total_frames as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmental_snr_identical() {
        let sig = vec![0.1, 0.2, -0.3, 0.4, -0.5, 0.6, -0.7, 0.8];
        let res = segmental_snr(&sig, &sig, 4, 2, -10.0, 35.0, 1e-4).unwrap();
        assert!((res - 35.0).abs() < 1e-5);
    }

    #[test]
    fn test_segmental_snr_noise() {
        let ref_sig = vec![0.5, 0.5, 0.5, 0.5];
        // Test sig has noise added
        let test_sig = vec![0.4, 0.6, 0.4, 0.6]; // Diff is 0.1, 0.1, 0.1, 0.1
        let res = segmental_snr(&ref_sig, &test_sig, 4, 0, -10.0, 35.0, 1e-4).unwrap();
        // ref energy = 0.25, noise energy = 0.01. SNR = 10 * log10(25) = 13.9794
        assert!((res - 13.9794).abs() < 1e-3);
    }

    #[test]
    fn test_lsd_identical() {
        let sig = vec![0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8];
        let res = log_spectral_distance(&sig, &sig, 8, 4).unwrap();
        assert!(res.abs() < 1e-5);
    }

    #[test]
    fn test_lsd_difference() {
        let ref_sig = vec![1.0; 8];
        let test_sig = vec![2.0; 8];
        let res = log_spectral_distance(&ref_sig, &test_sig, 8, 0).unwrap();
        // Constant signals will have their DC component scale by 2.
        // Ratio of power spectra is 1/4. However, due to windowing, the 5th frequency bin (k=4)
        // has mathematically zero energy (or below eps = 1e-10).
        // Therefore, only 4 of the 5 bins have a log ratio of -6.02 dB, while the k=4 bin
        // ratio is pulled to 1.0 (0.0 dB difference) by eps.
        // The expected LSD is therefore sqrt(4/5) * 6.020599 = 5.384989 dB.
        assert!(
            (res - 5.384989).abs() < 1e-3,
            "Actual LSD computed was: {}",
            res
        );
    }
}
