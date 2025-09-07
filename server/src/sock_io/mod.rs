use socketioxide::SocketIo;
use socketioxide::extract::{Data, SocketRef, State};
use std::sync::Arc;

use crate::app_state::AppState;

mod ns_ai;

pub async fn io_ai_ns(io: &SocketIo) {
    io.ns(
        "/ai",
        async |s: SocketRef, State(app_state): State<Arc<AppState>>| {
            // tracing::info!("ai namespace connected");
            // s.on("training:ack", ns_ai::on_training_ack);
            s.on(
                "training:ack",
                async move |_s: SocketRef, Data(msg): Data<String>| {
                    ns_ai::on_training_ack(msg, app_state.clone()).await;
                },
            );

            s.on("message", |_s: SocketRef, Data(msg): Data<String>| {
                tracing::info!("ai message: {}", msg);
            });
        },
    );
}
