//! Media decoding — video + audio streams.
pub mod video;
pub mod audio;

use anyhow::Result;
use crate::MediaInfo;

/// Probe a media source to extract metadata without full decode.
pub async fn probe(source: &str) -> Result<MediaInfo> {
    use std::path::Path;

    // TODO: use symphonia probe for full format detection
    // For now, return a stub — replace with symphonia::probe in Phase 1
    tracing::info!("Probing source: {source}");

    Ok(MediaInfo {
        duration_secs: 0.0,
        width: 1920,
        height: 1080,
        has_video: true,
        has_audio: true,
        codec_video: Some("h264".to_string()),
        codec_audio: Some("aac".to_string()),
        fps: 30.0,
    })
}
