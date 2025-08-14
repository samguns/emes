use axum::Router;
use std::sync::Arc;

use crate::api::py_tasks::routes::routes as py_tasks_routes;
use crate::api::upload::routes::routes as upload_routes;
use crate::app_state::AppState;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/py-tasks", py_tasks_routes(app_state.clone()))
        .nest("/upload", upload_routes(app_state.clone()))
}
