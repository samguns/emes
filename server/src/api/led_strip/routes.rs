use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;

use crate::api::led_strip::lib;
use crate::app_state::AppState;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/status", get(lib::get_led_strip_status))
        .route("/status", post(lib::set_led_strip_status))
        .with_state(app_state.clone())
}
