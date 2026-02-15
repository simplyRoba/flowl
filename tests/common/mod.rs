#![allow(dead_code)]

use std::path::PathBuf;

use axum::Router;
use axum::body::Body;
use axum::http::Request;
use flowl::state::AppState;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

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

pub async fn test_app() -> Router {
    let pool = test_pool().await;
    let upload_dir = std::env::temp_dir().join(format!("flowl-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&upload_dir).expect("Failed to create test upload dir");
    let state = AppState {
        pool,
        upload_dir,
        mqtt_client: None,
        mqtt_prefix: "flowl".to_string(),
    };
    flowl::server::router(state)
}

pub async fn test_app_with_uploads() -> (Router, PathBuf) {
    let pool = test_pool().await;
    let upload_dir = std::env::temp_dir().join(format!("flowl-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&upload_dir).expect("Failed to create test upload dir");
    let state = AppState {
        pool,
        upload_dir: upload_dir.clone(),
        mqtt_client: None,
        mqtt_prefix: "flowl".to_string(),
    };
    (flowl::server::router(state), upload_dir)
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
