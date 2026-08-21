//! Output module — native + WebAssembly rendering targets.
use crate::decoder::{audio::AudioFrame, video::VideoFrame};

/// Render a video frame to the active output target.
pub async fn render_frame(frame: &VideoFrame) {
    // Phase 1: log frame metadata
    // Phase 2: wgpu render pass (native) or Canvas2D (wasm)
    tracing::trace!("Render frame: pts={}ms {}x{}", frame.pts_ms, frame.width, frame.height);
}

/// Send audio samples to the active audio output.
pub async fn play_audio(samples: &AudioFrame) {
    // Phase 1: stub
    // Phase 2: cpal (cross-platform audio) for native, Web Audio API for wasm
    tracing::trace!("Audio: {} samples", samples.len());
}
