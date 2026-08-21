use crate::error::AcoustixError;
use num_complex::Complex;
use rustfft::FftPlanner;
use std::f32::consts::PI;

// =========================================================================
// 1. Matrix and Covariance Math for Fréchet Distance
// =========================================================================

/// Computes the mean vector of a dataset of vectors.
/// Input: slice of vectors of equal length.
fn mean_vector(data: &[Vec<f32>]) -> Result<Vec<f32>, &'static str> {
    if data.is_empty() {
        return Err("Data is empty");
    }
    let dim = data[0].len();
    let mut mean = vec![0.0; dim];
    for row in data {
        if row.len() != dim {
            return Err("Vector dimensions must match");
        }
        for j in 0..dim {
            mean[j] += row[j];
        }
    }
    for j in 0..dim {
        mean[j] /= data.len() as f32;
    }
    Ok(mean)
}

/// Computes the covariance matrix of a dataset of vectors.
fn covariance_matrix(data: &[Vec<f32>], mean: &[f32]) -> Result<Vec<Vec<f32>>, &'static str> {
    if data.is_empty() {
        return Err("Data is empty");
    }
    let dim = mean.len();
    let mut cov = vec![vec![0.0; dim]; dim];

    for row in data {
        for j in 0..dim {
            for k in 0..dim {
                cov[j][k] += (row[j] - mean[j]) * (row[k] - mean[k]);
            }
        }
    }

    // Using population covariance division by N
    let n = data.len() as f32;
    for j in 0..dim {
        for k in 0..dim {
            cov[j][k] /= n;
        }
    }
    Ok(cov)
}

/// Computes the eigenvalues and eigenvectors of a symmetric matrix using the Jacobi rotation algorithm.
/// Returns (eigenvalues, eigenvector_matrix).
fn jacobi_eigen(
    matrix: &[Vec<f32>],
    max_iterations: usize,
) -> Result<(Vec<f32>, Vec<Vec<f32>>), &'static str> {
    let n = matrix.len();
    let mut a = matrix.to_vec();
    let mut v = vec![vec![0.0; n]; n];
    for i in 0..n {
        v[i][i] = 1.0;
    }

    let eps = 1e-6_f32;
    let mut iter = 0;

    loop {
        // Find the largest off-diagonal element
        let mut max_val = 0.0_f32;
        let mut p = 0;
        let mut q = 0;
        for i in 0..n {
            for j in (i + 1)..n {
                let abs_val = a[i][j].abs();
                if abs_val > max_val {
                    max_val = abs_val;
                    p = i;
                    q = j;
                }
            }
        }

        if max_val < eps || iter >= max_iterations {
            break;
        }

        // Compute rotation angle
        let ap_p = a[p][p];
        let aq_q = a[q][q];
        let ap_q = a[p][q];

        let tau = (aq_q - ap_p) / (2.0 * ap_q);
        let t = if tau >= 0.0 {
            1.0 / (tau + (1.0 + tau * tau).sqrt())
        } else {
            -1.0 / (-tau + (1.0 + tau * tau).sqrt())
        };

        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = c * t;

        // Perform rotation on A
        a[p][p] = ap_p - t * ap_q;
        a[q][q] = aq_q + t * ap_q;
        a[p][q] = 0.0;
        a[q][p] = 0.0;

        for i in 0..n {
            if i != p && i != q {
                let a_ip = a[i][p];
                let a_iq = a[i][q];
                a[i][p] = c * a_ip - s * a_iq;
                a[p][i] = a[i][p];
                a[i][q] = s * a_ip + c * a_iq;
                a[q][i] = a[i][q];
            }
        }

        // Perform rotation on V
        for i in 0..n {
            let v_ip = v[i][p];
            let v_iq = v[i][q];
            v[i][p] = c * v_ip - s * v_iq;
            v[i][q] = s * v_ip + c * v_iq;
        }

        iter += 1;
    }

    let mut eigenvalues = vec![0.0; n];
    for i in 0..n {
        eigenvalues[i] = a[i][i];
    }

    Ok((eigenvalues, v))
}

/// Computes the square root of a symmetric positive semi-definite matrix.
fn matrix_sqrt(matrix: &[Vec<f32>]) -> Result<Vec<Vec<f32>>, &'static str> {
    let n = matrix.len();
    let (evals, evecs) = jacobi_eigen(matrix, 100)?;

    // Construct diagonal sqrt eigenvalues matrix
    let mut d_sqrt = vec![vec![0.0; n]; n];
    for i in 0..n {
        if evals[i] > 0.0 {
            d_sqrt[i][i] = evals[i].sqrt();
        }
    }

    // Multiply: V * D_sqrt
    let mut temp = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            temp[i][j] = evecs[i][j] * d_sqrt[j][j];
        }
    }

    // Multiply: (V * D_sqrt) * V^T
    let mut res = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            let mut sum = 0.0;
            for k in 0..n {
                sum += temp[i][k] * evecs[j][k]; // evecs[j][k] is V^T[k][j]
            }
            res[i][j] = sum;
        }
    }

    Ok(res)
}

/// Computes the Fréchet distance between two datasets of vectors.
/// Exposes the core math used in Fréchet Audio Distance (FAD).
pub fn frechet_distance(
    set_a: &[Vec<Vec<f32>>],
    set_b: &[Vec<Vec<f32>>],
) -> Result<f32, AcoustixError> {
    // Flatten the lists of frames to datasets of vectors
    let flat_a: Vec<Vec<f32>> = set_a.iter().flat_map(|v| v.clone()).collect();
    let flat_b: Vec<Vec<f32>> = set_b.iter().flat_map(|v| v.clone()).collect();

    if flat_a.is_empty() || flat_b.is_empty() {
        return Err(AcoustixError::EmptySignal(
            "Datasets cannot be empty".to_string(),
        ));
    }

    let mean_a =
        mean_vector(&flat_a).map_err(|e| AcoustixError::InvalidParameter(e.to_string()))?;
    let mean_b =
        mean_vector(&flat_b).map_err(|e| AcoustixError::InvalidParameter(e.to_string()))?;

    let cov_a = covariance_matrix(&flat_a, &mean_a)
        .map_err(|e| AcoustixError::InvalidParameter(e.to_string()))?;
    let cov_b = covariance_matrix(&flat_b, &mean_b)
        .map_err(|e| AcoustixError::InvalidParameter(e.to_string()))?;

    let dim = mean_a.len();

    // 1. Mean difference term: ||mean_a - mean_b||^2
    let mut mean_diff_sq = 0.0;
    for i in 0..dim {
        let diff = mean_a[i] - mean_b[i];
        mean_diff_sq += diff * diff;
    }

    // 2. Covariance trace term: Tr(cov_a + cov_b - 2 * (cov_a * cov_b)^0.5)
    // To compute (cov_a * cov_b)^0.5:
    // We compute cov_a_sqrt = cov_a^0.5, and then compute (cov_a_sqrt * cov_b * cov_a_sqrt)^0.5
    let cov_a_sqrt =
        matrix_sqrt(&cov_a).map_err(|e| AcoustixError::InvalidParameter(e.to_string()))?;

    // Temp matrix = cov_a_sqrt * cov_b
    let mut temp = vec![vec![0.0; dim]; dim];
    for i in 0..dim {
        for j in 0..dim {
            let mut sum = 0.0;
            for k in 0..dim {
                sum += cov_a_sqrt[i][k] * cov_b[k][j];
            }
            temp[i][j] = sum;
        }
    }

    // Temp2 matrix = cov_a_sqrt * cov_b * cov_a_sqrt
    let mut temp2 = vec![vec![0.0; dim]; dim];
    for i in 0..dim {
        for j in 0..dim {
            let mut sum = 0.0;
            for k in 0..dim {
                sum += temp[i][k] * cov_a_sqrt[k][j];
            }
            temp2[i][j] = sum;
        }
    }

    // Square root of the combined matrix: (cov_a_sqrt * cov_b * cov_a_sqrt)^0.5
    let combined_sqrt =
        matrix_sqrt(&temp2).map_err(|e| AcoustixError::InvalidParameter(e.to_string()))?;

    // Trace computation: Tr(cov_a + cov_b - 2 * combined_sqrt)
    let mut trace_sum = 0.0;
    for i in 0..dim {
        trace_sum += cov_a[i][i] + cov_b[i][i] - 2.0 * combined_sqrt[i][i];
    }

    Ok((mean_diff_sq + trace_sum).max(0.0))
}

// =========================================================================
// 2. F0 Pitch Tracking & Metrics (RMSE and Pearson Correlation)
// =========================================================================

/// Tracks fundamental frequency (F0) contour from a signal using Autocorrelation.
/// Returns a vector of F0 values (0.0 represents unvoiced segments).
pub fn track_f0(
    signal: &[f32],
    sample_rate: u32,
    frame_len: usize,
    overlap: usize,
    min_f0: f32,
    max_f0: f32,
) -> Result<Vec<f32>, AcoustixError> {
    if signal.is_empty() {
        return Err(AcoustixError::EmptySignal("Signal is empty".to_string()));
    }
    if frame_len == 0 || overlap >= frame_len {
        return Err(AcoustixError::InvalidFraming { frame_len, overlap });
    }

    let step = frame_len - overlap;
    let min_lag = (sample_rate as f32 / max_f0).floor() as usize;
    let max_lag = (sample_rate as f32 / min_f0).ceil() as usize;

    let mut f0_contour = Vec::new();
    let mut start = 0;

    while start + frame_len <= signal.len() {
        let frame = &signal[start..(start + frame_len)];

        // Compute frame energy
        let energy: f32 = frame.iter().map(|&x| x * x).sum();
        if energy < 1e-4 {
            // Silence frame is unvoiced
            f0_contour.push(0.0);
            start += step;
            continue;
        }

        // Find pitch lag in autocorrelation
        let mut best_lag = 0;
        let mut best_corr = 0.0_f32;

        for lag in min_lag..=max_lag {
            if lag >= frame_len {
                break;
            }
            let mut corr = 0.0;
            for i in 0..(frame_len - lag) {
                corr += frame[i] * frame[i + lag];
            }
            if corr > best_corr {
                best_corr = corr;
                best_lag = lag;
            }
        }

        // Threshold peak correlation ratio to declare voiced segment
        if best_lag > 0 && best_corr / energy > 0.3 {
            let f0 = sample_rate as f32 / best_lag as f32;
            f0_contour.push(f0);
        } else {
            f0_contour.push(0.0);
        }

        start += step;
    }

    Ok(f0_contour)
}

/// Computes the Root Mean Square Error (RMSE) of the F0 pitch contour between two aligned files.
/// Only compares frames where both signals are voiced (F0 > 0.0).
pub fn f0_rmse(f0_ref: &[f32], f0_test: &[f32]) -> Result<f32, AcoustixError> {
    let len = f0_ref.len().min(f0_test.len());
    let mut diff_sq_sum = 0.0;
    let mut count = 0;

    for i in 0..len {
        if f0_ref[i] > 0.0 && f0_test[i] > 0.0 {
            let diff = f0_ref[i] - f0_test[i];
            diff_sq_sum += diff * diff;
            count += 1;
        }
    }

    if count == 0 {
        Ok(0.0)
    } else {
        Ok((diff_sq_sum / count as f32).sqrt())
    }
}

/// Computes the Pearson correlation coefficient of F0 contours across voiced frames.
pub fn f0_correlation(f0_ref: &[f32], f0_test: &[f32]) -> Result<f32, AcoustixError> {
    let len = f0_ref.len().min(f0_test.len());
    let mut ref_voiced = Vec::new();
    let mut test_voiced = Vec::new();

    for i in 0..len {
        if f0_ref[i] > 0.0 && f0_test[i] > 0.0 {
            ref_voiced.push(f0_ref[i]);
            test_voiced.push(f0_test[i]);
        }
    }

    let n = ref_voiced.len();
    if n < 2 {
        return Ok(0.0); // Not enough matching voiced segments
    }

    let mean_ref = ref_voiced.iter().sum::<f32>() / n as f32;
    let mean_test = test_voiced.iter().sum::<f32>() / n as f32;

    let mut num = 0.0;
    let mut den_ref = 0.0;
    let mut den_test = 0.0;

    for i in 0..n {
        let diff_ref = ref_voiced[i] - mean_ref;
        let diff_test = test_voiced[i] - mean_test;

        num += diff_ref * diff_test;
        den_ref += diff_ref * diff_ref;
        den_test += diff_test * diff_test;
    }

    if den_ref <= 1e-10 || den_test <= 1e-10 {
        Ok(0.0)
    } else {
        Ok(num / (den_ref * den_test).sqrt())
    }
}

// =========================================================================
// 3. Defect & Glitch Detection (Clipping and Click Transient Checks)
// =========================================================================

/// Detects digital clipping occurrences where amplitude sample magnitudes stay at full scale.
pub fn detect_clipping(signal: &[f32], threshold: f32, consecutive_samples: usize) -> usize {
    let mut clipping_count = 0;
    let mut run = 0;

    for &sample in signal {
        if sample.abs() >= threshold {
            run += 1;
            if run == consecutive_samples {
                clipping_count += 1;
            }
        } else {
            run = 0;
        }
    }
    clipping_count
}

/// Detects transient glitches or digital clicks based on sharp changes (first-order derivative).
pub fn detect_glitches(signal: &[f32], derivative_threshold: f32) -> Vec<usize> {
    let mut glitch_indices = Vec::new();
    if signal.len() < 2 {
        return glitch_indices;
    }

    for i in 1..signal.len() {
        let diff = (signal[i] - signal[i - 1]).abs();
        if diff >= derivative_threshold {
            glitch_indices.push(i);
        }
    }
    glitch_indices
}

// =========================================================================
// 4. Looping & Duration Integrity Check
// =========================================================================

/// Evaluates if speaking duration (seconds per character) is within normal ranges.
/// Flags loops (duration too high) or swallowing/deletion errors (duration too low).
pub fn check_duration_ratio(
    signal_len: usize,
    sample_rate: u32,
    text: &str,
) -> Result<f32, AcoustixError> {
    if text.is_empty() {
        return Err(AcoustixError::InvalidParameter(
            "Text cannot be empty".to_string(),
        ));
    }
    let duration_sec = signal_len as f32 / sample_rate as f32;
    let char_count = text.chars().filter(|c| !c.is_whitespace()).count();

    if char_count == 0 {
        return Err(AcoustixError::InvalidParameter(
            "Text must contain non-whitespace characters".to_string(),
        ));
    }

    Ok(duration_sec / char_count as f32)
}

/// Computes the DC offset of a signal (its average amplitude value).
pub fn dc_offset(signal: &[f32]) -> f32 {
    if signal.is_empty() {
        return 0.0;
    }
    signal.iter().sum::<f32>() / signal.len() as f32
}

/// Computes the Crest Factor (Peak-to-RMS ratio) of a signal in dB.
pub fn crest_factor(signal: &[f32]) -> Result<f32, AcoustixError> {
    if signal.is_empty() {
        return Err(AcoustixError::EmptySignal("Signal is empty".to_string()));
    }
    let mut peak = 0.0_f32;
    let mut sum_sq = 0.0_f32;
    for &x in signal {
        let abs_x = x.abs();
        if abs_x > peak {
            peak = abs_x;
        }
        sum_sq += x * x;
    }
    let rms = (sum_sq / signal.len() as f32).sqrt();
    if rms <= 1e-10 {
        return Err(AcoustixError::InvalidParameter(
            "Signal RMS is zero".to_string(),
        ));
    }
    Ok(20.0 * (peak / rms).log10())
}

/// Computes the silent padding duration (in seconds) at the start and end of a signal.
pub fn silence_padding(
    signal: &[f32],
    sample_rate: u32,
    frame_len: usize,
    overlap: usize,
    silence_threshold: f32,
) -> Result<(f32, f32), AcoustixError> {
    if signal.is_empty() {
        return Err(AcoustixError::EmptySignal("Signal is empty".to_string()));
    }
    if frame_len == 0 || overlap >= frame_len {
        return Err(AcoustixError::InvalidFraming { frame_len, overlap });
    }
    let step = frame_len - overlap;
    let mut start_padding = 0.0;
    let mut end_padding = 0.0;

    // Forward scan to find speech start
    let mut start = 0;
    let mut found_start = false;
    while start + frame_len <= signal.len() {
        let frame = &signal[start..(start + frame_len)];
        let energy = frame.iter().map(|&x| x * x).sum::<f32>() / frame_len as f32;
        if energy >= silence_threshold {
            start_padding = start as f32 / sample_rate as f32;
            found_start = true;
            break;
        }
        start += step;
    }
    if !found_start {
        let total_dur = signal.len() as f32 / sample_rate as f32;
        return Ok((total_dur, 0.0));
    }

    // Backward scan to find speech end
    let mut r_start = signal.len() - frame_len;
    let mut found_end = false;
    loop {
        let frame = &signal[r_start..(r_start + frame_len)];
        let energy = frame.iter().map(|&x| x * x).sum::<f32>() / frame_len as f32;
        if energy >= silence_threshold {
            let active_end = r_start + frame_len;
            end_padding = (signal.len() - active_end) as f32 / sample_rate as f32;
            found_end = true;
            break;
        }
        if r_start < step {
            break;
        }
        r_start -= step;
    }
    if !found_end {
        end_padding = 0.0;
    }

    Ok((start_padding, end_padding))
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

/// Computes Log Spectral Distance (LSD) restricted to a specific frequency band.
pub fn band_spectral_distance(
    ref_sig: &[f32],
    test_sig: &[f32],
    sample_rate: u32,
    frame_len: usize,
    overlap: usize,
    freq_min: f32,
    freq_max: f32,
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

    let num_bins = frame_len / 2 + 1;
    let eps = 1e-10_f32;

    // Filter active bins in frequency range
    let mut active_bins = Vec::new();
    for k in 0..num_bins {
        let freq = k as f32 * sample_rate as f32 / frame_len as f32;
        if freq >= freq_min && freq <= freq_max {
            active_bins.push(k);
        }
    }

    if active_bins.is_empty() {
        return Err(AcoustixError::InvalidParameter(
            "No frequency bins fall in the specified range".to_string(),
        ));
    }

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
        for &k in &active_bins {
            let ref_psd = ref_buf[k].norm_sqr();
            let test_psd = test_buf[k].norm_sqr();

            let ref_log = 10.0 * (ref_psd + eps).log10();
            let test_log = 10.0 * (test_psd + eps).log10();

            let diff = ref_log - test_log;
            bin_diff_sq_sum += diff * diff;
        }

        let frame_lsd = (bin_diff_sq_sum / active_bins.len() as f32).sqrt();
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

/// Computes the spectral flatness contour of a signal frame-by-frame.
/// Flatness values range from 0.0 (highly tonal) to 1.0 (pure noise).
pub fn spectral_flatness(
    signal: &[f32],
    frame_len: usize,
    overlap: usize,
) -> Result<Vec<f32>, AcoustixError> {
    if signal.is_empty() {
        return Err(AcoustixError::EmptySignal("Signal is empty".to_string()));
    }
    if frame_len == 0 || overlap >= frame_len {
        return Err(AcoustixError::InvalidFraming { frame_len, overlap });
    }

    let step = frame_len - overlap;
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(frame_len);

    let window = hamming_window(frame_len);
    let num_bins = frame_len / 2 + 1;
    let eps = 1e-10_f32;

    let mut flatness_values = Vec::new();
    let mut start = 0;

    while start + frame_len <= signal.len() {
        let end = start + frame_len;

        let mut fft_buf: Vec<Complex<f32>> = signal[start..end]
            .iter()
            .zip(window.iter())
            .map(|(&x, &w)| Complex::new(x * w, 0.0))
            .collect();

        fft.process(&mut fft_buf);

        let mut sum_psd = 0.0;
        let mut sum_log_psd = 0.0;

        for k in 0..num_bins {
            let psd = fft_buf[k].norm_sqr();
            sum_psd += psd;
            sum_log_psd += (psd + eps).ln();
        }

        let am = sum_psd / num_bins as f32;
        let gm = (sum_log_psd / num_bins as f32).exp();

        let flatness = if am <= 1e-10 { 0.0 } else { gm / am };

        flatness_values.push(flatness.clamp(0.0, 1.0));
        start += step;
    }

    Ok(flatness_values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frechet_math_identical() {
        let set_a = vec![vec![vec![1.0, 2.0], vec![3.0, 4.0]]];
        let set_b = vec![vec![vec![1.0, 2.0], vec![3.0, 4.0]]];

        let fd = frechet_distance(&set_a, &set_b).unwrap();
        assert!(fd.abs() < 1e-4);
    }

    #[test]
    fn test_f0_pitch_estimation() {
        // Generate a synthetic sine wave at 100 Hz
        let sr = 16000;
        let freq = 100.0_f32;
        let mut sig = vec![0.0_f32; 1600];
        for i in 0..1600 {
            sig[i] = (2.0 * PI * freq * i as f32 / sr as f32).sin();
        }

        let f0 = track_f0(&sig, sr, 512, 256, 50.0, 500.0).unwrap();
        assert!(f0.len() > 0);
        // Autocorrelation peak should find approx 100 Hz
        for &pitch in &f0 {
            if pitch > 0.0 {
                assert!((pitch - 100.0).abs() < 5.0);
            }
        }
    }

    #[test]
    fn test_defect_detection() {
        // Mock signal with clipping (values at 1.0)
        let signal = vec![0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0];
        let clip = detect_clipping(&signal, 0.99, 4);
        assert_eq!(clip, 1);

        // Mock signal with a sudden glitch click
        let signal_glitch = vec![0.0, 0.0, 0.9, 0.0, 0.0];
        let glitches = detect_glitches(&signal_glitch, 0.8);
        assert_eq!(glitches, vec![2, 3]);
    }

    #[test]
    fn test_dc_offset() {
        let sig_zero = vec![1.0, -1.0, 1.0, -1.0];
        assert!((dc_offset(&sig_zero) - 0.0).abs() < 1e-6);

        let sig_pos = vec![0.5, 1.5, 0.5, 1.5];
        assert!((dc_offset(&sig_pos) - 1.0).abs() < 1e-6);

        let sig_empty: [f32; 0] = [];
        assert!((dc_offset(&sig_empty) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_crest_factor() {
        let sig_dc = vec![1.0; 100];
        let cf_dc = crest_factor(&sig_dc).unwrap();
        assert!(cf_dc.abs() < 1e-4);

        let sig_pulse = vec![1.0, 0.0, 0.0, 0.0];
        let cf_pulse = crest_factor(&sig_pulse).unwrap();
        // Peak = 1.0, RMS = sqrt(1/4) = 0.5. peak/RMS = 2.0. 20 * log10(2.0) = 6.0205999
        assert!((cf_pulse - 6.0205999).abs() < 1e-4);

        let sig_empty: [f32; 0] = [];
        assert!(crest_factor(&sig_empty).is_err());

        let sig_zero = vec![0.0; 10];
        assert!(crest_factor(&sig_zero).is_err());
    }

    #[test]
    fn test_silence_padding() {
        // 100 Hz sample rate, frame_len 10, overlap 0
        // Total duration 30 samples = 0.3 seconds
        let mut sig = vec![0.0; 30];
        // Middle block has signal
        for i in 10..20 {
            sig[i] = 1.0;
        }

        let (start_pad, end_pad) = silence_padding(&sig, 100, 10, 0, 0.5).unwrap();
        assert!((start_pad - 0.1).abs() < 1e-4);
        assert!((end_pad - 0.1).abs() < 1e-4);
    }

    #[test]
    fn test_band_spectral_distance() {
        // Compare a signal with itself - should be 0
        let sig = vec![0.5_f32; 256];
        let lsd = band_spectral_distance(&sig, &sig, 16000, 128, 64, 0.0, 8000.0).unwrap();
        assert!(lsd.abs() < 1e-4);
    }

    #[test]
    fn test_spectral_flatness() {
        // Highly tonal sine wave
        let sr = 16000;
        let freq = 400.0_f32;
        let mut sig_tone = vec![0.0_f32; 1024];
        for i in 0..1024 {
            sig_tone[i] = (2.0 * PI * freq * i as f32 / sr as f32).sin();
        }

        let flatness_tone = spectral_flatness(&sig_tone, 256, 128).unwrap();
        assert!(flatness_tone.len() > 0);
        // Tonal signal should have a low flatness score
        for &f in &flatness_tone {
            assert!(f < 0.1, "Expected tonal flatness < 0.1, got {}", f);
        }

        // Noise signal
        let mut sig_noise = vec![0.0_f32; 1024];
        // Deterministic pseudo-random noise using LCG
        let mut seed: u32 = 12345;
        for i in 0..1024 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            sig_noise[i] = (seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
        }
        let flatness_noise = spectral_flatness(&sig_noise, 256, 128).unwrap();
        assert!(flatness_noise.len() > 0);
        // Noise signal should have a significantly higher flatness score
        for &f in &flatness_noise {
            assert!(f > 0.3, "Expected noise flatness > 0.3, got {}", f);
        }
    }
}
