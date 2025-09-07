use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use std::sync::Arc;

use crate::api::upload::upload;
use crate::app_state::AppState;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(upload::upload_file))
        // Set a large max upload file size (e.g., 100 MB)
        .layer(DefaultBodyLimit::max(usize::MAX))
        .with_state(app_state.clone())
}
