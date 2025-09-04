use crate::dao::db_state::DBClientState;
use crate::player::PlayerState;

#[derive(Clone)]
pub struct AppState {
    pub db_state: DBClientState,
    pub player_state: PlayerState,
}

impl AppState {
    pub async fn new() -> Self {
        Self {
            db_state: DBClientState::new().await,
            player_state: PlayerState::new(),
        }
    }
}
