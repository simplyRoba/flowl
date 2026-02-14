use axum::Router;
use axum::response::Json;
use axum::routing::get;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

use crate::embedded::static_handler;

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .fallback(static_handler)
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
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
