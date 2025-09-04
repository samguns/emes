use axum::Router;
use axum::routing::post;
use std::sync::Arc;

// use crate::api::player::player;
use crate::app_state::AppState;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        // .route("/", post(player::get_player))
        .with_state(app_state.clone())
}
