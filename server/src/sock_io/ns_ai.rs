use socketioxide::extract::{Data, SocketRef, State};

use crate::app_state::AppState;

pub async fn on_training_ack(s: SocketRef, Data(msg): Data<String>, _app_state: State<AppState>) {
    tracing::info!("ai message: {}", msg);
}
