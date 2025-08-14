use axum::Router;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::api::py_tasks::routes::routes as py_tasks_routes;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
    .nest("/py-tasks", py_tasks_routes(app_state.clone()))
}
