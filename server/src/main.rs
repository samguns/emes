use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use pyo3::prelude::*;

// use crate::qwen2_vl::qwen2_vl_service::Qwen2VLService;
// Remove unresolved imports and fix module usage
mod api;
mod app_state;
mod dao;

use app_state::AppState;

const SERVER_ADDR: &str = "127.0.0.1:8642";

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

    let router = axum::Router::new()
        .nest("/api", api::routes::routes(app_state.clone()))
        // .nest_service("/mcp", service)
        .layer(cors);
    let tcp_listener = tokio::net::TcpListener::bind(SERVER_ADDR).await?;
    tracing::info!("Server is running on {}", SERVER_ADDR);
    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .await;

    Ok(())
}
