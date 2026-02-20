use axum::Router;
use axum::response::Json;
use axum::routing::get;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::info;

use crate::api;
use crate::embedded::static_handler;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let uploads = ServeDir::new(&state.upload_dir);
    Router::new()
        .route("/health", get(health))
        .route("/api/info", get(info))
        .nest("/api", api::router(state))
        .nest_service("/uploads", uploads)
        .fallback(static_handler)
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn info() -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "repository": env!("CARGO_PKG_REPOSITORY"),
        "license": env!("CARGO_PKG_LICENSE"),
    }))
}

/// # Errors
///
/// Returns an error if the TCP listener cannot bind to the given port.
pub async fn serve(router: Router, port: u16) -> std::io::Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {addr}");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("Shutdown signal received");
}
