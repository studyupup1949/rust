use acoustix::advanced::{
    band_spectral_distance, check_duration_ratio, crest_factor, dc_offset, detect_clipping,
    detect_glitches, f0_correlation, f0_rmse, frechet_distance, silence_padding, spectral_flatness,
    track_f0,
};
use acoustix::evaluation::{character_error_rate, word_error_rate};
use acoustix::quality::{log_spectral_distance, segmental_snr};
use acoustix::similarity::{compute_mcd_between_signals, extract_mfcc, load_audio};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Acoustix Speech Validation CLI");
        println!("=============================");
        println!("Usage:");
        println!(
            "  cargo run --example evaluate <reference_audio> <test_audio> [ref_text] [hyp_text]"
        );
        println!("\nExample:");
        println!(
            "  cargo run --example evaluate tests/files/LJ050-0002.wav tests/files/LJ050-0002.wav"
        );
        return;
    }

    let ref_path = &args[1];
    let test_path = &args[2];

    // Load reference audio
    let (ref_sig, sr_ref) = match load_audio(ref_path, 22050) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error loading reference audio: {}", e);
            std::process::exit(1);
        }
    };

    // Load test audio
    let (test_sig, sr_test) = match load_audio(test_path, 22050) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error loading test audio: {}", e);
            std::process::exit(1);
        }
    };

    let sample_rate = sr_ref.min(sr_test);
    println!("Evaluation Sample Rate: {} Hz", sample_rate);
    println!("---------------------------------------");

    // 1. Calculate Segmental SNR
    match segmental_snr(&ref_sig, &test_sig, 512, 256, -10.0, 35.0, 1e-4) {
        Ok(snr) => println!("Segmental SNR:          {:.4} dB", snr),
        Err(e) => eprintln!("Error computing SegSNR: {}", e),
    }

    // 2. Calculate Log Spectral Distance (LSD)
    match log_spectral_distance(&ref_sig, &test_sig, 512, 256) {
        Ok(lsd) => println!("Log Spectral Distance:  {:.4} dB", lsd),
        Err(e) => eprintln!("Error computing LSD:    {}", e),
    }

    // 2b. Calculate Band-Specific Log Spectral Distance (BSD, e.g. 300Hz - 3400Hz)
    match band_spectral_distance(&ref_sig, &test_sig, sample_rate, 512, 256, 300.0, 3400.0) {
        Ok(bsd) => println!("Band-Specific LSD:      {:.4} dB (300-3400 Hz)", bsd),
        Err(e) => eprintln!("Error computing BSD:    {}", e),
    }

    // 3. Calculate Mel Cepstral Distortion (MCD)
    match compute_mcd_between_signals(&ref_sig, &test_sig, sample_rate, 512, 256, 20, 13, true) {
        Ok(mcd) => println!("Mel Cepstral Distortion: {:.4} dB", mcd),
        Err(e) => eprintln!("Error computing MCD:    {}", e),
    }

    // 4. Calculate F0 Pitch Contour Metrics
    let f0_ref = track_f0(&ref_sig, sample_rate, 512, 256, 50.0, 500.0);
    let f0_test = track_f0(&test_sig, sample_rate, 512, 256, 50.0, 500.0);
    if let (Ok(ref_f0), Ok(test_f0)) = (f0_ref, f0_test) {
        if let Ok(rmse) = f0_rmse(&ref_f0, &test_f0) {
            println!("F0 Pitch Contour RMSE:  {:.4} Hz", rmse);
        }
        if let Ok(corr) = f0_correlation(&ref_f0, &test_f0) {
            println!("F0 Pitch Correlation:   {:.4}", corr);
        }
    }

    // 5. Calculate Frechet Distance (on extracted MFCC sets)
    let mfcc_ref = extract_mfcc(&ref_sig, sample_rate, 512, 256, 20, 13);
    let mfcc_test = extract_mfcc(&test_sig, sample_rate, 512, 256, 20, 13);
    if let (Ok(ref_mfccs), Ok(test_mfccs)) = (mfcc_ref, mfcc_test) {
        let set_a = vec![ref_mfccs];
        let set_b = vec![test_mfccs];
        if let Ok(fd) = frechet_distance(&set_a, &set_b) {
            println!("Fréchet MFCC Distance:  {:.4}", fd);
        }
    }

    // 6. Defect, Glitch, and DSP Signal Quality Checks
    let ref_clip = detect_clipping(&ref_sig, 0.999, 4);
    let test_clip = detect_clipping(&test_sig, 0.999, 4);
    let ref_glitches = detect_glitches(&ref_sig, 0.95).len();
    let test_glitches = detect_glitches(&test_sig, 0.95).len();

    let dc_ref = dc_offset(&ref_sig);
    let dc_test = dc_offset(&test_sig);

    let cf_ref = crest_factor(&ref_sig).unwrap_or(0.0);
    let cf_test = crest_factor(&test_sig).unwrap_or(0.0);

    let (sp_ref_start, sp_ref_end) =
        silence_padding(&ref_sig, sample_rate, 512, 256, 1e-4).unwrap_or((0.0, 0.0));
    let (sp_test_start, sp_test_end) =
        silence_padding(&test_sig, sample_rate, 512, 256, 1e-4).unwrap_or((0.0, 0.0));

    let flat_ref = spectral_flatness(&ref_sig, 512, 256).unwrap_or_default();
    let flat_test = spectral_flatness(&test_sig, 512, 256).unwrap_or_default();
    let mean_flat_ref = if flat_ref.is_empty() {
        0.0
    } else {
        flat_ref.iter().sum::<f32>() / flat_ref.len() as f32
    };
    let mean_flat_test = if flat_test.is_empty() {
        0.0
    } else {
        flat_test.iter().sum::<f32>() / flat_test.len() as f32
    };

    println!("---------------------------------------");
    println!(
        "Clipping Instances (Ref/Test): {} / {}",
        ref_clip, test_clip
    );
    println!(
        "Glitch Clicks Detected (Ref/Test): {} / {}",
        ref_glitches, test_glitches
    );
    println!(
        "DC Offset (Ref/Test):           {:.6} / {:.6}",
        dc_ref, dc_test
    );
    println!(
        "Crest Factor (Ref/Test):        {:.4} dB / {:.4} dB",
        cf_ref, cf_test
    );
    println!(
        "Silence Padding - Start (Ref/Test): {:.4}s / {:.4}s",
        sp_ref_start, sp_test_start
    );
    println!(
        "Silence Padding - End (Ref/Test):   {:.4}s / {:.4}s",
        sp_ref_end, sp_test_end
    );
    println!(
        "Mean Spectral Flatness (Ref/Test):  {:.4} / {:.4}",
        mean_flat_ref, mean_flat_test
    );

    // 7. Calculate Text & Duration Metrics if provided
    if args.len() >= 5 {
        let ref_text = &args[3];
        let hyp_text = &args[4];
        println!("---------------------------------------");
        println!("Reference Text:  \"{}\"", ref_text);
        println!("Hypothesis Text: \"{}\"", hyp_text);
        println!("---------------------------------------");

        match word_error_rate(ref_text, hyp_text) {
            Ok(wer) => println!(
                "Word Error Rate (WER):      {:.4} ({:.2}%)",
                wer,
                wer * 100.0
            ),
            Err(e) => eprintln!("Error computing WER:        {}", e),
        }

        match character_error_rate(ref_text, hyp_text) {
            Ok(cer) => println!(
                "Character Error Rate (CER): {:.4} ({:.2}%)",
                cer,
                cer * 100.0
            ),
            Err(e) => eprintln!("Error computing CER:        {}", e),
        }

        if let Ok(ratio_ref) = check_duration_ratio(ref_sig.len(), sample_rate, ref_text) {
            println!("Speaking Rate - Ref:        {:.4} sec/char", ratio_ref);
        }
        if let Ok(ratio_test) = check_duration_ratio(test_sig.len(), sample_rate, hyp_text) {
            println!("Speaking Rate - Test:       {:.4} sec/char", ratio_test);
        }
    }
}
