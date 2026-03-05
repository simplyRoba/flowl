mod common;

use std::sync::Arc;

use async_trait::async_trait;
use axum::Router;
use axum::http::StatusCode;
use flowl::ai::provider::AiProvider;
use flowl::ai::types::{ChatMessage, ChatResponseStream, IdentifyResponse};
use flowl::state::AppState;
use tower::ServiceExt;

struct MockSummarizeProvider;

#[async_trait]
impl AiProvider for MockSummarizeProvider {
    async fn identify(
        &self,
        _images: &[&[u8]],
        _locale: &str,
    ) -> Result<IdentifyResponse, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!()
    }

    async fn chat(
        &self,
        _system_prompt: &str,
        _messages: &[ChatMessage],
        _image: Option<&[u8]>,
        _locale: &str,
    ) -> Result<ChatResponseStream, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!()
    }

    async fn summarize(
        &self,
        _system_prompt: &str,
        _messages: &[ChatMessage],
        _locale: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(
            "Diagnosed yellowing leaves as overwatering. Recommended reducing frequency."
                .to_string(),
        )
    }
}

struct FailingSummarizeProvider;

#[async_trait]
impl AiProvider for FailingSummarizeProvider {
    async fn identify(
        &self,
        _images: &[&[u8]],
        _locale: &str,
    ) -> Result<IdentifyResponse, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!()
    }

    async fn chat(
        &self,
        _system_prompt: &str,
        _messages: &[ChatMessage],
        _image: Option<&[u8]>,
        _locale: &str,
    ) -> Result<ChatResponseStream, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!()
    }

    async fn summarize(
        &self,
        _system_prompt: &str,
        _messages: &[ChatMessage],
        _locale: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Err("upstream API error".into())
    }
}

async fn test_app_with_provider(provider: Arc<dyn AiProvider>) -> (Router, sqlx::SqlitePool) {
    let pool = common::test_pool().await;
    let upload_dir = std::env::temp_dir().join(format!("flowl-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&upload_dir).expect("Failed to create test upload dir");

    let state = AppState {
        pool: pool.clone(),
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
    (flowl::server::router(state), pool)
}

async fn insert_test_plant(pool: &sqlx::SqlitePool) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO plants (name, light_needs, watering_interval_days, created_at, updated_at) \
         VALUES ('TestPlant', 'indirect', 7, '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert test plant")
}

#[tokio::test]
async fn summarize_returns_503_when_ai_not_configured() {
    let app = common::test_app().await;

    let request = common::json_request(
        "POST",
        "/api/ai/summarize",
        Some(r#"{"plant_id":1,"history":[{"role":"user","content":"hi"}]}"#),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = common::body_json(response).await;
    assert_eq!(body["message"], "AI provider is not configured");
}

#[tokio::test]
async fn summarize_returns_404_when_plant_not_found() {
    let (app, _pool) = test_app_with_provider(Arc::new(MockSummarizeProvider)).await;

    let request = common::json_request(
        "POST",
        "/api/ai/summarize",
        Some(r#"{"plant_id":9999,"history":[{"role":"user","content":"hi"}]}"#),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn summarize_returns_422_for_empty_history() {
    let (app, pool) = test_app_with_provider(Arc::new(MockSummarizeProvider)).await;
    let plant_id = insert_test_plant(&pool).await;

    let request = common::json_request(
        "POST",
        "/api/ai/summarize",
        Some(&format!(r#"{{"plant_id":{plant_id},"history":[]}}"#)),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = common::body_json(response).await;
    assert!(body["message"].as_str().is_some());
}

#[tokio::test]
async fn summarize_returns_200_with_summary() {
    let (app, pool) = test_app_with_provider(Arc::new(MockSummarizeProvider)).await;
    let plant_id = insert_test_plant(&pool).await;

    let request = common::json_request(
        "POST",
        "/api/ai/summarize",
        Some(&format!(
            r#"{{"plant_id":{plant_id},"history":[{{"role":"user","content":"Leaves are yellow"}},{{"role":"assistant","content":"Likely overwatering"}}]}}"#
        )),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = common::body_json(response).await;
    let summary = body["summary"].as_str().unwrap();
    assert!(summary.contains("overwatering"));
}

#[tokio::test]
async fn summarize_returns_500_when_provider_fails() {
    let (app, pool) = test_app_with_provider(Arc::new(FailingSummarizeProvider)).await;
    let plant_id = insert_test_plant(&pool).await;

    let request = common::json_request(
        "POST",
        "/api/ai/summarize",
        Some(&format!(
            r#"{{"plant_id":{plant_id},"history":[{{"role":"user","content":"hi"}}]}}"#
        )),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = common::body_json(response).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("AI summarize failed")
    );
}
