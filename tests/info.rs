mod common;

use axum::http::StatusCode;
use tower::ServiceExt;

#[tokio::test]
async fn info_returns_200_with_metadata() {
    let app = common::test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/info")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["version"].is_string());
    assert!(json["repository"].is_string());
    assert!(json["license"].is_string());

    assert!(!json["version"].as_str().unwrap().is_empty());
    assert!(!json["repository"].as_str().unwrap().is_empty());
    assert!(!json["license"].as_str().unwrap().is_empty());
}
