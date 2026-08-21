# Abaco Roadmap

## DSP Module Expansion

- [ ] Window functions — Hann, Hamming, Blackman, Kaiser (currently inline in dhvani's FFT/STFT)
- [ ] Interpolation math — linear lerp, cubic, windowed sinc kernel (used in resamplers, delay lines)
- [ ] Chromagram helpers — `freq_to_pitch_class`, `C0` constant, semitone mapping (currently inline in dhvani analysis)

## Audio Unit Conversions

- [ ] Add audio-specific units to `UnitRegistry`: dBFS, samples, BPM, semitones
- [ ] BPM ↔ Hz conversion (`registry.convert(120.0, "bpm", "Hz")`)
- [ ] Samples ↔ milliseconds (sample-rate-aware)

## Ecosystem Rollout

- [ ] Audit shruti for duplicated math that should use `abaco::dsp`
- [ ] Audit jalwa for duplicated math
- [ ] Audit tarang for duplicated math
- [ ] Audit aethersafta for duplicated math
- [ ] Standardize all Agnos projects on abaco for shared math
