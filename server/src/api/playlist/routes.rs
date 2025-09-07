use axum::Router;
use axum::routing::post;
use std::sync::Arc;

use crate::api::playlist::playlist;
use crate::app_state::AppState;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(playlist::get_playlist))
        .with_state(app_state.clone())
}
