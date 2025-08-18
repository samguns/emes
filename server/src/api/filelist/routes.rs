use axum::Router;
use axum::routing::post;
use std::sync::Arc;

use crate::api::filelist::filelist;
use crate::app_state::AppState;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(filelist::get_file_list))
        .with_state(app_state.clone())
}
