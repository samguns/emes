use pyo3::prelude::*;
use socketioxide::SocketIo;
use std::sync::Arc;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// use crate::qwen2_vl::qwen2_vl_service::Qwen2VLService;
// Remove unresolved imports and fix module usage
mod api;
mod app_state;
mod dao;
mod player;
mod sock_io;
mod ws2812;

use app_state::AppState;

use crate::ws2812::Ws2812StripTask;

const SERVER_ADDR: &str = "0.0.0.0:8642";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Ensure Python can import modules from the crate's `python` directory
    Python::with_gil(|py| {
        let result: PyResult<()> = (|| {
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;
            let python_dir = format!("{}/python", env!("CARGO_MANIFEST_DIR"));
            let _ = path.call_method1("insert", (0, python_dir));
            Ok(())
        })();
        if let Err(e) = result {
            tracing::error!("Failed to extend sys.path for Python modules: {}", e);
        }
    });

    // let service = StreamableHttpService::new(
    //     || Ok(Qwen2VLService::new()),
    //     LocalSessionManager::default().into(),
    //     Default::default(),
    // );

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(AllowOrigin::any())
        .expose_headers(Any);

    let app_state = Arc::new(AppState::new().await);
    let tracker = TaskTracker::new();
    let shutdown_token = CancellationToken::new();
    background_tasks(app_state.clone(), tracker.clone(), shutdown_token.clone()).await;

    let (io_layer, io) = SocketIo::builder()
        .with_state(app_state.clone())
        .build_layer();

    sock_io::io_ai_ns(&io).await;

    let router = axum::Router::new()
        .nest("/api", api::routes::routes(app_state.clone()))
        // .nest_service("/mcp", service)
        .layer(cors)
        .layer(io_layer);

    let tcp_listener = tokio::net::TcpListener::bind(SERVER_ADDR).await?;
    tracing::info!("Server is running on {}", SERVER_ADDR);
    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(graceful_shutdown(tracker, shutdown_token))
        .await;

    Ok(())
}

async fn background_tasks(
    app_state: Arc<AppState>,
    tracker: TaskTracker,
    shutdown_token: CancellationToken,
) {
    let led_strip_task_shutdown_token = shutdown_token.clone();

    let player = app_state.player_state.get_music_player();
    tracker.spawn(async move {
        player.run(shutdown_token).await;
    });

    let led_strip_task = Ws2812StripTask::new(app_state.clone());
    tracker.spawn(async move {
        led_strip_task.run(led_strip_task_shutdown_token).await;
    });
}

async fn graceful_shutdown(tracker: TaskTracker, shutdown_token: CancellationToken) {
    let ctrl_c = async {
        if signal::ctrl_c().await.is_err() {
            tracing::error!("Failed to install Ctrl+C handler");
        }
    };

    tokio::select! {
        _ = ctrl_c => {
            tracing::warn!("Shutting down...");
            shutdown_token.cancel();
        }
    }

    tracker.close();
    tracker.wait().await;

    tracing::info!("Shutdown complete");
}
