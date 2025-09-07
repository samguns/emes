use std::sync::Arc;

use crate::player::MusicPlayer;

#[derive(Clone)]

pub struct PlayerState {
    music_player: Arc<MusicPlayer>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            music_player: Arc::new(MusicPlayer::new()),
        }
    }

    pub fn get_music_player(&self) -> Arc<MusicPlayer> {
        self.music_player.clone()
    }
}
