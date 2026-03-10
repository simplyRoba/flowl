mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
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
    assert_eq!(json["photo_count"], 0);
}

#[tokio::test]
async fn stats_counts_all_photos() {
    let (app, _upload_dir) = common::test_app_with_uploads().await;

    // Create a plant
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Photo Plant"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Upload plant photo
    let data = vec![0xFF, 0xD8, 0xFF, 0xE0];
    let resp = app
        .clone()
        .oneshot(multipart_request("/api/plants/1/photo", &data))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Create a care event
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/api/plants/1/care",
            Some(r#"{"event_type":"watered"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Upload care event photo
    let resp = app
        .clone()
        .oneshot(multipart_request("/api/plants/1/care/1/photo", &data))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Check stats
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = common::body_json(resp).await;
    assert_eq!(json["photo_count"], 2);
}

fn multipart_request(uri: &str, data: &[u8]) -> Request<Body> {
    let boundary = "----testboundary";
    let mut body_bytes = Vec::new();
    body_bytes.extend_from_slice(b"------testboundary\r\n");
    body_bytes.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"test.jpg\"\r\n\
          Content-Type: image/jpeg\r\n\r\n",
    );
    body_bytes.extend_from_slice(data);
    body_bytes.extend_from_slice(b"\r\n------testboundary--\r\n");

    Request::builder()
        .method("POST")
        .uri(uri)
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body_bytes))
        .unwrap()
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
    assert_eq!(json["photo_count"], 0);
}
