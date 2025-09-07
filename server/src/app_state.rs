use crate::dao::db_state::DBClientState;
use crate::player::PlayerState;
use crate::ws2812::LedStripState;

#[derive(Clone)]
pub struct AppState {
    pub db_state: DBClientState,
    pub player_state: PlayerState,
    pub led_strip_state: LedStripState,
}

impl AppState {
    pub async fn new() -> Self {
        Self {
            db_state: DBClientState::new().await,
            player_state: PlayerState::new(),
            led_strip_state: LedStripState::new(),
        }
    }
}
