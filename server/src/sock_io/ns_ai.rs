use std::sync::Arc;

use crate::app_state::AppState;

pub async fn on_training_ack(msg: String, app_state: Arc<AppState>) {
    tracing::info!("on_training_ack: {}", msg);
}
