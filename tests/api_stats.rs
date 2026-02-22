mod common;

use axum::http::StatusCode;
use tower::ServiceExt;

#[tokio::test]
async fn stats_returns_counts() {
    let app = common::test_app().await;

    let response = app
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Living Room"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let response = app
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Test Plant","location_id":1}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

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
    assert_eq!(json["plant_count"], 1);
    assert_eq!(json["care_event_count"], 0);
    assert_eq!(json["location_count"], 1);
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
    assert_eq!(json["location_count"], 0);
}
