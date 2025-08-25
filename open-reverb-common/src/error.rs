use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenReverbError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Audio error: {0}")]
    AudioError(String),
    
    #[error("Video error: {0}")]
    VideoError(String),
    
    #[error("Screen sharing error: {0}")]
    ScreenShareError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, OpenReverbError>;