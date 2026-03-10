use std::time::Instant;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::get;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::{debug, info};

use crate::api;
use crate::embedded::static_handler;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let uploads = ServeDir::new(state.image_store.upload_dir());
    let pool = state.pool.clone();
    Router::new()
        .route("/health", get(move || health(pool)))
        .route("/api/info", get(info))
        .nest("/api", api::router(state))
        .nest_service("/uploads", uploads)
        .fallback(static_handler)
        .layer(middleware::from_fn(access_log))
}

async fn access_log(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();
    let response = next.run(req).await;
    let status = response.status().as_u16();
    let latency = start.elapsed();
    debug!(method = %method, path, status, latency_ms = latency.as_millis(), "access");
    response
}

async fn health(pool: SqlitePool) -> impl IntoResponse {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "ok"}))),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": "unhealthy"})),
        ),
    }
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
