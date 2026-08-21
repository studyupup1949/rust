//! ACE Player — High-Performance Rust Media Engine SDK
//!
//! Memory-safe, high-concurrency media playback engine.
//! Designed to deliver ACE-generated content to enterprise clients.
//!
//! # Architecture
//! - `decoder` — video + audio decoding (symphonia + ffmpeg-next)
//! - `sync`    — AV synchronization (tokio-based, race-condition-free)
//! - `output`  — native + WebAssembly rendering targets
//! - `drm`     — DRM module (Phase 2: Widevine)

pub mod decoder;
pub mod sync;
pub mod output;
pub mod drm;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Playback configuration for a media session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    /// Source URL or local file path
    pub source: String,
    /// Target width (default: 1920)
    pub width: u32,
    /// Target height (default: 1080)
    pub height: u32,
    /// Volume 0.0–1.0 (default: 1.0)
    pub volume: f32,
    /// Loop playback
    pub looping: bool,
    /// Hardware acceleration enabled
    pub hw_accel: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            source: String::new(),
            width: 1920,
            height: 1080,
            volume: 1.0,
            looping: false,
            hw_accel: true,
        }
    }
}

/// Core ACE Player — manages decode, sync, and output lifecycle.
pub struct AcePlayer {
    config: PlayerConfig,
}

impl AcePlayer {
    pub fn new(config: PlayerConfig) -> Self {
        Self { config }
    }

    /// Load and prepare a media source for playback.
    pub async fn load(&mut self) -> Result<MediaInfo> {
        decoder::probe(&self.config.source).await
    }

    /// Start playback (non-blocking, tokio task).
    pub async fn play(&self) -> Result<()> {
        let config = self.config.clone();
        tokio::spawn(async move {
            if let Err(e) = _run_playback(config).await {
                tracing::error!("Playback error: {e}");
            }
        });
        Ok(())
    }
}

async fn _run_playback(config: PlayerConfig) -> Result<()> {
    let (video_tx, video_rx) = tokio::sync::mpsc::channel(32);
    let (audio_tx, audio_rx) = tokio::sync::mpsc::channel(32);

    // Spawn decode tasks in parallel (fearless concurrency)
    let source = config.source.clone();
    let decode_video = tokio::spawn(decoder::video::decode_stream(source.clone(), video_tx));
    let decode_audio = tokio::spawn(decoder::audio::decode_stream(source, audio_tx));

    // AV sync + render
    let render = tokio::spawn(sync::av_sync::run(video_rx, audio_rx));

    let _ = tokio::join!(decode_video, decode_audio, render);
    Ok(())
}

/// Media info returned after probing a source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub duration_secs: f64,
    pub width: u32,
    pub height: u32,
    pub has_video: bool,
    pub has_audio: bool,
    pub codec_video: Option<String>,
    pub codec_audio: Option<String>,
    pub fps: f64,
}

// ── Python bindings (pyo3) ────────────────────────────────────────────────

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
#[pymodule]
fn ace_player_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAcePlayer>()?;
    Ok(())
}

#[cfg(feature = "python-bindings")]
#[pyclass(name = "AcePlayer")]
struct PyAcePlayer {
    inner: AcePlayer,
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl PyAcePlayer {
    #[new]
    fn new(source: String) -> Self {
        Self {
            inner: AcePlayer::new(PlayerConfig {
                source,
                ..Default::default()
            }),
        }
    }

    fn play(&self) -> PyResult<()> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.inner.play()).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(e.to_string())
        })
    }
}
