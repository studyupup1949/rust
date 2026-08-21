//! DSP math primitives — pure numeric functions for audio signal processing.
//!
//! These are stateless, allocation-free helpers used by audio engines (e.g.,
//! [dhvani](https://crates.io/crates/dhvani)) for decibel conversion, filter
//! design, waveform generation, and other common DSP tasks.
//!
//! All functions operate on `f32` or `f64` — no audio buffers, no allocations,
//! no state. This makes them suitable for real-time (`#[inline]`) hot paths.
//!
//! # Quick Start
//!
//! ```rust
//! use abaco::dsp;
//!
//! // dB ↔ amplitude
//! let db = dsp::amplitude_to_db(0.5);       // ≈ -6.02
//! let amp = dsp::db_to_amplitude(-6.0);      // ≈ 0.501
//!
//! // MIDI ↔ frequency
//! let freq = dsp::midi_to_freq(69.0);         // 440.0 Hz (A4)
//! let note = dsp::freq_to_midi(440.0);       // 69.0
//!
//! // Envelope time constant
//! let coeff = dsp::time_constant(10.0, 44100); // 10ms attack/release
//!
//! // Constant-power panning
//! let (l, r) = dsp::constant_power_pan(0.0);  // center: both ≈ 0.707
//! ```

use std::f64::consts::TAU;

// Precomputed constants for fast dB ↔ amplitude conversion.
// 20 / ln(10) ≈ 8.685889638  (amplitude → dB via ln)
// ln(10) / 20 ≈ 0.115129254  (dB → amplitude via exp)
// ln(10) / 40 ≈ 0.057564627  (dB → gain factor via exp)
const DB_SCALE_F32: f32 = 20.0 / std::f32::consts::LN_10; // amplitude_to_db
const DB_EXP_F32: f32 = std::f32::consts::LN_10 / 20.0; // db_to_amplitude
const DB_SCALE_F64: f64 = 20.0 / std::f64::consts::LN_10;
const DB_EXP_F64: f64 = std::f64::consts::LN_10 / 20.0;
const DB_GAIN_EXP_F64: f64 = std::f64::consts::LN_10 / 40.0;

// ── Decibel conversions ──────────────────────────────────────────────────────

/// Convert linear amplitude to decibels (f32).
///
/// Returns `f32::NEG_INFINITY` for amplitudes ≤ 0.
///
/// Uses `20/ln(10) × ln(x)` instead of `20 × log10(x)` to avoid the
/// `log10` → `ln` indirection on most architectures.
#[inline]
pub fn amplitude_to_db(amplitude: f32) -> f32 {
    if amplitude <= 0.0 {
        return f32::NEG_INFINITY;
    }
    DB_SCALE_F32 * amplitude.ln()
}

/// Convert decibels to linear amplitude (f32).
///
/// Uses `exp(dB × ln(10)/20)` instead of `10^(dB/20)` — avoids the
/// general-purpose `powf` in favour of a single `exp`.
#[inline]
pub fn db_to_amplitude(db: f32) -> f32 {
    (db * DB_EXP_F32).exp()
}

/// Convert linear amplitude to decibels (f64).
///
/// Returns `f64::NEG_INFINITY` for amplitudes ≤ 0.
#[inline]
pub fn amplitude_to_db_f64(amplitude: f64) -> f64 {
    if amplitude <= 0.0 {
        return f64::NEG_INFINITY;
    }
    DB_SCALE_F64 * amplitude.ln()
}

/// Convert decibels to linear amplitude (f64).
#[inline]
pub fn db_to_amplitude_f64(db: f64) -> f64 {
    (db * DB_EXP_F64).exp()
}

/// Convert dB to a linear gain factor for filter coefficient design.
///
/// Uses `10^(dB / 40)` — the standard formula from the Bristow-Johnson
/// Audio EQ Cookbook for peaking and shelving filter gain.
///
/// Implemented as `exp(dB × ln(10)/40)` for speed.
#[inline]
pub fn db_gain_factor(db: f64) -> f64 {
    (db * DB_GAIN_EXP_F64).exp()
}

// ── MIDI ↔ Frequency ─────────────────────────────────────────────────────────

/// A4 reference frequency in Hz (concert pitch).
pub const A4_FREQUENCY: f64 = 440.0;
/// A4 MIDI note number.
pub const A4_MIDI_NOTE: f64 = 69.0;
/// Semitones per octave.
pub const SEMITONES_PER_OCTAVE: f64 = 12.0;

/// Convert a MIDI note number to frequency in Hz (12-TET, A4 = 440 Hz).
///
/// Accepts fractional notes for pitch bends and microtuning.
/// Uses `exp2` instead of `powf(2, x)` for a direct base-2 exponential.
#[inline]
pub fn midi_to_freq(note: f64) -> f64 {
    A4_FREQUENCY * ((note - A4_MIDI_NOTE) / SEMITONES_PER_OCTAVE).exp2()
}

/// Convert a frequency in Hz to a (possibly fractional) MIDI note number.
///
/// Returns `f64::NEG_INFINITY` for frequencies ≤ 0.
#[inline]
pub fn freq_to_midi(freq: f64) -> f64 {
    if freq <= 0.0 {
        return f64::NEG_INFINITY;
    }
    A4_MIDI_NOTE + SEMITONES_PER_OCTAVE * (freq / A4_FREQUENCY).log2()
}

// ── Envelope / dynamics ──────────────────────────────────────────────────────

/// Compute a one-pole exponential smoothing coefficient from a time in milliseconds.
///
/// Used for attack/release envelope followers in compressors and limiters.
/// The returned coefficient `c` is applied as:
///
/// ```text
/// envelope = c * envelope + (1 - c) * input
/// ```
///
/// Larger `time_ms` → `c` closer to 1.0 → slower response.
#[inline]
pub fn time_constant(time_ms: f32, sample_rate: u32) -> f32 {
    let samples = (time_ms * 0.001 * sample_rate as f32).max(1.0);
    (-1.0f32 / samples).exp()
}

// ── Sample utilities ─────────────────────────────────────────────────────────

/// Sanitize a sample: replace NaN or infinity with 0.0.
#[inline]
pub fn sanitize_sample(s: f32) -> f32 {
    if s.is_finite() { s } else { 0.0 }
}

// ── Angular frequency ────────────────────────────────────────────────────────

/// Compute the normalized angular frequency `ω₀ = 2π × f₀ / fs`.
///
/// Used in biquad filter coefficient design (Bristow-Johnson EQ Cookbook).
#[inline]
pub fn angular_frequency(freq_hz: f64, sample_rate: f64) -> f64 {
    TAU * freq_hz / sample_rate
}

// ── Waveform helpers ─────────────────────────────────────────────────────────

/// PolyBLEP correction value for a discontinuity at phase boundary.
///
/// Reduces aliasing in saw, square, and pulse waveforms by smoothing
/// the discontinuity over one sample on each side.
///
/// - `t` — normalized phase (0.0–1.0)
/// - `dt` — phase increment per sample (`freq / sample_rate`)
#[inline]
pub fn poly_blep(t: f64, dt: f64) -> f64 {
    if dt <= 0.0 {
        return 0.0;
    }
    if t < dt {
        // Just past discontinuity
        let t = t / dt;
        2.0 * t - t * t - 1.0
    } else if t > 1.0 - dt {
        // Just before discontinuity
        let t = (t - 1.0) / dt;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}

// ── Panning / crossfade ──────────────────────────────────────────────────────

/// Compute constant-power pan gains for a stereo signal.
///
/// - `pan` — position in `[-1.0, +1.0]` (left to right)
///
/// Returns `(left_gain, right_gain)` where `left² + right² ≈ 1.0`.
///
/// Uses the sin/cos law: `θ = (pan + 1) × π/4`, then `L = cos(θ)`, `R = sin(θ)`.
/// Computes both via a single `sin_cos` call.
#[inline]
pub fn constant_power_pan(pan: f32) -> (f32, f32) {
    let pan = pan.clamp(-1.0, 1.0);
    let theta = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
    let (sin, cos) = theta.sin_cos();
    (cos, sin)
}

/// Compute equal-power crossfade gains at position `t` ∈ `[0.0, 1.0]`.
///
/// Returns `(gain_a, gain_b)` where `a` fades out and `b` fades in,
/// maintaining constant energy: `gain_a² + gain_b² ≈ 1.0`.
/// Computes both via a single `sin_cos` call.
#[inline]
pub fn equal_power_crossfade(t: f32) -> (f32, f32) {
    let angle = t.clamp(0.0, 1.0) * std::f32::consts::FRAC_PI_2;
    let (sin, cos) = angle.sin_cos();
    (cos, sin)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── dB conversions ───────────────────────────────────────────────────

    #[test]
    fn db_unity() {
        assert!(amplitude_to_db(1.0).abs() < 0.01);
        assert!((db_to_amplitude(0.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn db_half_amplitude() {
        assert!((amplitude_to_db(0.5) - (-6.02)).abs() < 0.1);
        assert!((db_to_amplitude(-6.02) - 0.5).abs() < 0.01);
    }

    #[test]
    fn db_zero_amplitude() {
        assert!(amplitude_to_db(0.0).is_infinite());
        assert!(amplitude_to_db(-1.0).is_infinite());
    }

    #[test]
    fn db_f64_matches_f32() {
        for amp in [0.1, 0.5, 1.0, 2.0] {
            let db32 = amplitude_to_db(amp as f32) as f64;
            let db64 = amplitude_to_db_f64(amp);
            assert!((db32 - db64).abs() < 0.01);
        }
    }

    #[test]
    fn db_gain_factor_peaking() {
        // 12 dB gain → factor ≈ 1.995 (10^(12/40))
        let f = db_gain_factor(12.0);
        assert!((f - 1.995).abs() < 0.01);
    }

    // ── MIDI ↔ frequency ─────────────────────────────────────────────────

    #[test]
    fn midi_a4() {
        assert!((midi_to_freq(69.0) - 440.0).abs() < 0.01);
    }

    #[test]
    fn midi_c4() {
        assert!((midi_to_freq(60.0) - 261.63).abs() < 0.1);
    }

    #[test]
    fn midi_roundtrip() {
        for note in [0.0, 36.0, 60.0, 69.0, 84.0, 127.0] {
            let freq = midi_to_freq(note);
            let back = freq_to_midi(freq);
            assert!(
                (back - note).abs() < 1e-10,
                "roundtrip failed for note {note}"
            );
        }
    }

    #[test]
    fn freq_to_midi_zero() {
        assert!(freq_to_midi(0.0).is_infinite());
    }

    // ── time constant ────────────────────────────────────────────────────

    #[test]
    fn time_constant_fast() {
        let c = time_constant(0.01, 44100); // ~0.01ms ≈ instant
        assert!(c < 0.5, "near-instant should have low coefficient");
    }

    #[test]
    fn time_constant_slow() {
        let c = time_constant(1000.0, 44100); // 1 second
        assert!(c > 0.99, "slow release should have high coefficient");
    }

    // ── sanitize ─────────────────────────────────────────────────────────

    #[test]
    fn sanitize_finite() {
        assert_eq!(sanitize_sample(0.5), 0.5);
    }

    #[test]
    fn sanitize_nan() {
        assert_eq!(sanitize_sample(f32::NAN), 0.0);
    }

    #[test]
    fn sanitize_inf() {
        assert_eq!(sanitize_sample(f32::INFINITY), 0.0);
    }

    // ── angular frequency ────────────────────────────────────────────────

    #[test]
    fn angular_freq_1khz() {
        let w0 = angular_frequency(1000.0, 44100.0);
        let expected = TAU * 1000.0 / 44100.0;
        assert!((w0 - expected).abs() < 1e-12);
    }

    // ── poly_blep ────────────────────────────────────────────────────────

    #[test]
    fn poly_blep_mid_phase_zero() {
        // In the middle of a cycle, correction should be zero
        assert_eq!(poly_blep(0.5, 0.01), 0.0);
    }

    #[test]
    fn poly_blep_near_discontinuity() {
        let dt = 0.01;
        // Just past wrap: t < dt
        let v = poly_blep(dt * 0.5, dt);
        assert!(v != 0.0, "should correct near discontinuity");
    }

    #[test]
    fn poly_blep_zero_dt() {
        assert_eq!(poly_blep(0.0, 0.0), 0.0);
    }

    // ── panning ──────────────────────────────────────────────────────────

    #[test]
    fn pan_center() {
        let (l, r) = constant_power_pan(0.0);
        assert!((l - r).abs() < 0.01);
        assert!((l - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.01);
    }

    #[test]
    fn pan_full_left() {
        let (l, r) = constant_power_pan(-1.0);
        assert!((l - 1.0).abs() < 0.01);
        assert!(r.abs() < 0.01);
    }

    #[test]
    fn pan_full_right() {
        let (l, r) = constant_power_pan(1.0);
        assert!(l.abs() < 0.01);
        assert!((r - 1.0).abs() < 0.01);
    }

    #[test]
    fn pan_constant_power() {
        for p in [-1.0, -0.5, 0.0, 0.25, 0.5, 1.0] {
            let (l, r) = constant_power_pan(p);
            let power = l * l + r * r;
            assert!((power - 1.0).abs() < 0.01, "power={power} at pan={p}");
        }
    }

    // ── crossfade ────────────────────────────────────────────────────────

    #[test]
    fn crossfade_endpoints() {
        let (a, b) = equal_power_crossfade(0.0);
        assert!((a - 1.0).abs() < 0.01);
        assert!(b.abs() < 0.01);

        let (a, b) = equal_power_crossfade(1.0);
        assert!(a.abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
    }

    #[test]
    fn crossfade_constant_energy() {
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let (a, b) = equal_power_crossfade(t);
            let energy = a * a + b * b;
            assert!((energy - 1.0).abs() < 0.01, "energy={energy} at t={t}");
        }
    }
}
