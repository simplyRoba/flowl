mod common;

use std::sync::Arc;

use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use flowl::ai::provider::AiProvider;
use flowl::ai::types::{ChatMessage, ChatResponseStream, IdentifyResponse, IdentifyResult};
use flowl::state::{AiRateLimiter, AppState};
use tower::ServiceExt;

struct MockAiProvider;

#[async_trait]
impl AiProvider for MockAiProvider {
    async fn identify(
        &self,
        _images: &[&[u8]],
        _locale: &str,
    ) -> Result<IdentifyResponse, Box<dyn std::error::Error + Send + Sync>> {
        Ok(IdentifyResponse {
            suggestions: vec![
                IdentifyResult {
                    common_name: "Monstera".to_string(),
                    scientific_name: "Monstera deliciosa".to_string(),
                    confidence: Some(0.95),
                    summary: Some("A popular tropical houseplant".to_string()),
                    care_profile: None,
                },
                IdentifyResult {
                    common_name: "Philodendron".to_string(),
                    scientific_name: "Philodendron bipinnatifidum".to_string(),
                    confidence: Some(0.72),
                    summary: Some("A tropical foliage plant".to_string()),
                    care_profile: None,
                },
            ],
            rejected: Some(false),
            rejected_reason: None,
        })
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
        unimplemented!()
    }
}

struct FailingAiProvider;

#[async_trait]
impl AiProvider for FailingAiProvider {
    async fn identify(
        &self,
        _images: &[&[u8]],
        _locale: &str,
    ) -> Result<IdentifyResponse, Box<dyn std::error::Error + Send + Sync>> {
        Err("upstream API returned 502".into())
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
        unimplemented!()
    }
}

async fn test_app_with_provider(provider: Arc<dyn AiProvider>) -> (Router, tempfile::TempDir) {
    let pool = common::test_pool().await;
    let tmp = tempfile::TempDir::new().expect("Failed to create temp dir");

    let state = AppState {
        pool,
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
    };
    (flowl::server::router(state), tmp)
}

async fn test_app_mock() -> (Router, tempfile::TempDir) {
    test_app_with_provider(Arc::new(MockAiProvider)).await
}

async fn test_app_rate_limited() -> (Router, tempfile::TempDir) {
    let pool = common::test_pool().await;
    let tmp = tempfile::TempDir::new().expect("Failed to create temp dir");

    let state = AppState {
        pool,
        image_store: flowl::images::ImageStore::new(tmp.path().to_path_buf()),
        mqtt_client: None,
        mqtt_prefix: "flowl".to_string(),
        mqtt_connected: None,
        mqtt_host: "localhost".to_string(),
        mqtt_port: 1883,
        mqtt_disabled: true,
        ai_provider: Some(Arc::new(MockAiProvider)),
        ai_base_url: "https://api.openai.com/v1".to_string(),
        ai_model: "gpt-4.1-mini".to_string(),
        ai_rate_limiter: Some(Arc::new(AiRateLimiter::new(1))),
    };
    (flowl::server::router(state), tmp)
}

async fn test_app_failing() -> (Router, tempfile::TempDir) {
    test_app_with_provider(Arc::new(FailingAiProvider)).await
}

fn multipart_body(parts: &[(&str, &str, &[u8])]) -> (String, Vec<u8>) {
    let boundary = "----TestBoundary";
    let mut body = Vec::new();

    for (field_name, content_type, data) in parts {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"{field_name}\"; filename=\"photo.jpg\"\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(format!("Content-Type: {content_type}\r\n\r\n").as_bytes());
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
    }

    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    let content_type = format!("multipart/form-data; boundary={boundary}");
    (content_type, body)
}

#[tokio::test]
async fn identify_returns_503_when_ai_not_configured() {
    let (app, _dir) = common::test_app().await;

    let (content_type, body) = multipart_body(&[("photos", "image/jpeg", &[0xFF, 0xD8, 0xFF])]);

    let request = Request::builder()
        .method("POST")
        .uri("/api/ai/identify")
        .header("content-type", content_type)
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = common::body_json(response).await;
    assert_eq!(body["code"], "AI_NOT_CONFIGURED");
    assert!(body["message"].as_str().is_some());
}

#[tokio::test]
async fn identify_returns_422_when_no_photos() {
    let (app, _dir) = test_app_mock().await;

    let boundary = "----TestBoundary";
    let body = format!("--{boundary}--\r\n");
    let content_type = format!("multipart/form-data; boundary={boundary}");

    let request = Request::builder()
        .method("POST")
        .uri("/api/ai/identify")
        .header("content-type", content_type)
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = common::body_json(response).await;
    assert_eq!(body["code"], "PHOTO_NO_FILE");
    assert!(body["message"].as_str().is_some());
}

#[tokio::test]
async fn identify_returns_422_for_invalid_content_type() {
    let (app, _dir) = test_app_mock().await;

    let (content_type, body) = multipart_body(&[("photos", "application/pdf", b"not an image")]);

    let request = Request::builder()
        .method("POST")
        .uri("/api/ai/identify")
        .header("content-type", content_type)
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = common::body_json(response).await;
    assert_eq!(body["code"], "PHOTO_INVALID_TYPE");
    assert!(body["message"].as_str().is_some());
}

#[tokio::test]
async fn identify_returns_200_for_single_photo() {
    let (app, _dir) = test_app_mock().await;

    let (content_type, body) = multipart_body(&[("photos", "image/jpeg", &[0xFF, 0xD8, 0xFF])]);

    let request = Request::builder()
        .method("POST")
        .uri("/api/ai/identify")
        .header("content-type", content_type)
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = common::body_json(response).await;
    let suggestions = body["suggestions"].as_array().unwrap();
    assert_eq!(suggestions.len(), 2);
    assert_eq!(suggestions[0]["common_name"], "Monstera");
    assert_eq!(suggestions[0]["scientific_name"], "Monstera deliciosa");
    assert_eq!(suggestions[0]["confidence"], 0.95);
    assert_eq!(suggestions[0]["summary"], "A popular tropical houseplant");
    assert_eq!(suggestions[1]["common_name"], "Philodendron");
}

#[tokio::test]
async fn identify_returns_200_for_multiple_photos() {
    let (app, _dir) = test_app_mock().await;

    let (content_type, body) = multipart_body(&[
        ("photos", "image/jpeg", &[0xFF, 0xD8, 0xFF]),
        ("photos", "image/png", &[0x89, 0x50, 0x4E, 0x47]),
        ("photos", "image/webp", &[0x52, 0x49, 0x46, 0x46]),
    ]);

    let request = Request::builder()
        .method("POST")
        .uri("/api/ai/identify")
        .header("content-type", content_type)
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = common::body_json(response).await;
    let suggestions = body["suggestions"].as_array().unwrap();
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0]["common_name"], "Monstera");
    assert_eq!(suggestions[0]["scientific_name"], "Monstera deliciosa");
}

#[tokio::test]
async fn identify_returns_500_when_ai_provider_fails() {
    let (app, _dir) = test_app_failing().await;

    let (content_type, body) = multipart_body(&[("photos", "image/jpeg", &[0xFF, 0xD8, 0xFF])]);

    let request = Request::builder()
        .method("POST")
        .uri("/api/ai/identify")
        .header("content-type", content_type)
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = common::body_json(response).await;
    assert_eq!(body["code"], "AI_PROVIDER_FAILED");
    assert!(body["message"].as_str().is_some());
}

#[tokio::test]
async fn identify_rejects_body_exceeding_size_limit() {
    let (app, _dir) = test_app_mock().await;

    // 31 MB payload exceeds the 30 MB limit
    let large_data = vec![0u8; 31 * 1024 * 1024];
    let (content_type, body) = multipart_body(&[("photos", "image/jpeg", &large_data)]);

    let request = Request::builder()
        .method("POST")
        .uri("/api/ai/identify")
        .header("content-type", content_type)
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn identify_returns_429_when_rate_limited() {
    let (app, _dir) = test_app_rate_limited().await;

    let (content_type, body) = multipart_body(&[("photos", "image/jpeg", &[0xFF, 0xD8, 0xFF])]);

    // First request should succeed
    let request = Request::builder()
        .method("POST")
        .uri("/api/ai/identify")
        .header("content-type", &content_type)
        .body(Body::from(body.clone()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Second request should be rate limited
    let request = Request::builder()
        .method("POST")
        .uri("/api/ai/identify")
        .header("content-type", &content_type)
        .body(Body::from(body))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    let body = common::body_json(response).await;
    assert_eq!(body["code"], "AI_RATE_LIMITED");
}

struct RejectingAiProvider;

#[async_trait]
impl AiProvider for RejectingAiProvider {
    async fn identify(
        &self,
        _images: &[&[u8]],
        _locale: &str,
    ) -> Result<IdentifyResponse, Box<dyn std::error::Error + Send + Sync>> {
        Ok(IdentifyResponse {
            suggestions: vec![],
            rejected: Some(true),
            rejected_reason: Some("This is a coffee mug".to_string()),
        })
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
        unimplemented!()
    }
}

#[tokio::test]
async fn identify_returns_422_when_ai_rejects_non_plant() {
    let (app, _dir) = test_app_with_provider(Arc::new(RejectingAiProvider)).await;

    let (content_type, body) = multipart_body(&[("photos", "image/jpeg", &[0xFF, 0xD8, 0xFF])]);

    let request = Request::builder()
        .method("POST")
        .uri("/api/ai/identify")
        .header("content-type", content_type)
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = common::body_json(response).await;
    assert_eq!(body["code"], "AI_IDENTIFY_NOT_A_PLANT");
    assert!(body["message"].as_str().is_some());
}

#[tokio::test]
async fn identify_returns_200_when_ai_accepts_plant() {
    let (app, _dir) = test_app_mock().await;

    let (content_type, body) = multipart_body(&[("photos", "image/jpeg", &[0xFF, 0xD8, 0xFF])]);

    let request = Request::builder()
        .method("POST")
        .uri("/api/ai/identify")
        .header("content-type", content_type)
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = common::body_json(response).await;
    let suggestions = body["suggestions"].as_array().unwrap();
    assert_eq!(suggestions.len(), 2);
    assert_eq!(suggestions[0]["common_name"], "Monstera");
}
