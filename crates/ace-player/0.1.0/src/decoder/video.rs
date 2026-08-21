//! Video decode — stub for Phase 1. Full ffmpeg-next integration in Phase 2.
use anyhow::Result;
use tokio::sync::mpsc::Sender;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFrame {
    pub pts_ms: u64,
    pub width: u32,
    pub height: u32,
    /// Raw RGBA pixel data
    pub data: Vec<u8>,
}

/// Decode video stream and send frames to channel.
/// Phase 1: stub implementation.
/// Phase 2: integrate ffmpeg-next for H.264/H.265/AV1 decode.
pub async fn decode_stream(source: String, tx: Sender<VideoFrame>) -> Result<()> {
    tracing::info!("Video decode started for: {source}");

    // Phase 1 stub — signals completion immediately
    // Replace with ffmpeg-next demux + decode loop in Phase 2
    tracing::warn!("Video decode not yet implemented — awaiting ffmpeg-next integration");

    drop(tx); // Signal end of stream
    Ok(())
}
