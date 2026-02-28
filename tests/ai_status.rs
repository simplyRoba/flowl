mod common;

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use flowl::ai::openai::OpenAiProvider;
use flowl::ai::provider::AiProvider;
use flowl::state::AppState;
use tower::ServiceExt;

async fn test_app_ai_enabled() -> Router {
    let pool = common::test_pool().await;
    let upload_dir = std::env::temp_dir().join(format!("flowl-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&upload_dir).expect("Failed to create test upload dir");

    let provider: Arc<dyn AiProvider> = Arc::new(OpenAiProvider::new(
        "sk-test".into(),
        "https://api.openai.com/v1".into(),
        "gpt-4.1-mini".into(),
    ));

    let state = AppState {
        pool,
        image_store: flowl::images::ImageStore::new(upload_dir),
        mqtt_client: None,
        mqtt_prefix: "flowl".to_string(),
        mqtt_connected: None,
        mqtt_host: "localhost".to_string(),
        mqtt_port: 1883,
        mqtt_disabled: true,
        ai_provider: Some(provider),
        ai_base_url: "https://api.openai.com/v1".to_string(),
        ai_model: "gpt-4.1-mini".to_string(),
    };
    flowl::server::router(state)
}

#[tokio::test]
async fn ai_status_enabled() {
    let app = test_app_ai_enabled().await;

    let response = app
        .oneshot(common::json_request("GET", "/api/ai/status", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = common::body_json(response).await;
    assert_eq!(body["enabled"], true);
    assert_eq!(body["base_url"], "https://api.openai.com/v1");
    assert_eq!(body["model"], "gpt-4.1-mini");
}

#[tokio::test]
async fn ai_status_disabled() {
    let app = common::test_app().await;

    let response = app
        .oneshot(common::json_request("GET", "/api/ai/status", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = common::body_json(response).await;
    assert_eq!(body["enabled"], false);
    assert!(body["base_url"].is_null());
    assert!(body["model"].is_null());
}
