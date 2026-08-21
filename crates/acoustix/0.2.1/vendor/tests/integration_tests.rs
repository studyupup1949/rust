#[cfg(feature = "wav")]
use acoustix::quality::{log_spectral_distance, segmental_snr};
#[cfg(feature = "wav")]
use acoustix::ranking::{MetricType, ModelEvaluation, compare_preference, rank_models};
#[cfg(feature = "wav")]
use acoustix::similarity::compute_mcd_between_signals;

#[test]
fn test_wer_on_metadata_file() {
    use acoustix::evaluation::word_error_rate;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open("tests/files/metadata.csv").expect("Failed to open metadata.csv");
    let reader = BufReader::new(file);

    // Read the third line
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
    let line3 = &lines[2]; // 0-indexed third line

    let parts: Vec<&str> = line3.split('|').collect();
    assert_eq!(parts.len(), 3);

    let reference = parts[1]; // Transcription: "Chapter 8... Part 5"
    let hypothesis = parts[2]; // Normalized Transcription: "Chapter eight... Part five"

    let wer = word_error_rate(reference, hypothesis).unwrap();
    // 2 substitutions ("8" -> "eight", "5" -> "five") out of 19 reference words.
    // WER = 2 / 19 ≈ 0.105263
    assert!((wer - 0.105263).abs() < 1e-4);
}

#[test]
fn test_cosine_similarity_and_acc() {
    use acoustix::evaluation::{cosine_similarity, speaker_attribution_accuracy};

    // Test cosine similarity of some mock speaker embedding vectors
    let speaker_ref = vec![0.5, -0.5, 0.5, 0.5];
    let speaker_same = vec![0.5, -0.5, 0.5, 0.5];
    let speaker_diff = vec![-0.5, 0.5, -0.5, -0.5]; // Completely opposite

    let sim_same = cosine_similarity(&speaker_ref, &speaker_same).unwrap();
    let sim_diff = cosine_similarity(&speaker_ref, &speaker_diff).unwrap();

    assert!((sim_same - 1.0).abs() < 1e-5);
    assert!((sim_diff - (-1.0)).abs() < 1e-5);

    // Test Speaker Attribution Accuracy (ACC)
    let actual = vec!["spk1".to_string(), "spk2".to_string(), "spk1".to_string()];
    let predicted = vec!["spk1".to_string(), "spk1".to_string(), "spk1".to_string()];
    let acc = speaker_attribution_accuracy(&actual, &predicted).unwrap();
    assert!((acc - 0.666667).abs() < 1e-4);
}

#[cfg(feature = "wav")]
mod wav_tests {
    use super::*;
    use acoustix::similarity::load_wav;

    #[test]
    fn test_real_wav_loading() {
        let path = "tests/files/LJ050-0002.wav";
        let (samples, sr) = load_wav(path).expect("Failed to load WAV file");

        assert_eq!(sr, 22050);
        assert!(samples.len() > 0);
        // Samples should be normalized in [-1.0, 1.0]
        for &sample in &samples {
            assert!(sample >= -1.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_quality_on_real_files_identical() {
        let path = "tests/files/LJ050-0002.wav";
        let (samples, _) = load_wav(path).unwrap();

        let snr = segmental_snr(&samples, &samples, 512, 256, -10.0, 35.0, 1e-4).unwrap();
        let lsd = log_spectral_distance(&samples, &samples, 512, 256).unwrap();

        assert!((snr - 35.0).abs() < 1e-5);
        assert!(lsd.abs() < 1e-5);
    }

    #[test]
    fn test_quality_on_real_files_different() {
        let (samples1, _) = load_wav("tests/files/LJ050-0002.wav").unwrap();
        let (samples2, _) = load_wav("tests/files/LJ050-0004.wav").unwrap();

        let snr = segmental_snr(&samples1, &samples2, 512, 256, -10.0, 35.0, 1e-4).unwrap();
        let lsd = log_spectral_distance(&samples1, &samples2, 512, 256).unwrap();

        assert!(snr < 35.0);
        assert!(lsd > 0.0);
    }

    #[test]
    fn test_similarity_on_real_files_identical() {
        let path = "tests/files/LJ050-0002.wav";
        let (samples, sr) = load_wav(path).unwrap();

        let mcd =
            compute_mcd_between_signals(&samples, &samples, sr, 512, 256, 20, 13, true).unwrap();
        assert!(mcd.abs() < 1e-5);
    }

    #[test]
    fn test_similarity_on_real_files_different() {
        let (samples1, sr1) = load_wav("tests/files/LJ050-0002.wav").unwrap();
        let (samples2, _) = load_wav("tests/files/LJ050-0004.wav").unwrap();

        // Since they are different utterances, MCD should be positive and non-zero
        let mcd =
            compute_mcd_between_signals(&samples1, &samples2, sr1, 512, 256, 20, 13, true).unwrap();
        assert!(mcd > 0.0);
    }

    #[test]
    fn test_preference_on_real_files() {
        let (ref_samples, sr) = load_wav("tests/files/LJ050-0002.wav").unwrap();
        let (test_a, _) = load_wav("tests/files/LJ050-0002.wav").unwrap(); // Perfect match
        let (test_b, _) = load_wav("tests/files/LJ050-0004.wav").unwrap(); // Different

        let pref = compare_preference(
            &ref_samples,
            &test_a,
            &test_b,
            MetricType::Mcd,
            sr,
            512,
            256,
        )
        .unwrap();
        assert_eq!(pref.preferred_model, "A");
        assert!(pref.score_a < pref.score_b);
    }

    #[test]
    fn test_ranking_on_real_files() {
        let (ref_samples, sr) = load_wav("tests/files/LJ050-0002.wav").unwrap();
        let (test_a, _) = load_wav("tests/files/LJ050-0002.wav").unwrap(); // Perfect match
        let (test_b, _) = load_wav("tests/files/LJ050-0004.wav").unwrap(); // Different
        let (test_c, _) = load_wav("tests/files/LJ050-0005.wav").unwrap(); // Different

        let mcd_a =
            compute_mcd_between_signals(&ref_samples, &test_a, sr, 512, 256, 20, 13, true).unwrap();
        let mcd_b =
            compute_mcd_between_signals(&ref_samples, &test_b, sr, 512, 256, 20, 13, true).unwrap();
        let mcd_c =
            compute_mcd_between_signals(&ref_samples, &test_c, sr, 512, 256, 20, 13, true).unwrap();

        let lsd_a = log_spectral_distance(&ref_samples, &test_a, 512, 256).unwrap();
        let lsd_b = log_spectral_distance(&ref_samples, &test_b, 512, 256).unwrap();
        let lsd_c = log_spectral_distance(&ref_samples, &test_c, 512, 256).unwrap();

        let snr_a = segmental_snr(&ref_samples, &test_a, 512, 256, -10.0, 35.0, 1e-4).unwrap();
        let snr_b = segmental_snr(&ref_samples, &test_b, 512, 256, -10.0, 35.0, 1e-4).unwrap();
        let snr_c = segmental_snr(&ref_samples, &test_c, 512, 256, -10.0, 35.0, 1e-4).unwrap();

        let evals = vec![
            ModelEvaluation {
                model_name: "Model_A".to_string(),
                mcd: mcd_a,
                lsd: lsd_a,
                seg_snr: snr_a,
            },
            ModelEvaluation {
                model_name: "Model_B".to_string(),
                mcd: mcd_b,
                lsd: lsd_b,
                seg_snr: snr_b,
            },
            ModelEvaluation {
                model_name: "Model_C".to_string(),
                mcd: mcd_c,
                lsd: lsd_c,
                seg_snr: snr_c,
            },
        ];

        let rankings = rank_models(&evals, 1.0, 1.0, 1.0);

        // Model_A (which is identical to reference) must rank 1st
        assert_eq!(rankings[0].model_name, "Model_A");
        assert_eq!(rankings[0].rank, 1);
    }

    #[test]
    fn test_advanced_f0_metrics_real_audio() {
        use acoustix::advanced::{f0_correlation, f0_rmse, track_f0};

        let (samples, sr) = load_wav("tests/files/LJ050-0002.wav").unwrap();

        // Track F0 pitch contour on the real wave
        let f0 = track_f0(&samples, sr, 512, 256, 50.0, 500.0).unwrap();
        assert!(f0.len() > 0);

        // Compute metrics against itself (perfect match)
        let rmse = f0_rmse(&f0, &f0).unwrap();
        let corr = f0_correlation(&f0, &f0).unwrap();

        assert!(rmse.abs() < 1e-4);
        assert!((corr - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_advanced_defects_and_duration_real_audio() {
        use acoustix::advanced::{check_duration_ratio, detect_clipping, detect_glitches};

        let (samples, sr) = load_wav("tests/files/LJ050-0002.wav").unwrap();

        // Verify LJ Speech clip is clean (no clipping at 0.99 threshold)
        let clip = detect_clipping(&samples, 0.99, 4);
        assert_eq!(clip, 0);

        // Verify click/glitch index list is empty or minimal (0.99 delta amplitude threshold)
        let glitches = detect_glitches(&samples, 0.99);
        assert!(glitches.is_empty());

        // Verify speaking duration ratio (seconds per character)
        let text = "The Warren Commission Report.";
        let ratio = check_duration_ratio(samples.len(), sr, text).unwrap();

        // Duration should be around 1.62s. 29 chars. Ratio ≈ 0.056s/char.
        assert!(ratio > 0.01 && ratio < 0.2);
    }

    #[test]
    fn test_advanced_frechet_distance_real_audio() {
        use acoustix::advanced::frechet_distance;
        use acoustix::similarity::extract_mfcc;

        let (samples, sr) = load_wav("tests/files/LJ050-0002.wav").unwrap();

        // Extract MFCCs
        let mfccs = extract_mfcc(&samples, sr, 512, 256, 20, 13).unwrap();

        let set_a = vec![mfccs.clone()];
        let set_b = vec![mfccs.clone()];

        let fd = frechet_distance(&set_a, &set_b).unwrap();
        assert!(fd.abs() < 1e-5);
    }

    #[test]
    fn test_advanced_new_checks_real_audio() {
        use acoustix::advanced::{
            band_spectral_distance, crest_factor, dc_offset, silence_padding, spectral_flatness,
        };

        let (samples, sr) = load_wav("tests/files/LJ050-0002.wav").unwrap();

        // 1. DC offset should be very close to 0 for a clean wav file
        let dc = dc_offset(&samples);
        assert!(dc.abs() < 0.05);

        // 2. Crest factor should be within typical range (e.g. 5 - 30 dB) for natural speech
        let cf = crest_factor(&samples).unwrap();
        assert!(cf > 5.0 && cf < 30.0);

        // 3. Silence padding should return valid times
        let (start_pad, end_pad) = silence_padding(&samples, sr, 512, 256, 1e-4).unwrap();
        assert!(start_pad >= 0.0);
        assert!(end_pad >= 0.0);

        // 4. Band spectral distance of identical signals should be 0.0
        let bsd = band_spectral_distance(&samples, &samples, sr, 512, 256, 300.0, 3400.0).unwrap();
        assert!(bsd.abs() < 1e-4);

        // 5. Spectral flatness should return scores between 0.0 and 1.0
        let flatness = spectral_flatness(&samples, 512, 256).unwrap();
        assert!(flatness.len() > 0);
        for &f in &flatness {
            assert!(f >= 0.0 && f <= 1.0);
        }
    }
}
