// File: lib.rs

use std::process::{Command, Stdio};
use std::io;

pub struct AudioVideoConverter;

impl AudioVideoConverter {
    /// Converts audio file to video by combining with a background image.
    ///
    /// # Arguments
    ///
    /// * `audio_file` - The path to the input audio file.
    /// * `output_file` - The path to the output video file.
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion process fails or if there are issues with I/O operations.
    pub fn convert_audio_to_video(audio_file: &str, output_file: &str) -> Result<(), io::Error> {
        let output = Command::new("ffmpeg")
            .args(&[
                "-loop", "1", "-i", "background_image.jpg", // Assuming background image
                "-i", audio_file,
                "-c:v", "libx264", "-tune", "stillimage", "-c:a", "aac",
                "-strict", "-2", "-b:a", "192k",
                "-shortest", output_file,
            ])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output();

        match output {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Converts video file to audio file.
    ///
    /// # Arguments
    ///
    /// * `video_file` - The path to the input video file.
    /// * `output_file` - The path to the output audio file.
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion process fails or if there are issues with I/O operations.
    pub fn convert_video_to_audio(video_file: &str, output_file: &str) -> Result<(), io::Error> {
        let output = Command::new("ffmpeg")
            .args(&[
                "-i", video_file,
                "-vn", "-ac", "2", "-ar", "44100", "-ab", "192k",
                "-f", "mp3", output_file,
            ])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output();

        match output {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
