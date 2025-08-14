use axum::routing::get;
use axum::Router;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::api::py_tasks::py_tasks;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
    .route("/greet", get(py_tasks::greet))
    .with_state(app_state.clone())
}
