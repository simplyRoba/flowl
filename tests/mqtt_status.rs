mod common;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use axum::Router;
use axum::http::StatusCode;
use flowl::state::AppState;
use tower::ServiceExt;

async fn test_app_mqtt_enabled(connected: bool) -> (Router, tempfile::TempDir) {
    let pool = common::test_pool().await;
    let tmp = tempfile::TempDir::new().expect("Failed to create temp dir");

    let flag = Arc::new(AtomicBool::new(connected));
    let state = AppState {
        pool,
        image_store: flowl::images::ImageStore::new(tmp.path().to_path_buf()),
        mqtt_client: None,
        mqtt_prefix: "flowl".to_string(),
        mqtt_connected: Some(flag),
        mqtt_host: "broker.local".to_string(),
        mqtt_port: 1883,
        mqtt_disabled: false,
        ai_provider: None,
        ai_base_url: String::new(),
        ai_model: String::new(),
    };
    (flowl::server::router(state), tmp)
}

#[tokio::test]
async fn mqtt_status_disabled() {
    let (app, _dir) = common::test_app().await;

    let response = app
        .oneshot(common::json_request("GET", "/api/mqtt/status", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let json = common::body_json(response).await;
    assert_eq!(json["status"], "disabled");
    assert!(json["broker"].is_null());
    assert!(json["topic_prefix"].is_null());
}

#[tokio::test]
async fn mqtt_status_disconnected() {
    let (app, _dir) = test_app_mqtt_enabled(false).await;

    let response = app
        .oneshot(common::json_request("GET", "/api/mqtt/status", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let json = common::body_json(response).await;
    assert_eq!(json["status"], "disconnected");
    assert_eq!(json["broker"], "broker.local:1883");
    assert_eq!(json["topic_prefix"], "flowl");
}

#[tokio::test]
async fn mqtt_status_connected() {
    let (app, _dir) = test_app_mqtt_enabled(true).await;

    let response = app
        .oneshot(common::json_request("GET", "/api/mqtt/status", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let json = common::body_json(response).await;
    assert_eq!(json["status"], "connected");
    assert_eq!(json["broker"], "broker.local:1883");
    assert_eq!(json["topic_prefix"], "flowl");
}
