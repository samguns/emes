use axum::Router;
use axum::routing::get;
use std::sync::Arc;

use crate::api::py_tasks::py_tasks;
use crate::app_state::AppState;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/greet", get(py_tasks::greet))
        .with_state(app_state.clone())
}
