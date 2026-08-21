# AudioVideoConverter

AudioVideoConverter is a Rust library that provides functionality to convert between audio and video files using the FFmpeg library.

## Features
- Convert Audio to Video: Convert an audio file to a video file by combining with a background image.
- Convert Video to Audio: Extract audio from a video file and save it as a separate audio file.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
audio_video_converter = "0.1.0"
```

# Usage
```rust
fn main() {
    // Convert audio file to video
    AudioVideoConverter::convert_audio_to_video("input_audio.mp3", "output_video.mp4")
        .unwrap();
    println!("Audio converted to video successfully!");

    // Convert video file to audio
    AudioVideoConverter::convert_video_to_audio("input_video.mp4", "output_audio.mp3")
        .unwrap();
    println!("Video converted to audio successfully!");
}
```

## Error
- The functions return Result<(), io::Error>. Errors can occur due to issues with the conversion process or I/O operations. Handle errors appropriately in your code.

## Dependencies
- This crate depends on FFmpeg. Make sure FFmpeg is installed on your system and available in your PATH.
