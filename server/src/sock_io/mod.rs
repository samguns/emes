use socketioxide::SocketIo;
use socketioxide::extract::{Data, SocketRef};

mod ns_ai;

pub async fn io_ai_ns(io: &SocketIo) {
    io.ns("/ai", async |s: SocketRef| {
        tracing::info!("ai namespace connected");
        s.on("training:ack", ns_ai::on_training_ack);

        s.on("message", |s: SocketRef, Data(msg): Data<String>| {
            tracing::info!("ai message: {}", msg);
        });
    });
}
