use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;

use crate::api::player::lib;
use crate::app_state::AppState;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/play", post(lib::play))
        .route("/stop", post(lib::stop))
        .route("/toggle", post(lib::toggle))
        .route("/status", get(lib::status))
        .route("/volume", post(lib::set_volume))
        .route("/seek", post(lib::seek))
        .route("/seek_to", post(lib::seek_to))
        .route("/next", post(lib::next))
        .route("/prev", post(lib::prev))
        .with_state(app_state.clone())
}
