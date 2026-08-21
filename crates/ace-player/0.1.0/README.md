# ACE Player

## Description
ACE Player is a high-performance Rust media engine SDK designed to handle a wide range of media formats efficiently. It provides robust support for decoding, encoding, and streaming media content, making it ideal for developers working with multimedia applications.

## Features
- **High Performance**: Optimized for fast media processing with low latency.
- **Cross-Platform Support**: Works on Windows, macOS, and Linux.
- **Extensive Media Formats**: Supports a variety of video and audio formats.
- **Advanced Decoding**: Utilizes symphonia for pure Rust audio decoding.
- **Flexible Bindings**: Supports Python and Node bindings for easy integration into other applications.
- **WebAssembly Support**: Enables web-based media processing with wasm-bindgen.
- **Customizable Configurations**: Offers extensive configuration options for different use cases.

## Installation
To install ACE Player, follow these steps:

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/PurpleSquirrelMedia/ace-player.git
   ```

2. **Navigate to the Project Directory**:
   ```bash
   cd ace-player
   ```

3. **Install the Project**:
   ```bash
   cargo install
   ```

## Usage
Here's a basic example of how to use ACE Player to decode a video file:

```rust
use ace_player::{
    AudioDecoder,
    VideoDecoder,
};

fn main() {
    let video_path = "/path/to/your/video.mp4";
    let audio_path = "/path/to/your/audio.mp3";

    let decoder = VideoDecoder::new(video_path);
    let audio_decoder = AudioDecoder::new(audio_path);

    // Decode the video and audio
    decoder.decode().expect("Failed to decode video");
    audio_decoder.decode().expect("Failed to decode audio");

    // Process the decoded data
    // ...
}
```

## Configuration
ACE Player supports various configuration options to tailor its behavior to specific requirements. You can configure the SDK using the `cargo config` command.

### Example Configuration
```toml
[config]
debug = true
log_level = "info"
```

### Running with Debug Mode
```bash
cargo run --config debug
```

## Development
For development, you can use the `cargo dev` command to build and run the project with debug information.

## License
ACE Player is licensed under a proprietary license. For more information, please refer to the LICENSE file in the repository.

This README provides a comprehensive overview of ACE Player, including its features, installation, usage, and development information.
