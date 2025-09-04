use axum::Router;
use std::sync::Arc;

use crate::api::filelist::routes::routes as filelist_routes;
use crate::api::player::routes as player_routes;
use crate::api::playlist::routes::routes as playlist_routes;
use crate::api::py_tasks::routes::routes as py_tasks_routes;
use crate::api::upload::routes::routes as upload_routes;
use crate::app_state::AppState;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/filelist", filelist_routes(app_state.clone()))
        .nest("/py-tasks", py_tasks_routes(app_state.clone()))
        .nest("/upload", upload_routes(app_state.clone()))
        .nest("/playlist", playlist_routes(app_state.clone()))
        .nest("/player", player_routes(app_state.clone()))
}
