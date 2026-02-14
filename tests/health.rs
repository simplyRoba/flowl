use axum::http::StatusCode;
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_200_with_status_ok() {
    let app = flowl::server::router();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
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

    assert_eq!(json, serde_json::json!({"status": "ok"}));
}
