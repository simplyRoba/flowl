mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{body_json, json_request};
use tower::ServiceExt;

fn multipart_request(uri: &str, content_type: &str, data: &[u8]) -> Request<Body> {
    let boundary = "----testboundary";
    let mut body_bytes = Vec::new();
    body_bytes.extend_from_slice(format!("------testboundary\r\n").as_bytes());
    body_bytes.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"test.jpg\"\r\n\
             Content-Type: {content_type}\r\n\r\n"
        )
        .as_bytes(),
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

async fn create_plant(app: &axum::Router) -> i64 {
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Test Plant"}"#),
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    json["id"].as_i64().unwrap()
}

#[tokio::test]
async fn upload_photo() {
    let (app, upload_dir) = common::test_app_with_uploads().await;
    let id = create_plant(&app).await;

    let data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // minimal JPEG header bytes
    let resp = app
        .clone()
        .oneshot(multipart_request(
            &format!("/api/plants/{id}/photo"),
            "image/jpeg",
            &data,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp).await;
    assert!(json["photo_url"].as_str().unwrap().starts_with("/uploads/"));
    assert!(json["photo_url"].as_str().unwrap().ends_with(".jpg"));

    // Verify file exists on disk
    let filename = json["photo_url"]
        .as_str()
        .unwrap()
        .strip_prefix("/uploads/")
        .unwrap();
    assert!(upload_dir.join(filename).exists());
}

#[tokio::test]
async fn upload_to_nonexistent_plant() {
    let app = common::test_app().await;
    let data = vec![0xFF, 0xD8, 0xFF, 0xE0];
    let resp = app
        .oneshot(multipart_request(
            "/api/plants/999/photo",
            "image/jpeg",
            &data,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn upload_replaces_existing_photo() {
    let (app, upload_dir) = common::test_app_with_uploads().await;
    let id = create_plant(&app).await;

    // Upload first photo
    let resp = app
        .clone()
        .oneshot(multipart_request(
            &format!("/api/plants/{id}/photo"),
            "image/jpeg",
            &[0xFF, 0xD8],
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let first_filename = json["photo_url"]
        .as_str()
        .unwrap()
        .strip_prefix("/uploads/")
        .unwrap()
        .to_string();
    assert!(upload_dir.join(&first_filename).exists());

    // Upload second photo
    let resp = app
        .clone()
        .oneshot(multipart_request(
            &format!("/api/plants/{id}/photo"),
            "image/png",
            &[0x89, 0x50],
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["photo_url"].as_str().unwrap().ends_with(".png"));

    // Old file should be deleted
    assert!(!upload_dir.join(&first_filename).exists());
}

#[tokio::test]
async fn upload_rejects_invalid_type() {
    let app = common::test_app().await;
    let id = create_plant(&app).await;

    let resp = app
        .clone()
        .oneshot(multipart_request(
            &format!("/api/plants/{id}/photo"),
            "text/plain",
            b"not an image",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn upload_rejects_oversized_file() {
    let app = common::test_app().await;
    let id = create_plant(&app).await;

    let data = vec![0u8; 6 * 1024 * 1024]; // 6 MB
    let resp = app
        .clone()
        .oneshot(multipart_request(
            &format!("/api/plants/{id}/photo"),
            "image/jpeg",
            &data,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn delete_photo() {
    let (app, upload_dir) = common::test_app_with_uploads().await;
    let id = create_plant(&app).await;

    // Upload photo
    let resp = app
        .clone()
        .oneshot(multipart_request(
            &format!("/api/plants/{id}/photo"),
            "image/jpeg",
            &[0xFF, 0xD8],
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let filename = json["photo_url"]
        .as_str()
        .unwrap()
        .strip_prefix("/uploads/")
        .unwrap()
        .to_string();
    assert!(upload_dir.join(&filename).exists());

    // Delete photo
    let resp = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/plants/{id}/photo"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify file is removed
    assert!(!upload_dir.join(&filename).exists());

    // Verify plant has no photo_url
    let resp = app
        .oneshot(json_request("GET", &format!("/api/plants/{id}"), None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert!(json["photo_url"].is_null());
}

#[tokio::test]
async fn delete_photo_when_none_exists() {
    let app = common::test_app().await;
    let id = create_plant(&app).await;

    let resp = app
        .oneshot(json_request(
            "DELETE",
            &format!("/api/plants/{id}/photo"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_plant_deletes_photo_file() {
    let (app, upload_dir) = common::test_app_with_uploads().await;
    let id = create_plant(&app).await;

    // Upload photo
    let resp = app
        .clone()
        .oneshot(multipart_request(
            &format!("/api/plants/{id}/photo"),
            "image/jpeg",
            &[0xFF, 0xD8],
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let filename = json["photo_url"]
        .as_str()
        .unwrap()
        .strip_prefix("/uploads/")
        .unwrap()
        .to_string();
    assert!(upload_dir.join(&filename).exists());

    // Delete plant
    let resp = app
        .clone()
        .oneshot(json_request("DELETE", &format!("/api/plants/{id}"), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Photo file should be cleaned up
    assert!(!upload_dir.join(&filename).exists());
}
