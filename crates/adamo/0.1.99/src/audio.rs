//! Audio send (source-driven) — the audio analogue of the video `attach_*`
//! methods in [`robot`](crate::robot).
//!
//! Behind the `audio` feature, which **implies `video`** (the [`Robot`] handle
//! lives in the video module, and the prebuilt `libadamo` is built
//! audio-implies-video). Requires a `libadamo` built with the `audio` feature —
//! stock published prebuilts do not have the audio symbols yet, so until the
//! release pipeline restages one, build `adamo-c --features audio` locally and
//! point `ADAMO_LIB_DIR` at it (see `adamo-sys/prebuilt/README.md`).
//!
//! **Send only**, mirroring the video receive asymmetry (Rust/Python are
//! send-only; C/C++ and TS receive). The caller-fed push path
//! (`AudioTrack::send(pcm)`) is not built — see the phase-3 TODO in
//! `internal_docs/protocols/audio-streaming.md`.

use std::ffi::CString;

use crate::error::{Result, last_ffi_error};
use crate::robot::Robot;

/// Configuration for a source-driven audio track. Defaults mirror the robot
/// binary: Opus 64 kbps stereo, 48 kHz, 20 ms frames, FEC/DTX off.
#[derive(Debug, Clone)]
pub struct AudioOptions {
    /// `auto` | `alsa` | `pulse` | `pipeline` | `test`.
    pub source_type: String,
    /// ALSA/Pulse device string (e.g. `plughw:1,0`); `None` for other sources.
    pub device: Option<String>,
    /// GStreamer launch string, required when `source_type` is `pipeline`.
    pub pipeline: Option<String>,
    pub codec: String,
    pub bitrate_kbps: u32,
    pub sample_rate: u32,
    /// Requested channel count — a ceiling (a mono source stays mono).
    pub channels: u8,
    pub frame_ms: u32,
    /// Opus in-band FEC (LBRR). Only native consumers can decode it.
    pub inband_fec: bool,
    /// Discontinuous transmission — send nothing during silence.
    pub dtx: bool,
    /// Keep the robot running if this source is missing.
    pub allow_missing: bool,
}

impl Default for AudioOptions {
    fn default() -> Self {
        Self {
            source_type: "auto".to_string(),
            device: None,
            pipeline: None,
            codec: "opus".to_string(),
            bitrate_kbps: 64,
            sample_rate: 48_000,
            channels: 2,
            frame_ms: 20,
            inband_fec: false,
            dtx: false,
            allow_missing: false,
        }
    }
}

impl AudioOptions {
    pub fn with_source_type(mut self, s: impl Into<String>) -> Self {
        self.source_type = s.into();
        self
    }
    pub fn with_device(mut self, d: impl Into<String>) -> Self {
        self.device = Some(d.into());
        self
    }
    pub fn with_pipeline(mut self, p: impl Into<String>) -> Self {
        self.pipeline = Some(p.into());
        self
    }
    pub fn with_bitrate_kbps(mut self, b: u32) -> Self {
        self.bitrate_kbps = b;
        self
    }
    pub fn with_channels(mut self, c: u8) -> Self {
        self.channels = c;
        self
    }
    pub fn with_sample_rate(mut self, r: u32) -> Self {
        self.sample_rate = r;
        self
    }
    pub fn with_frame_ms(mut self, ms: u32) -> Self {
        self.frame_ms = ms;
        self
    }
    pub fn with_inband_fec(mut self, on: bool) -> Self {
        self.inband_fec = on;
        self
    }
    pub fn with_dtx(mut self, on: bool) -> Self {
        self.dtx = on;
        self
    }
    pub fn with_allow_missing(mut self, on: bool) -> Self {
        self.allow_missing = on;
        self
    }
}

/// FFI marshaling — owns the `CString`s backing the borrowed `char*` pointers
/// so they outlive the attach call. Mirrors `RawVideoOptions`.
struct RawAudioOptions {
    raw: adamo_sys::adamo_audio_options_t,
    _source_type: CString,
    _device: Option<CString>,
    _pipeline: Option<CString>,
    _codec: CString,
}

impl RawAudioOptions {
    fn new(options: &AudioOptions) -> Result<Self> {
        let source_type = CString::new(options.source_type.as_str())?;
        let device = options.device.as_deref().map(CString::new).transpose()?;
        let pipeline = options.pipeline.as_deref().map(CString::new).transpose()?;
        let codec = CString::new(options.codec.as_str())?;

        let mut raw = unsafe { adamo_sys::adamo_audio_options_default() };
        raw.source_type = source_type.as_ptr();
        raw.device = device
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null());
        raw.pipeline = pipeline
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null());
        raw.codec = codec.as_ptr();
        raw.bitrate_kbps = options.bitrate_kbps;
        raw.sample_rate = options.sample_rate;
        raw.channels = options.channels;
        raw.frame_ms = options.frame_ms;
        raw.inband_fec = options.inband_fec;
        raw.dtx = options.dtx;
        raw.allow_missing = options.allow_missing;

        Ok(Self {
            raw,
            _source_type: source_type,
            _device: device,
            _pipeline: pipeline,
            _codec: codec,
        })
    }

    fn as_ptr(&self) -> *const adamo_sys::adamo_audio_options_t {
        &self.raw
    }
}

impl Robot {
    /// Attach a microphone captured directly from ALSA (libasound, no
    /// GStreamer). Prefer a `plughw:` device so libasound resamples/reformats to
    /// the Opus input; pass `None` for the ALSA `default` device. `channels` is
    /// a ceiling (a mono mic stays mono). `frame_ms` must be a valid Opus frame
    /// size (2/5/10/20/40/60, where 2 denotes 2.5 ms).
    pub fn attach_audio_alsa(
        &mut self,
        name: &str,
        device: Option<&str>,
        bitrate_kbps: u32,
        channels: u8,
        sample_rate: u32,
        frame_ms: u32,
    ) -> Result<()> {
        let name = CString::new(name)?;
        let device = device.map(CString::new).transpose()?;
        let rc = unsafe {
            adamo_sys::adamo_robot_attach_audio_alsa(
                self.raw.as_ptr(),
                name.as_ptr(),
                device
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(std::ptr::null()),
                bitrate_kbps,
                channels,
                sample_rate,
                frame_ms,
            )
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }

    /// Attach a synthetic test-tone audio track (`audiotestsrc` via GStreamer).
    /// No hardware required — the loopback/CI source.
    pub fn attach_audio_test(&mut self, name: &str) -> Result<()> {
        let name = CString::new(name)?;
        let rc =
            unsafe { adamo_sys::adamo_robot_attach_audio_test(self.raw.as_ptr(), name.as_ptr()) };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }

    /// Attach an audio track backed by an arbitrary GStreamer pipeline string
    /// ending in raw audio (the robot appends the Opus encode chain).
    pub fn attach_audio_gst(&mut self, name: &str, pipeline: &str) -> Result<()> {
        let name = CString::new(name)?;
        let pipeline = CString::new(pipeline)?;
        let rc = unsafe {
            adamo_sys::adamo_robot_attach_audio_gst(
                self.raw.as_ptr(),
                name.as_ptr(),
                pipeline.as_ptr(),
            )
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }

    /// Attach an audio track using the full [`AudioOptions`] surface; its
    /// `source_type` selects the backend.
    pub fn attach_audio(&mut self, name: &str, options: &AudioOptions) -> Result<()> {
        let name = CString::new(name)?;
        let options = RawAudioOptions::new(options)?;
        let rc = unsafe {
            adamo_sys::adamo_robot_attach_audio_configured(
                self.raw.as_ptr(),
                name.as_ptr(),
                options.as_ptr(),
            )
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }
}
