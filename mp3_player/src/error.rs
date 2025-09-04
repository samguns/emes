use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlayerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("FFmpeg error: {0}")]
    Ffmpeg(#[from] ffmpeg_next::Error),
    
    #[error("Audio device error: {0}")]
    AudioDevice(String),
    
    #[error("Decode error: {0}")]
    Decode(String),
    
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
    
    #[error("No audio tracks found")]
    NoAudioTracks,
    
    #[error("Player not initialized")]
    NotInitialized,
    
    #[error("Invalid operation in current state: {0}")]
    InvalidOperation(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Unsupported codec: {0}")]
    UnsupportedCodec(String),
}

pub type Result<T> = std::result::Result<T, PlayerError>;