mod common;

use std::sync::Arc;

use async_trait::async_trait;
use axum::Router;
use axum::http::StatusCode;
use flowl::ai::provider::AiProvider;
use flowl::ai::types::{ChatMessage, ChatResponseStream, IdentifyResponse};
use flowl::state::AppState;
use tower::ServiceExt;

struct MockChatProvider;

#[async_trait]
impl AiProvider for MockChatProvider {
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
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            let _ = tx.send(Ok("Hello ".to_string())).await;
            let _ = tx.send(Ok("plant friend!".to_string())).await;
        });
        Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
    }

    async fn summarize(
        &self,
        _system_prompt: &str,
        _messages: &[ChatMessage],
        _locale: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!()
    }
}

struct FailingChatProvider;

#[async_trait]
impl AiProvider for FailingChatProvider {
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
        Err("upstream API error".into())
    }

    async fn summarize(
        &self,
        _system_prompt: &str,
        _messages: &[ChatMessage],
        _locale: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!()
    }
}

struct StreamingErrorProvider;

#[async_trait]
impl AiProvider for StreamingErrorProvider {
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
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            let _ = tx.send(Ok("partial response".to_string())).await;
            let _ = tx.send(Err("stream failed".to_string())).await;
            let _ = tx.send(Ok("must not be sent".to_string())).await;
        });
        Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
    }

    async fn summarize(
        &self,
        _system_prompt: &str,
        _messages: &[ChatMessage],
        _locale: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!()
    }
}

async fn test_app_with_provider(
    provider: Arc<dyn AiProvider>,
) -> (Router, sqlx::SqlitePool, tempfile::TempDir) {
    let pool = common::test_pool().await;
    let tmp = tempfile::TempDir::new().expect("Failed to create temp dir");

    let state = AppState {
        pool: pool.clone(),
        image_store: flowl::images::ImageStore::new(tmp.path().to_path_buf()),
        mqtt_client: None,
        mqtt_prefix: "flowl".to_string(),
        mqtt_connected: None,
        mqtt_host: "localhost".to_string(),
        mqtt_port: 1883,
        mqtt_disabled: true,
        ai_provider: Some(provider),
        ai_base_url: "https://api.openai.com/v1".to_string(),
        ai_model: "gpt-4.1-mini".to_string(),
        ai_rate_limiter: None,
        auth: None,
    };
    (flowl::server::router(state), pool, tmp)
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

async fn body_bytes(response: axum::response::Response) -> Vec<u8> {
    axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec()
}

#[tokio::test]
async fn chat_returns_503_when_ai_not_configured() {
    let (app, _dir) = common::test_app().await;

    let request = common::json_request(
        "POST",
        "/api/ai/chat",
        Some(r#"{"plant_id":1,"message":"hello"}"#),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = common::body_json(response).await;
    assert_eq!(body["code"], "AI_NOT_CONFIGURED");
}

#[tokio::test]
async fn chat_returns_404_when_plant_not_found() {
    let (app, _pool, _dir) = test_app_with_provider(Arc::new(MockChatProvider)).await;

    let request = common::json_request(
        "POST",
        "/api/ai/chat",
        Some(r#"{"plant_id":9999,"message":"hello"}"#),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn chat_streams_sse_events() {
    let (app, pool, _dir) = test_app_with_provider(Arc::new(MockChatProvider)).await;
    let plant_id = insert_test_plant(&pool).await;

    let request = common::json_request(
        "POST",
        "/api/ai/chat",
        Some(&format!(
            r#"{{"plant_id":{plant_id},"message":"Why are leaves yellow?"}}"#
        )),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = String::from_utf8(body_bytes(response).await).unwrap();

    // SSE events should contain delta tokens and a done marker
    assert!(body.contains(r#""delta":"Hello "#));
    assert!(body.contains(r#""delta":"plant friend!"#));
    assert!(body.contains(r#""done":true"#));
}

#[tokio::test]
async fn chat_stream_error_is_terminal() {
    let (app, pool, _dir) = test_app_with_provider(Arc::new(StreamingErrorProvider)).await;
    let plant_id = insert_test_plant(&pool).await;

    let request = common::json_request(
        "POST",
        "/api/ai/chat",
        Some(&format!(r#"{{"plant_id":{plant_id},"message":"help"}}"#)),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = String::from_utf8(body_bytes(response).await).unwrap();
    assert!(body.contains(r#""delta":"partial response""#));
    assert!(body.contains(r#""code":"AI_STREAM_ERROR""#));
    assert!(!body.contains(r#""done":true"#));
    assert!(!body.contains("must not be sent"));
}

#[tokio::test]
async fn chat_streams_with_history() {
    let (app, pool, _dir) = test_app_with_provider(Arc::new(MockChatProvider)).await;
    let plant_id = insert_test_plant(&pool).await;

    let request = common::json_request(
        "POST",
        "/api/ai/chat",
        Some(&format!(
            r#"{{"plant_id":{plant_id},"message":"And now?","history":[{{"role":"user","content":"Hi"}},{{"role":"assistant","content":"Hello!"}}]}}"#
        )),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = String::from_utf8(body_bytes(response).await).unwrap();
    assert!(body.contains(r#""delta""#));
    assert!(body.contains(r#""done":true"#));
}

#[tokio::test]
async fn chat_streams_with_image_in_history() {
    let (app, pool, _dir) = test_app_with_provider(Arc::new(MockChatProvider)).await;
    let plant_id = insert_test_plant(&pool).await;
    let image = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
    );

    let request = common::json_request(
        "POST",
        "/api/ai/chat",
        Some(&format!(
            r#"{{"plant_id":{plant_id},"message":"And now?","history":[{{"role":"user","content":"What is this?","image":"data:image/png;base64,{image}"}},{{"role":"assistant","content":"It looks stressed."}}]}}"#
        )),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn chat_rejects_invalid_history_image() {
    let (app, pool, _dir) = test_app_with_provider(Arc::new(FailingChatProvider)).await;
    let plant_id = insert_test_plant(&pool).await;

    let png_as_jpeg = format!(
        "data:image/jpeg;base64,{}",
        base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        )
    );
    for image in [
        "data:image/gif;base64,R0lGODlh".to_string(),
        "data:image/png;base64,not-valid-base64!".to_string(),
        png_as_jpeg,
    ] {
        let request = common::json_request(
            "POST",
            "/api/ai/chat",
            Some(&format!(
                r#"{{"plant_id":{plant_id},"message":"And now?","history":[{{"role":"user","content":"What is this?","image":"{image}"}}]}}"#
            )),
        );

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = common::body_json(response).await;
        assert_eq!(body["code"], "AI_INVALID_IMAGE");
    }
}

#[tokio::test]
async fn chat_streams_with_image_data_url() {
    let (app, pool, _dir) = test_app_with_provider(Arc::new(MockChatProvider)).await;
    let plant_id = insert_test_plant(&pool).await;
    let image = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        [0x52, 0x49, 0x46, 0x46, 0, 0, 0, 0, 0x57, 0x45, 0x42, 0x50],
    );

    let request = common::json_request(
        "POST",
        "/api/ai/chat",
        Some(&format!(
            r#"{{"plant_id":{plant_id},"message":"What is this?","image":"data:image/webp;base64,{image}"}}"#
        )),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = String::from_utf8(body_bytes(response).await).unwrap();
    assert!(body.contains(r#""delta""#));
    assert!(body.contains(r#""done":true"#));
}

#[tokio::test]
async fn chat_returns_400_for_invalid_base64_image() {
    let (app, pool, _dir) = test_app_with_provider(Arc::new(MockChatProvider)).await;
    let plant_id = insert_test_plant(&pool).await;

    let request = common::json_request(
        "POST",
        "/api/ai/chat",
        Some(&format!(
            r#"{{"plant_id":{plant_id},"message":"What?","image":"data:image/png;base64,!!!not-base64!!!"}}"#
        )),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn chat_returns_500_when_provider_fails() {
    let (app, pool, _dir) = test_app_with_provider(Arc::new(FailingChatProvider)).await;
    let plant_id = insert_test_plant(&pool).await;

    let request = common::json_request(
        "POST",
        "/api/ai/chat",
        Some(&format!(r#"{{"plant_id":{plant_id},"message":"hello"}}"#)),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = common::body_json(response).await;
    assert_eq!(body["code"], "AI_PROVIDER_FAILED");
}

#[tokio::test]
async fn chat_returns_429_when_rate_limited() {
    let pool = common::test_pool().await;
    let tmp = tempfile::TempDir::new().expect("Failed to create temp dir");

    let state = AppState {
        pool: pool.clone(),
        image_store: flowl::images::ImageStore::new(tmp.path().to_path_buf()),
        mqtt_client: None,
        mqtt_prefix: "flowl".to_string(),
        mqtt_connected: None,
        mqtt_host: "localhost".to_string(),
        mqtt_port: 1883,
        mqtt_disabled: true,
        ai_provider: Some(Arc::new(MockChatProvider)),
        ai_base_url: "https://api.openai.com/v1".to_string(),
        ai_model: "gpt-4.1-mini".to_string(),
        ai_rate_limiter: Some(Arc::new(flowl::state::AiRateLimiter::new(1))),
        auth: None,
    };
    let app = flowl::server::router(state);
    let plant_id = insert_test_plant(&pool).await;

    let body_str = format!(r#"{{"plant_id":{plant_id},"message":"hello"}}"#);

    // First request should succeed
    let request = common::json_request("POST", "/api/ai/chat", Some(&body_str));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Second request should be rate limited
    let request = common::json_request("POST", "/api/ai/chat", Some(&body_str));
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    let body = common::body_json(response).await;
    assert_eq!(body["code"], "AI_RATE_LIMITED");
}
