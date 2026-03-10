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
async fn mqtt_repair_disabled_returns_409() {
    let (app, _dir) = common::test_app().await;

    let response = app
        .oneshot(common::json_request("POST", "/api/mqtt/repair", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let json = common::body_json(response).await;
    assert_eq!(json["code"], "MQTT_DISABLED");
}

#[tokio::test]
async fn mqtt_repair_disconnected_returns_503() {
    let (app, _dir) = test_app_mqtt_enabled(false).await;

    let response = app
        .oneshot(common::json_request("POST", "/api/mqtt/repair", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let json = common::body_json(response).await;
    assert_eq!(json["code"], "MQTT_UNAVAILABLE");
}
