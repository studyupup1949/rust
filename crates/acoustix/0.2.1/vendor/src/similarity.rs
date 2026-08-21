use crate::error::AcoustixError;
use num_complex::Complex;
use rustfft::FftPlanner;
use std::f32::consts::PI;
use std::path::Path;

/// Loads a WAV file and converts it to mono f32 samples normalized between -1.0 and 1.0.
/// Returns a tuple of (samples, sample_rate).
#[cfg(feature = "wav")]
pub fn load_wav<P: AsRef<Path>>(path: P) -> Result<(Vec<f32>, u16), AcoustixError> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as u16;

    let raw_samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect(),
        hound::SampleFormat::Int => {
            let max_val = (1 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.unwrap_or(0) as f32 / max_val)
                .collect()
        }
    };

    let channels = spec.channels as usize;
    if channels > 1 {
        let mut mono = Vec::with_capacity(raw_samples.len() / channels);
        for chunk in raw_samples.chunks_exact(channels) {
            let sum: f32 = chunk.iter().sum();
            mono.push(sum / channels as f32);
        }
        Ok((mono, sample_rate))
    } else {
        Ok((raw_samples, sample_rate))
    }
}

/// Loads raw signed 16-bit little-endian mono PCM samples from a file and normalizes them to the range `[-1.0, 1.0]`.
pub fn load_pcm<P: AsRef<Path>>(path: P) -> Result<Vec<f32>, AcoustixError> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    let _ = file.read_to_end(&mut bytes)?;

    let mut samples = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let val = i16::from_le_bytes([chunk[0], chunk[1]]);
        samples.push(val as f32 / 32768.0);
    }
    Ok(samples)
}

/// Unified loader that loads audio from a file. Checks the file extension to decide between loading as WAV or raw PCM.
///
/// If it is a PCM or raw file, the `pcm_sample_rate` is used as the sample rate. For WAV files, the sample rate is read
/// directly from the WAV header.
pub fn load_audio<P: AsRef<Path>>(
    path: P,
    pcm_sample_rate: u16,
) -> Result<(Vec<f32>, u16), AcoustixError> {
    let path_ref = path.as_ref();
    let extension = path_ref
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());

    match extension.as_deref() {
        #[cfg(feature = "wav")]
        Some("wav") => load_wav(path_ref),
        #[cfg(not(feature = "wav"))]
        Some("wav") => Err(AcoustixError::InvalidParameter(
            "WAV support is disabled. Enable the 'wav' feature to load WAV files.".to_string(),
        )),
        Some("pcm") | Some("raw") => {
            let samples = load_pcm(path_ref)?;
            Ok((samples, pcm_sample_rate))
        }
        _ => {
            let msg = format!("Unsupported file extension. Please use .wav, .pcm, or .raw");
            Err(AcoustixError::InvalidParameter(msg))
        }
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

/// Converts Hz frequency to Mel scale.
fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

/// Converts Mel scale back to Hz frequency.
fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0)
}

/// Computes orthonormal Discrete Cosine Transform (DCT-II) of a slice.
fn dct_ii_ortho(input: &[f32]) -> Vec<f32> {
    let b = input.len();
    let mut output = vec![0.0; b];
    let factor = PI / b as f32;
    for n in 0..b {
        let mut sum = 0.0;
        for k in 0..b {
            sum += input[k] * (factor * n as f32 * (k as f32 + 0.5)).cos();
        }
        let scale = if n == 0 {
            (1.0 / b as f32).sqrt()
        } else {
            (2.0 / b as f32).sqrt()
        };
        output[n] = sum * scale;
    }
    output
}

/// Extracts MFCC features from a signal.
/// Returns a vector of frames, where each frame is a vector of MFCC coefficients.
pub fn extract_mfcc(
    signal: &[f32],
    sample_rate: u16,
    frame_len: usize,
    overlap: usize,
    num_mel_bands: usize,
    num_coefficients: usize,
) -> Result<Vec<Vec<f32>>, AcoustixError> {
    if signal.is_empty() {
        return Err(AcoustixError::EmptySignal(
            "Input signal cannot be empty".to_string(),
        ));
    }
    if frame_len == 0 || overlap >= frame_len {
        return Err(AcoustixError::InvalidFraming { frame_len, overlap });
    }
    if num_coefficients > num_mel_bands {
        return Err(AcoustixError::InvalidParameter(
            "Number of coefficients cannot exceed number of Mel bands".to_string(),
        ));
    }

    let step = frame_len - overlap;
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(frame_len);
    let window = hamming_window(frame_len);

    let num_bins = frame_len / 2 + 1;
    let f_min = 0.0_f32;
    let f_max = sample_rate as f32 / 2.0;

    let mel_min = hz_to_mel(f_min);
    let mel_max = hz_to_mel(f_max);

    // Compute Mel scale filter bank center/edge frequencies
    let mut mel_pts = Vec::with_capacity(num_mel_bands + 2);
    for i in 0..=(num_mel_bands + 1) {
        let m = mel_min + i as f32 * (mel_max - mel_min) / (num_mel_bands + 1) as f32;
        mel_pts.push(mel_to_hz(m));
    }

    // Map Hz frequencies to FFT bin indices
    let mut bin_pts = Vec::with_capacity(num_mel_bands + 2);
    for hz in mel_pts {
        let b = ((frame_len + 1) as f32 * hz / sample_rate as f32).floor() as usize;
        bin_pts.push(b.min(num_bins - 1));
    }

    // Build triangular filterbank weights
    let mut filters = vec![vec![0.0_f32; num_bins]; num_mel_bands];
    for m in 0..num_mel_bands {
        let left = bin_pts[m];
        let center = bin_pts[m + 1];
        let right = bin_pts[m + 2];

        // Ramping up
        if center > left {
            for k in left..center {
                filters[m][k] = (k - left) as f32 / (center - left) as f32;
            }
        }
        // Peak
        filters[m][center] = 1.0;
        // Ramping down
        if right > center {
            for k in (center + 1)..=right {
                if k < num_bins {
                    filters[m][k] = (right - k) as f32 / (right - center) as f32;
                }
            }
        }
    }

    let eps = 1e-10_f32;
    let mut mfcc_frames = Vec::new();

    let mut start = 0;
    while start + frame_len <= signal.len() {
        let end = start + frame_len;

        let mut fft_buf: Vec<Complex<f32>> = signal[start..end]
            .iter()
            .zip(window.iter())
            .map(|(&x, &w)| Complex::new(x * w, 0.0))
            .collect();

        fft.process(&mut fft_buf);

        // Compute power spectrum (only first half)
        let mut psd = Vec::with_capacity(num_bins);
        for k in 0..num_bins {
            psd.push(fft_buf[k].norm_sqr());
        }

        // Apply Mel filterbank
        let mut mel_energies = vec![0.0_f32; num_mel_bands];
        for m in 0..num_mel_bands {
            let mut energy = 0.0_f32;
            for k in 0..num_bins {
                energy += psd[k] * filters[m][k];
            }
            mel_energies[m] = (energy + eps).ln(); // Log Mel energies (natural log)
        }

        // Apply DCT-II to obtain MFCCs
        let dct_out = dct_ii_ortho(&mel_energies);
        let mfcc: Vec<f32> = dct_out.into_iter().take(num_coefficients).collect();
        mfcc_frames.push(mfcc);

        start += step;
    }

    Ok(mfcc_frames)
}

/// Performs Dynamic Time Warping (DTW) sequence alignment between two sequences of feature vectors.
/// Returns a tuple of (cumulative_distance, path_coordinates).
pub fn dynamic_time_warping(seq_a: &[Vec<f32>], seq_b: &[Vec<f32>]) -> (f32, Vec<(usize, usize)>) {
    let m = seq_a.len();
    let n = seq_b.len();

    if m == 0 || n == 0 {
        return (0.0, Vec::new());
    }

    // Helper for Euclidean distance between two frames
    let dist = |v1: &[f32], v2: &[f32]| {
        let mut sum = 0.0;
        for (&x, &y) in v1.iter().zip(v2.iter()) {
            let d = x - y;
            sum += d * d;
        }
        sum.sqrt()
    };

    // 2D cost matrix
    let mut d = vec![vec![f32::INFINITY; n + 1]; m + 1];
    d[0][0] = 0.0;

    for i in 1..=m {
        for j in 1..=n {
            let cost = dist(&seq_a[i - 1], &seq_b[j - 1]);
            let min_prev = d[i - 1][j].min(d[i][n.min(j - 1)]).min(d[i - 1][j - 1]);
            d[i][j] = cost + min_prev;
        }
    }

    // Backtrack to find path
    let mut path = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 && j > 0 {
        path.push((i - 1, j - 1));
        if i == 1 && j == 1 {
            break;
        }
        if i == 1 {
            j -= 1;
        } else if j == 1 {
            i -= 1;
        } else {
            let p_diag = d[i - 1][j - 1];
            let p_up = d[i - 1][j];
            let p_left = d[i][j - 1];

            if p_diag <= p_up && p_diag <= p_left {
                i -= 1;
                j -= 1;
            } else if p_up <= p_left {
                i -= 1;
            } else {
                j -= 1;
            }
        }
    }
    path.reverse();

    (d[m][n], path)
}

/// Computes the Mel Cepstral Distortion (MCD) in dB between two MFCC feature sequences using DTW alignment.
pub fn compute_mcd(
    seq_a: &[Vec<f32>],
    seq_b: &[Vec<f32>],
    exclude_first: bool,
) -> Result<f32, AcoustixError> {
    if seq_a.is_empty() || seq_b.is_empty() {
        return Err(AcoustixError::EmptySignal(
            "MFCC sequences must not be empty".to_string(),
        ));
    }

    let start_idx = if exclude_first { 1 } else { 0 };

    // Align sequences via DTW
    let (_, path) = dynamic_time_warping(seq_a, seq_b);
    if path.is_empty() {
        return Err(AcoustixError::AlignmentError(
            "DTW alignment produced an empty path".to_string(),
        ));
    }

    let mut dist_sum = 0.0;
    for &(i, j) in &path {
        let frame_a = &seq_a[i];
        let frame_b = &seq_b[j];

        let mut bin_diff_sum = 0.0;
        for d in start_idx..frame_a.len().min(frame_b.len()) {
            let diff = frame_a[d] - frame_b[d];
            bin_diff_sum += diff * diff;
        }
        dist_sum += bin_diff_sum.sqrt();
    }

    let mean_dist = dist_sum / path.len() as f32;

    // MCD scaling factor: (10 * sqrt(2)) / ln(10) ≈ 6.1418514
    let mcd_scale = 10.0 * 2.0_f32.sqrt() / 10.0_f32.ln();
    Ok(mcd_scale * mean_dist)
}

/// Computes end-to-end MCD directly from raw signals.
pub fn compute_mcd_between_signals(
    ref_sig: &[f32],
    test_sig: &[f32],
    sample_rate: u16,
    frame_len: usize,
    overlap: usize,
    num_mel_bands: usize,
    num_coefficients: usize,
    exclude_first: bool,
) -> Result<f32, AcoustixError> {
    let seq_a = extract_mfcc(
        ref_sig,
        sample_rate,
        frame_len,
        overlap,
        num_mel_bands,
        num_coefficients,
    )?;
    let seq_b = extract_mfcc(
        test_sig,
        sample_rate,
        frame_len,
        overlap,
        num_mel_bands,
        num_coefficients,
    )?;
    compute_mcd(&seq_a, &seq_b, exclude_first)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_dct_ii() {
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let dct = dct_ii_ortho(&input);

        // Sum of squares of input: 1 + 4 + 9 + 16 = 30
        let input_energy: f32 = input.iter().map(|x| x * x).sum();

        // Sum of squares of orthogonal DCT output should be equal (Parseval's theorem)
        let dct_energy: f32 = dct.iter().map(|x| x * x).sum();

        assert!((input_energy - dct_energy).abs() < 1e-4);
    }

    #[test]
    fn test_dtw_simple() {
        let seq_a = vec![vec![1.0], vec![2.0], vec![3.0]];
        let seq_b = vec![vec![1.0], vec![1.5], vec![2.0], vec![3.0]];

        let (dist, path) = dynamic_time_warping(&seq_a, &seq_b);

        // Cumulative distance should be 0.5 (alignment: 1-1, 2-1.5, 2-2, 3-3)
        // Euclidean distance steps: |1-1|=0, |2-1.5|=0.5, |2-2|=0, |3-3|=0. Total = 0.5
        assert!((dist - 0.5).abs() < 1e-5);
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], (0, 0));
        assert_eq!(path[1], (0, 1));
        assert_eq!(path[2], (1, 2));
        assert_eq!(path[3], (2, 3));
    }

    #[test]
    fn test_mcd_identical() {
        let sig = vec![0.1; 1000];
        let res = compute_mcd_between_signals(&sig, &sig, 16000, 256, 128, 20, 13, true).unwrap();
        assert!(res.abs() < 1e-5);
    }

    #[test]
    fn test_mcd_different() {
        let sig_a = vec![0.1; 1000];
        let mut sig_b = vec![0.1; 1000];
        for i in (0..1000).step_by(2) {
            sig_b[i] = -0.1;
        }
        let res =
            compute_mcd_between_signals(&sig_a, &sig_b, 16000, 256, 128, 20, 13, true).unwrap();
        assert!(res > 0.0);
    }

    #[test]
    #[cfg(feature = "wav")]
    fn test_load_wav_integration() {
        let path = "target/test_temp.wav";

        // Write a mock mono wav file
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        {
            let mut writer = hound::WavWriter::create(path, spec).unwrap();
            // Write a simple sine wave or constant samples
            for i in 0..160 {
                let sample = (32767.0 * (2.0 * PI * i as f32 / 160.0).sin()) as i16;
                writer.write_sample(sample).unwrap();
            }
            writer.finalize().unwrap();
        }

        // Load it back
        let (samples, sr) = load_wav(path).unwrap();
        assert_eq!(sr, 16000);
        assert_eq!(samples.len(), 160);
        assert!((samples[40] - 1.0).abs() < 0.05); // sin(pi/2) = 1.0

        // Clean up
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_load_pcm_integration() {
        use std::io::Write;
        let path = "target/test_temp.pcm";

        // Write a mock mono pcm file (raw i16 little-endian samples)
        {
            let mut file = fs::File::create(path).unwrap();
            for i in 0..160 {
                let sample = (32767.0 * (2.0 * PI * i as f32 / 160.0).sin()) as i16;
                let bytes = sample.to_le_bytes();
                file.write_all(&bytes).unwrap();
            }
        }

        // Load it back using load_pcm
        let samples = load_pcm(path).unwrap();
        assert_eq!(samples.len(), 160);
        assert!((samples[40] - 1.0).abs() < 0.05); // sin(pi/2) = 1.0

        // Load it back using load_audio
        let (samples_audio, sr) = load_audio(path, 22050).unwrap();
        assert_eq!(sr, 22050);
        assert_eq!(samples_audio.len(), 160);
        assert!((samples_audio[40] - 1.0).abs() < 0.05);

        // Clean up
        let _ = fs::remove_file(path);
    }
}
