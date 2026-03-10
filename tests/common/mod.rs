#![allow(dead_code)]

use axum::Router;
use axum::body::Body;
use axum::http::Request;
use flowl::state::AppState;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tempfile::TempDir;

pub async fn test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(":memory:")
                .create_if_missing(true),
        )
        .await
        .expect("Failed to create test pool");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

fn make_state(pool: SqlitePool, upload_dir: &std::path::Path) -> AppState {
    AppState {
        pool,
        image_store: flowl::images::ImageStore::new(upload_dir.to_path_buf()),
        mqtt_client: None,
        mqtt_prefix: "flowl".to_string(),
        mqtt_connected: None,
        mqtt_host: "localhost".to_string(),
        mqtt_port: 1883,
        mqtt_disabled: true,
        ai_provider: None,
        ai_base_url: String::new(),
        ai_model: String::new(),
    }
}

pub async fn test_app() -> (Router, TempDir) {
    let pool = test_pool().await;
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let state = make_state(pool, tmp.path());
    (flowl::server::router(state), tmp)
}

pub async fn test_app_with_uploads() -> (Router, TempDir) {
    let pool = test_pool().await;
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let state = make_state(pool, tmp.path());
    (flowl::server::router(state), tmp)
}

pub fn json_request(method: &str, uri: &str, body: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    builder
        .body(body.map_or_else(Body::empty, |b| Body::from(b.to_string())))
        .unwrap()
}

pub async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}
