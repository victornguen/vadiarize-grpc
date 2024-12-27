use thiserror::Error;

#[derive(Error, Debug)]
pub enum VadServiceError {
    #[error("VAD error: {0}")]
    VadError(
        #[source]
        #[from]
        silero_vad::error::VadError,
    ),
    #[error("Failed to parse audio: {0}")]
    WavReaderError(
        #[source]
        #[from]
        hound::Error,
    ),
    #[error("Invalid audio: {0}")]
    InvalidAudio(String),
    #[error("Error: {0}")]
    Internal(
        #[source]
        #[from]
        Box<dyn std::error::Error + Send>,
    ),
}

pub type Result<T> = std::result::Result<T, VadServiceError>;
