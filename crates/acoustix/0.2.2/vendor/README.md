# acoustix

[![Cargo Test](https://github.com/quietscroll/acoustix/actions/workflows/cargo-test.yml/badge.svg)](https://github.com/quietscroll/acoustix/actions/workflows/cargo-test.yml)
[![docs.rs](https://img.shields.io/docsrs/acoustix)](https://docs.rs/acoustix)

`acoustix` is a high-performance, open-source Rust library for the **automated validation and evaluation** of Text-to-Speech (TTS) and voice AI models.

It implements standard objective speech metrics (Mel Cepstral Distortion, Dynamic Time Warping, Log Spectral Distance, and Segmental SNR) alongside transcription evaluations (WER, CER), speaker embedding similarity (SIM), speaker attribution accuracy (ACC), and advanced TTS validation checks (Fréchet distance solver, pitch contour analysis, and audio defect detection). This allows developers to run fast, memory-safe, and highly concurrent evaluations locally or in CI/CD pipelines without needing human raters or heavy machine learning models.

## Features

- **Objective Quality Metrics**:
  - **Segmental SNR (SegSNR)**: Frame-by-frame Signal-to-Noise Ratio calculation, filtering out silent frames and clipping frame SNR to prevent outliers.
  - **Log Spectral Distance (LSD)**: Measures distortion in the frequency domain using FFT and overlapping windowed frames.
- **Objective Similarity Metrics**:
  - **Mel Cepstral Distortion (MCD)**: Computes the spectral envelope difference (MCD-13, excluding the 0th energy coefficient by default) in decibels (dB).
  - **Dynamic Time Warping (DTW)**: Aligns signals of different lengths to ensure accurate frame-to-frame distortion calculation regardless of speaking rate differences.
- **Transcription & Speaker Evaluation** (matching MOSS-TTS evaluation standards):
  - **Word Error Rate (WER) & Character Error Rate (CER)**: Text-based alignment metrics using the Levenshtein distance dynamic programming algorithm.
  - **Speaker Similarity (SIM)**: Cosine similarity computation between speaker embedding vectors.
  - **Speaker Attribution Accuracy (ACC)**: Measures classification accuracy of speaker IDs across turns in multi-speaker dialogue synthesis.
- **Advanced TTS Quality Controls** (`advanced` module):
  - **Fréchet Distance Calculator**: Exposes the core mathematical solver used in Fréchet Audio Distance (FAD) to compute distances between datasets of frame embedding vectors.
  - **F0 Contour Tracking**: Autocorrelation-based pitch tracker to estimate pitch curves.
  - **F0 Intonation Metrics**: F0 RMSE (root mean square error) and F0 Pearson Correlation to measure speech intonation alignment.
  - **Defect Detection**: Clipping detectors (identifying full-scale flat-lining) and click/transient glitch detectors.
  - **Speaking Rate / Integrity Check**: Evaluates speaking speed (seconds per character) to flag abnormal silence loops or swallowed speech segments.
- **Decision & Aggregation Engine**:
  - **Automated Preference**: Compares two candidate audios against a reference baseline and declares the superior one.
  - **Rank Aggregation**: Takes a list of candidate model evaluations, normalizes their scores, and ranks them using a weighted multi-metric composite score.

## Installation

Add `acoustix` to your `Cargo.toml`:

```toml
[dependencies]
acoustix = { version = "0.1" }
```

### Feature Flags

`acoustix` keeps its dependency footprint minimal and supports optional feature gates:

* **`wav`** (Enabled by default): Enables WAV audio file loading support (depends on `hound`).
* **`default`**: Includes `wav`.

To use `acoustix` in a lightweight PCM-only environment, disable default features:

```toml
[dependencies]
acoustix = { version = "0.1.0", default-features = false }
```

---

## Quick Start

### 1. Load Audio and Compute Quality Metrics (SNR / LSD / MCD)
```rust
use acoustix::quality::{segmental_snr, log_spectral_distance};
use acoustix::similarity::{compute_mcd_between_signals, load_wav};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load reference and synthesized WAV files
    let (ref_sig, sr_ref) = load_wav("tests/files/LJ050-0002.wav")?;
    let (test_sig, sr_test) = load_wav("tests/files/LJ050-0004.wav")?;

    // Calculate Segmental SNR (frame size: 512, overlap: 256)
    let snr = segmental_snr(&ref_sig, &test_sig, 512, 256, -10.0, 35.0, 1e-4)?;
    println!("Segmental SNR: {:.4} dB", snr);

    // Calculate Log Spectral Distance
    let lsd = log_spectral_distance(&ref_sig, &test_sig, 512, 256)?;
    println!("Log Spectral Distance: {:.4} dB", lsd);

    // Compute MCD-13 with DTW alignment
    let mcd = compute_mcd_between_signals(&ref_sig, &test_sig, sr_ref, 512, 256, 20, 13, true)?;
    println!("Mel Cepstral Distortion: {:.4} dB", mcd);

    Ok(())
}
```

### 2. Run Advanced TTS Quality Checks
```rust
use acoustix::advanced::{track_f0, f0_rmse, f0_correlation, detect_clipping, detect_glitches};
use acoustix::similarity::load_wav;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (ref_sig, sr) = load_wav("tests/files/LJ050-0002.wav")?;
    let (test_sig, _) = load_wav("tests/files/LJ050-0002.wav")?;

    // 1. Pitch Tracking & Intonation Alignment
    let f0_ref = track_f0(&ref_sig, sr, 512, 256, 50.0, 500.0)?;
    let f0_test = track_f0(&test_sig, sr, 512, 256, 50.0, 500.0)?;

    let pitch_rmse = f0_rmse(&f0_ref, &f0_test)?;
    let pitch_corr = f0_correlation(&f0_ref, &f0_test)?;
    println!("Pitch RMSE: {:.4} Hz, Correlation: {:.4}", pitch_rmse, pitch_corr);

    // 2. Audio Defect Checks
    let clips = detect_clipping(&test_sig, 0.999, 4);
    let clicks = detect_glitches(&test_sig, 0.95).len();
    println!("Clips: {}, Clicks/Glitches: {}", clips, clicks);

    Ok(())
}
```

---

## CLI Evaluation Utility

`acoustix` includes a test binary in the `examples/` directory that reads WAV or raw 16-bit PCM files (decided by extension) and runs all objective evaluations.

To evaluate audio files:
```bash
cargo run --example evaluate <reference_audio> <test_audio> [ref_text] [hyp_text]
```

### Output Example:
```
Loading WAV file: "tests/files/LJ050-0002.wav"
Loading WAV file: "tests/files/LJ050-0002.wav"
Evaluation Sample Rate: 22050 Hz
---------------------------------------
Segmental SNR:          35.0000 dB
Log Spectral Distance:  0.0000 dB
Band-Specific LSD:      0.0000 dB (300-3400 Hz)
Mel Cepstral Distortion: 0.0000 dB
F0 Pitch Contour RMSE:  0.0000 Hz
F0 Pitch Correlation:   1.0000
Fréchet MFCC Distance:  0.0000
---------------------------------------
Clipping Instances (Ref/Test): 0 / 0
Glitch Clicks Detected (Ref/Test): 0 / 0
DC Offset (Ref/Test):           0.000006 / 0.000006
Crest Factor (Ref/Test):        19.1515 dB / 19.1515 dB
Silence Padding - Start (Ref/Test): 0.0000s / 0.0000s
Silence Padding - End (Ref/Test):   0.1277s / 0.1277s
Mean Spectral Flatness (Ref/Test):  0.0237 / 0.0237
```

---

## Detailed Metric & Function Reference

Here is a detailed breakdown of all existing functions and speech quality/similarity metrics implemented in `acoustix`:

### 1. Segmental SNR (`segmental_snr`)
* **What it is**: The Segmental Signal-to-Noise Ratio (SegSNR) in dB between a reference signal and a test signal.
* **Why it's important**: Standard SNR can be heavily biased by silent intervals or brief loud bursts. SegSNR divides the signals into short frames, calculates the SNR for each active frame, and averages them. This prevents silent regions from inflating or deflating the overall SNR and provides a truer assessment of perceived speech noise levels.
* **Implementation**: Divides the signals into frames using a specified frame length and overlap. For each frame where the reference signal energy exceeds a `silence_threshold`, it computes the frame SNR:
  $$\text{Frame SNR} = 10 \log_{10} \left( \frac{\sum x_i^2}{\sum (x_i - y_i)^2} \right)$$
  The frame SNR is clamped between `min_snr` (default -10.0) and `max_snr` (default 35.0) to prevent outliers from distorting the overall average.
* **When to use**: To measure background noise levels, distortion, or reconstruction errors in audio coding, vocoding, or speech synthesis relative to a natural reference signal.

### 2. Log Spectral Distance (`log_spectral_distance`)
* **What it is**: The Log Spectral Distance (LSD) in dB between a reference signal and a test signal.
* **Why it's important**: Measures how much the power spectrum of the synthesized speech deviates from the reference speech. It is highly sensitive to overall spectral shape distortion, which directly correlates to perceived coloring of the voice or frequency anomalies.
* **Implementation**: Breaks the signals into overlapping frames, applies a Hamming window, and computes the power spectral density (PSD) using FFT. The LSD for each frame is calculated across all Nyquist frequency bins ($K$):
  $$\text{Frame LSD} = \sqrt{ \frac{1}{K} \sum_{k=0}^{K-1} \left( 10 \log_{10} |X_{\text{ref}}(k)|^2 - 10 \log_{10} |X_{\text{test}}(k)|^2 \right)^2 }$$
  The overall LSD is the average of these frame LSD values.
* **When to use**: To evaluate general spectral fidelity, voice timbre reconstruction, and frequency distortion in speech synthesis models.

### 3. Band-Specific LSD (`band_spectral_distance`)
* **What it is**: The Log Spectral Distance computed only over a specific band of frequencies $[f_{\text{min}}, f_{\text{max}}]$:
  $$\text{Band LSD} = \frac{1}{M} \sum_{m=1}^{M} \sqrt{ \frac{1}{|K_{\text{band}}|} \sum_{k \in K_{\text{band}}} \left( 10 \log_{10} |X_{\text{ref}}(m, k)|^2 - 10 \log_{10} |X_{\text{test}}(m, k)|^2 \right)^2 }$$
* **Why it's important**: Speech energy is concentrated in specific frequency bands (e.g. the telephone band $300\text{ Hz} - 3400\text{ Hz}$ or the core human voice band $100\text{ Hz} - 8000\text{ Hz}$). Distortions in these bands affect intelligibility and speaker identity far more than high-frequency noise or subsonic rumble. Focusing LSD on specific bands provides a targeted quality metric.
* **Implementation**: Computes the Short-Time Fourier Transform (STFT) of both signals using a Hamming window. Then, it identifies FFT bins that fall within the range $[f_{\text{min}}, f_{\text{max}}]$ and calculates the root-mean-square log spectral distance across these bins.
* **When to use**: To isolate and assess speech quality within the core vocal range, disregarding subsonic hum, high-frequency compression artifacts, or background noise outside the voice band.

### 4. Mel Cepstral Distortion (`compute_mcd` / `compute_mcd_between_signals`)
* **What it is**: The Mel Cepstral Distortion (MCD) in dB between two speech signals.
* **Why it's important**: MCD is the most widely used objective metric for assessing the quality of speech synthesis models. It measures differences in the spectral envelope (specifically vocal tract characteristics) while mimicking human auditory perception via the Mel scale.
* **Implementation**: Extracts Mel-Frequency Cepstral Coefficients (MFCCs) for both signals. Aligns the MFCC vectors using Dynamic Time Warping (DTW) to handle speaking rate variations. The MCD is computed as:
  $$\text{MCD (dB)} = \frac{10 \sqrt{2}}{\ln 10} \frac{1}{P} \sum_{p=1}^{P} \sqrt{ \sum_{d=d_{\text{start}}}^{D-1} (c_{\text{ref}}(p, d) - c_{\text{test}}(p, d))^2 }$$
  where $P$ is the number of aligned frames, $D$ is the number of coefficients, and $d_{\text{start}}$ is usually 1 to exclude the 0-th energy coefficient (vocal power).
* **When to use**: The default quality metric for comparing TTS models, comparing speaker timbre preservation, and benchmarking voice conversion systems.

### 5. Dynamic Time Warping (`dynamic_time_warping`)
* **What it is**: An algorithm that computes an optimal alignment between two time-dependent sequences of different lengths.
* **Why it's important**: TTS models often generate speech that is slightly faster or slower than the reference audio, or has slightly different syllable timing. Simple frame-by-frame comparison would show huge errors. DTW non-linearly warps the time dimension to align matching acoustic events, enabling accurate comparison.
* **Implementation**: Constructs a cost matrix where each cell represents the Euclidean distance between feature vectors. It finds the minimal cost path from $(0,0)$ to $(M-1, N-1)$ using dynamic programming:
  $$D(i, j) = \text{dist}(A_i, B_j) + \min \Big( D(i-1, j), D(i, j-1), D(i-1, j-1) \Big)$$
* **When to use**: To align spectral feature sequences (like MFCCs or Mel spectrograms) before computing distance metrics such as MCD or F0 alignment metrics.

### 6. Word Error Rate (`word_error_rate`)
* **What it is**: Measures transcription accuracy at the word level between a reference transcript and a hypothesis transcript.
* **Why it's important**: Speech quality metrics don't tell you if the synthesized speech is intelligible or if the words were pronounced correctly. High WER indicates poor pronunciation, mumbled words, or complete omissions.
* **Implementation**: Tokenizes the texts and removes punctuation. Uses the Levenshtein distance dynamic programming algorithm to count Insertions ($I$), Deletions ($D$), and Substitutions ($S$), normalized by the length of the reference ($N$):
  $$\text{WER} = \frac{S + D + I}{N}$$
* **When to use**: To assess the intelligibility and text-to-speech accuracy of synthetic voice models, making sure words are not omitted, duplicated, or mispronounced.

### 7. Character Error Rate (`character_error_rate`)
* **What it is**: Measures transcription accuracy at the character level between a reference transcript and a hypothesis transcript.
* **Why it's important**: CER is more sensitive than WER for morphologically rich languages or minor spelling/pronunciation errors, capturing single-character additions or omissions.
* **Implementation**: Computes Levenshtein distance at the character level, ignoring punctuation:
  $$\text{CER} = \frac{S_{\text{char}} + D_{\text{char}} + I_{\text{char}}}{N_{\text{char}}}$$
* **When to use**: When evaluating pronunciation accuracy on short utterances, fine-grained phonetic transcriptions, or morphologically rich languages.

### 8. Speaker Similarity (`cosine_similarity`)
* **What it is**: The speaker similarity (SIM) calculated as the cosine similarity between two high-dimensional speaker embedding vectors.
* **Why it's important**: Speaker embeddings represent the unique speaker identity (timbre, accent, physiological voice features). A high cosine similarity between the reference and synthesized voice embeddings indicates strong speaker identity preservation.
* **Implementation**: Calculates the normalized dot product of two embedding vectors:
  $$\text{Cosine Similarity} = \frac{\mathbf{u} \cdot \mathbf{v}}{\|\mathbf{u}\| \|\mathbf{v}\|} = \frac{\sum u_i v_i}{\sqrt{\sum u_i^2} \sqrt{\sum v_i^2}}$$
  Output is bounded in $[-1.0, 1.0]$.
* **When to use**: To evaluate speaker mimicry, speaker adaptation, or voice clone consistency in zero-shot multi-speaker TTS models.

### 9. Speaker Attribution Accuracy (`speaker_attribution_accuracy`)
* **What it is**: The Speaker Attribution Accuracy (ACC) of a speaker classifier on a sequence of turns in multi-speaker speech synthesis.
* **Why it's important**: In conversational or multi-speaker synthesis, it is vital that the correct voice is attributed to each speaker segment.
* **Implementation**: Compares a sequence of predicted speaker labels against the actual labels and calculates the accuracy:
  $$\text{ACC} = \frac{\sum_{i=1}^{L} [y_i == \hat{y}_i]}{L}$$
* **When to use**: To evaluate the performance of dialogue TTS models, podcast generation systems, or role-playing voice synthesis.

### 10. Fréchet Audio Distance Math (`frechet_distance`)
* **What it is**: The Fréchet Distance calculated between the distribution of feature vectors (e.g., embeddings) of a reference audio dataset and a generated audio dataset.
* **Why it's important**: Used for calculating Fréchet Audio Distance (FAD), a reference-free speech/audio quality metric that correlates closely with human ratings. A lower distance indicates that the statistical distribution of the generated audio matches the reference corpus.
* **Implementation**: Fits multivariate Gaussian distributions to the feature datasets and computes the 2-Wasserstein distance:
  $$d^2 = \|\boldsymbol{\mu}_a - \boldsymbol{\mu}_b\|^2 + \text{Tr}\left(\mathbf{\Sigma}_a + \mathbf{\Sigma}_b - 2\big(\mathbf{\Sigma}_a^{1/2} \mathbf{\Sigma}_b \mathbf{\Sigma}_a^{1/2}\big)^{1/2}\right)$$
* **When to use**: To run corporate or broad-scale reference-free evaluations of overall audio quality, recording quality, and acoustic realism over large sets of generated audios.

### 11. F0 Pitch Contour Tracking (`track_f0`)
* **What it is**: Tracks the fundamental frequency (F0) contour of a signal over time.
* **Why it's important**: Pitch contour represents the melody and intonation of speech. Tracking F0 is the foundation for checking emotional expressiveness, sentence stress, and speaker characteristics.
* **Implementation**: Performs a windowed autocorrelation search on the signal. The fundamental frequency is estimated from the best lag in the range $[f_{\text{min}}, f_{\text{max}}]$ whose autocorrelation strength exceeds a voicing threshold. Unvoiced frames default to `0.0`.
* **When to use**: To extract pitch curves for downstream prosody comparisons or voice analysis.

### 12. F0 Pitch Contour RMSE (`f0_rmse`)
* **What it is**: Computes the Root Mean Square Error (RMSE) of the F0 pitch contour between two aligned files.
* **Why it's important**: Measures absolute difference in pitch values across voiced speech segments. Lower RMSE indicates that the pitch levels and range of the synthesized voice closely match the reference speaker.
* **Implementation**: Compares only frames where both signals are voiced (F0 > 0.0):
  $$\text{F0 RMSE} = \sqrt{\frac{1}{N_{\text{voiced}}} \sum_{i \in \text{voiced}} (F0_{\text{ref}}(i) - F0_{\text{test}}(i))^2}$$
* **When to use**: To check if a voice clone has matching pitch levels and contour scale to the source speaker.

### 13. F0 Pitch Correlation (`f0_correlation`)
* **What it is**: Computes the Pearson correlation coefficient between F0 contours across voiced frames.
* **Why it's important**: RMSE only measures absolute pitch values. Pitch correlation measures the shape/relative contour of the pitch curve (e.g. rising intonation at the end of questions). High correlation indicates natural emotional and prosodic tracking.
* **Implementation**: Calculates the Pearson correlation over shared voiced segments:
  $$\rho = \frac{\sum (F0_{\text{ref}} - \bar{F0}_{\text{ref}})(F0_{\text{test}} - \bar{F0}_{\text{test}})}{\sqrt{\sum (F0_{\text{ref}} - \bar{F0}_{\text{ref}})^2 \sum (F0_{\text{test}} - \bar{F0}_{\text{test}})^2}}$$
* **When to use**: To evaluate prosody quality, emotional intonation preservation, and sentence-level inflection matching.

### 14. Clipping Detection (`detect_clipping`)
* **What it is**: Detects occurrences of digital clipping where sample values stay at the full digital scale.
* **Why it's important**: Clipping introduces harsh harmonic distortion and clicks that are highly unpleasant to listen to.
* **Implementation**: Counts the number of times a run of consecutive samples exceeds a threshold (e.g. `threshold = 0.999` for `consecutive_samples = 4`).
* **When to use**: As a quality-gate check to flag and discard over-amplified or poorly mixed synthesized files.

### 15. Click/Glitch Detection (`detect_glitches`)
* **What it is**: Detects transient glitches or digital clicks based on sudden, sharp sample-to-sample transitions.
* **Why it's important**: Clicks and glitches can occur during voice synthesis due to model errors or alignment boundaries. They ruin audio cleanliness.
* **Implementation**: Scans the audio file to find indices where the absolute first-order derivative (difference between consecutive samples) exceeds a threshold:
  $$\text{Diff}_i = |x_i - x_{i-1}| \ge \text{threshold}$$
* **When to use**: To audit synthesized files for transient artifacts or speaker-stitching errors.

### 16. Duration Ratio Check (`check_duration_ratio`)
* **What it is**: Evaluates the ratio of the audio duration to the number of non-whitespace characters in the synthesis text.
* **Why it's important**: Speech synthesis models can fail catastrophically by dropping parts of the text (swallowing speech) or entering infinite loops (repeating sounds or generating endless silence). Abnormal duration ratios flag these failures.
* **Implementation**: Computes the ratio of speech duration to character count:
  $$\text{Duration Ratio} = \frac{\text{Duration (seconds)}}{\text{Character Count}}$$
* **When to use**: As an automated sanity check in production TTS systems to detect phrase-deletion or infinite-loop model failures.

### 17. DC Offset Detection (`dc_offset`)
* **What it is**: The average (mean) value of the audio signal's waveform amplitude. In a perfectly centered audio signal, the average amplitude is `0.0`.
* **Why it's important**: A non-zero average amplitude (DC offset) shifts the entire waveform away from zero. This reduces the available headroom before clipping occurs, can introduce audible clicks or pops at the beginning and end of playback, and can cause distortion or issues in downstream DSP or model components.
* **Implementation**: Calculates the arithmetic mean of all sample values in the signal:
  $$\text{DC Offset} = \frac{1}{N} \sum_{i=1}^{N} x_i$$
  If the signal is empty, it returns `0.0`.
* **When to use**: To validate that synthesized or recorded speech is properly zero-centered prior to downstream features extraction (like MFCC) or hardware playback, and to flag recording system malfunctions.

### 18. Crest Factor (`crest_factor`)
* **What it is**: The ratio of the peak amplitude of the signal to its Root-Mean-Square (RMS) value, expressed in decibels (dB):
  $$\text{Crest Factor (dB)} = 20 \log_{10} \left( \frac{|x_{\text{peak}}|}{x_{\text{RMS}}} \right)$$
* **Why it's important**: Natural human speech has a typical crest factor range (usually between 12 dB and 18 dB). An abnormally high crest factor indicates sudden clipping spikes, transient clicks, or pops. An abnormally low crest factor suggests over-compression, aggressive limiting, or unnatural flat-lining (like sustained tone or square-wave distortion).
* **Implementation**: Scans the signal to find the maximum absolute value ($|x_{\text{peak}}|$) and computes the RMS value. Returns an error if the RMS is zero ($< 10^{-10}$). Otherwise, returns the ratio in decibels.
* **When to use**: Use it to monitor the dynamic range and naturalness of generated speech, identifying clipping transients or over-processed flat-lined audio outputs.

### 19. Silence Padding (`silence_padding`)
* **What it is**: The duration (in seconds) of the silence segments at the very beginning (start padding) and the very end (end padding) of the audio signal.
* **Why it's important**: TTS applications and voice assistants require precise silence boundaries. Too little silence padding can lead to cut-off speech (clipping word boundaries), while too much silence padding makes responses feel laggy, unresponsive, or unnatural.
* **Implementation**: Runs windowed energy analysis (using a specified frame size, overlap, and energy threshold). The forward scan finds the first frame exceeding the threshold to determine start padding. The backward scan finds the last frame exceeding the threshold to determine end padding.
* **When to use**: To verify that synthesized TTS responses have acceptable and standardized start/end silence padding (e.g. 50ms to 200ms) before user delivery.

### 20. Spectral Flatness (`spectral_flatness`)
* **What it is**: A measure of the noise-like versus tonal quality of a signal, calculated frame-by-frame as the ratio of the geometric mean to the arithmetic mean of the power spectrum:
  $$\text{Spectral Flatness} = \frac{\exp \left( \frac{1}{N} \sum_{k=0}^{N-1} \ln S(k) \right)}{\frac{1}{N} \sum_{k=0}^{N-1} S(k)}$$
  A value close to `1.0` indicates a white noise-like flat spectrum, while a value close to `0.0` indicates a highly tonal, peaky spectrum.
* **Why it's important**: Natural speech consists of a mixture of tonal/voiced sounds (vowels, low flatness) and noise-like/unvoiced sounds (fricatives like "s", "f", high flatness). Spectral flatness contours track this structure. Constant high flatness indicates persistent static or background noise; constant low flatness indicates artificial metallic resonance or robotic speech.
* **Implementation**: Computes the power spectral density (PSD) per frame via FFT. Calculates the geometric mean and arithmetic mean across the FFT bins (using an epsilon factor to prevent zero values) and clamps the result between `0.0` and `1.0`.
* **When to use**: For frame-by-frame voiced/unvoiced voice activity analysis, detecting robotic/metallic voice artifacts, or detecting background hiss/white noise in TTS audio outputs.

### 21. Pairwise Preference Comparison (`compare_preference`)
* **What it is**: Compares two test model outputs against a baseline reference to declare which output is preferred based on a chosen metric (MCD, LSD, or SegSNR).
* **Why it's important**: Automates subjective comparison of model variants, producing a clear winning model.
* **Implementation**: Computes the selected metric for Model A and Model B against the reference, comparing them according to minimization (MCD, LSD) or maximization (SegSNR) rules.
* **When to use**: For automated A/B model testing and ranking of voice samples.

### 22. Model Rank Aggregation (`rank_models`)
* **What it is**: Ranks multiple models based on a weighted sum of normalized composite metrics.
* **Why it's important**: No single objective metric captures all aspects of voice quality. A multi-metric ranking engine provides a holistic evaluation rank.
* **Implementation**: Normalizes MCD, LSD, and SegSNR to a scale of $[0.0, 1.0]$ (where 0.0 is best and 1.0 is worst) and computes:
  $$\text{Composite Score} = w_{\text{MCD}} \cdot \bar{\text{MCD}} + w_{\text{LSD}} \cdot \bar{\text{LSD}} + w_{\text{SegSNR}} \cdot \bar{\text{SegSNR}}$$
* **When to use**: To automatically rank and select the best TTS model checkpoint in multi-metric optimization pipelines.

### 23. Audio Loaders (`load_wav`, `load_pcm`, `load_audio`)
* **What it is**: Utility functions to open, decode, and normalize speech signals.
* **Why it's important**: Audio models and DSP calculations require clean, normalized, single-channel floating-point arrays. These loaders standardized loading from both metadata-rich files (WAV) and raw headerless files (PCM).
* **Implementation**:
  * `load_wav`: Decodes signed 16/32-bit or floating-point WAV files, downmixes multiple channels, and normalizes samples to the range $[-1.0, 1.0]$.
  * `load_pcm`: Decodes raw signed 16-bit little-endian mono PCM files, normalizes samples to the range $[-1.0, 1.0]$.
  * `load_audio`: Automatically checks the file extension, loading WAV via `load_wav` and PCM/raw via `load_pcm` with a user-supplied fallback sample rate.
* **When to use**: Before running any DSP checks or similarity metrics on WAV or raw PCM audio files.

---

## Verification and Testing

`acoustix` features a complete suite of unit and integration tests (validating calculations against real LJ Speech audio clips).

Run tests:
```bash
cargo nextest run --status-level fail -p acoustix
```

---

## License

This project is licensed under the MIT License.
