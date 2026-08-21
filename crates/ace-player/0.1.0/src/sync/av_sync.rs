//! AV synchronization — aligns video and audio streams.
//!
//! Uses tokio channels for race-condition-free frame timing.
//! This is the "Fearless Concurrency" guarantee from the business proposal.
use anyhow::Result;
use tokio::sync::mpsc::Receiver;
use crate::decoder::{audio::AudioFrame, video::VideoFrame};

/// Run AV sync loop: consume video + audio frames and deliver at correct PTS.
pub async fn run(
    mut video_rx: Receiver<VideoFrame>,
    mut audio_rx: Receiver<AudioFrame>,
) -> Result<()> {
    tracing::info!("AV sync started");

    // Master clock: derive from audio PTS (audio drives sync, video follows)
    let mut audio_sample_count: u64 = 0;
    let sample_rate: u64 = 44100;

    loop {
        tokio::select! {
            Some(video_frame) = video_rx.recv() => {
                // Calculate current audio clock in ms
                let audio_clock_ms = (audio_sample_count * 1000) / sample_rate;
                let video_pts_ms = video_frame.pts_ms;

                // Drop late frames, hold early frames
                if video_pts_ms + 50 < audio_clock_ms {
                    tracing::debug!("Dropping late video frame: pts={video_pts_ms}ms clock={audio_clock_ms}ms");
                    continue;
                }
                if video_pts_ms > audio_clock_ms + 100 {
                    // Video is ahead — brief yield
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        (video_pts_ms - audio_clock_ms).min(100)
                    )).await;
                }

                // Render frame (send to output module)
                crate::output::render_frame(&video_frame).await;
            }

            Some(audio_frame) = audio_rx.recv() => {
                audio_sample_count += audio_frame.len() as u64 / 2; // stereo
                crate::output::play_audio(&audio_frame).await;
            }

            else => break,
        }
    }

    tracing::info!("AV sync completed");
    Ok(())
}
