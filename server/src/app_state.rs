use crate::dao::db_state::DBClientState;

#[derive(Clone)]
pub struct AppState {
    pub db_state: DBClientState,
}

impl AppState {
    pub async fn new() -> Self {
        Self {
            db_state: DBClientState::new().await,
        }
    }
}
