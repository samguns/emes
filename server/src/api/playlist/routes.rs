use axum::Router;
use axum::routing::post;
use std::sync::Arc;

use crate::api::playlist::playlist::get_list;


use crate::app_state::AppState;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(get_list))
        .with_state(app_state.clone())
}
