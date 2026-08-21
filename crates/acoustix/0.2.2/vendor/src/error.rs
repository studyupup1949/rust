use thiserror::Error;

/// The main error type for the `acoustix` library operations.
#[derive(Error, Debug)]
pub enum AcoustixError {
    /// Error related to reading or writing WAV files.
    #[cfg(feature = "wav")]
    #[error("Audio file read/write error: {0}")]
    AudioError(#[from] hound::Error),

    /// General I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error thrown when an input signal or dataset is empty.
    #[error("Signal must not be empty: {0}")]
    EmptySignal(String),

    /// Error thrown when DSP frame partitioning parameters are invalid.
    #[error(
        "Invalid framing parameters: frame_len={frame_len}, overlap={overlap}. Frame length must be > 0 and overlap must be < frame_len."
    )]
    InvalidFraming {
        /// The length of each frame in samples.
        frame_len: usize,
        /// The overlap between consecutive frames in samples.
        overlap: usize,
    },

    /// Error thrown when an invalid configuration or calculation parameter is passed.
    #[error("Invalid calculation parameter: {0}")]
    InvalidParameter(String),

    /// Error thrown when sequence alignment (DTW) fails.
    #[error("DTW alignment error: {0}")]
    AlignmentError(String),
}
