mod common;

use axum::http::StatusCode;
use tower::ServiceExt;

#[tokio::test]
async fn stats_returns_counts() {
    let app = common::test_app().await;

    // Create a plant first
    let app = app
        .oneshot(common::json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Test Plant"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(app.status(), StatusCode::CREATED);

    let app = common::test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/stats")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let json = common::body_json(response).await;
    assert!(json["plant_count"].is_number());
    assert!(json["care_event_count"].is_number());
}

#[tokio::test]
async fn stats_empty_database() {
    let app = common::test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/stats")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let json = common::body_json(response).await;
    assert_eq!(json["plant_count"], 0);
    assert_eq!(json["care_event_count"], 0);
}
