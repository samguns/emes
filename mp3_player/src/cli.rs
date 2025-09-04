use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerCommand {
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    Seek(f64),
    SetVolume(f32),
    GetVolume,
    GetPosition,
    GetDuration,
    GetStatus,
    LoadFile(String),
    LoadPlaylist(Vec<String>),
    SetLoop(bool),
    SetShuffle(bool),
    Quit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerResponse {
    Ok,
    Error(String),
    Volume(f32),
    Position(f64),
    Duration(f64),
    Status(PlayerStatus),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatus {
    pub state: PlaybackState,
    pub current_track: Option<String>,
    pub position: f64,
    pub duration: f64,
    pub volume: f32,
    pub loop_enabled: bool,
    pub shuffle_enabled: bool,
    pub playlist_length: usize,
    pub current_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}