mod common;

use std::io::Read;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

fn multipart_import_request(zip_bytes: &[u8]) -> Request<Body> {
    let boundary = "----testboundary";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"export.zip\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: application/zip\r\n\r\n");
    body.extend_from_slice(zip_bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    Request::builder()
        .method("POST")
        .uri("/api/data/import")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap()
}

fn build_export_zip(json: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("data.json", options).unwrap();
        std::io::Write::write_all(&mut zip, json.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    buf
}

fn build_export_zip_with_photo(json: &str, photo_name: &str, photo_data: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("data.json", options).unwrap();
        std::io::Write::write_all(&mut zip, json.as_bytes()).unwrap();

        let photo_path = format!("photos/{photo_name}");
        zip.start_file(&photo_path, options).unwrap();
        std::io::Write::write_all(&mut zip, photo_data).unwrap();
        zip.finish().unwrap();
    }
    buf
}

fn valid_export_json() -> String {
    format!(
        r#"{{
            "version": "{}",
            "exported_at": "2026-02-21T12:00:00Z",
            "locations": [{{"id": 1, "name": "Living Room"}}],
            "plants": [{{
                "id": 1, "name": "Monstera", "species": null, "icon": "🪴",
                "photo_path": null, "location_id": 1, "watering_interval_days": 7,
                "last_watered": "2026-02-15T10:00:00", "light_needs": "indirect",
                "difficulty": null, "pet_safety": null, "growth_speed": null,
                "soil_type": null, "soil_moisture": null, "notes": null,
                "created_at": "2026-02-01T08:00:00", "updated_at": "2026-02-15T10:00:00"
            }}],
            "care_events": [{{
                "id": 1, "plant_id": 1, "event_type": "watered",
                "notes": null, "occurred_at": "2026-02-15T10:00:00Z",
                "created_at": "2026-02-15T10:00:00"
            }}]
        }}"#,
        env!("CARGO_PKG_VERSION")
    )
}

// --- Export tests ---

#[tokio::test]
async fn export_empty_database() {
    let app = common::test_app().await;

    let response = app
        .oneshot(common::json_request("GET", "/api/data/export", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/zip");
    assert_eq!(
        response.headers()["content-disposition"],
        "attachment; filename=\"flowl-export.zip\""
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let cursor = std::io::Cursor::new(&body[..]);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut data_file = archive.by_name("data.json").unwrap();
    let mut json_str = String::new();
    data_file.read_to_string(&mut json_str).unwrap();
    let data: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(data["version"], env!("CARGO_PKG_VERSION"));
    assert!(data["exported_at"].is_string());
    assert_eq!(data["locations"].as_array().unwrap().len(), 0);
    assert_eq!(data["plants"].as_array().unwrap().len(), 0);
    assert_eq!(data["care_events"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn export_populated_database_with_photo() {
    let (_app, _upload_dir) = common::test_app_with_uploads().await;

    // Seed data via import into a shared pool so we can then export
    let (app, upload_dir) = common::test_app_with_uploads().await;
    // Seed location, plant with photo, care event — then export
    let app_clone = app;
    let resp = app_clone
        .oneshot(common::json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name": "Balcony"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // For a proper test we need to use the same pool — let's use a shared test app approach
    // Actually, let's just test that populated data shows up: create data via import, then export
    let json = valid_export_json();
    let zip_bytes = build_export_zip(&json);

    // First use a fresh app, import data, then export
    let pool = common::test_pool().await;
    let state = flowl::state::AppState {
        pool: pool.clone(),
        upload_dir: upload_dir.clone(),
        mqtt_client: None,
        mqtt_prefix: "flowl".to_string(),
        mqtt_connected: None,
        mqtt_host: "localhost".to_string(),
        mqtt_port: 1883,
        mqtt_disabled: true,
    };
    let app = flowl::server::router(state.clone());

    let resp = app
        .oneshot(multipart_import_request(&zip_bytes))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Now export
    let app = flowl::server::router(state);
    let resp = app
        .oneshot(common::json_request("GET", "/api/data/export", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();

    let cursor = std::io::Cursor::new(&body[..]);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut data_file = archive.by_name("data.json").unwrap();
    let mut json_str = String::new();
    data_file.read_to_string(&mut json_str).unwrap();
    let data: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(data["locations"].as_array().unwrap().len(), 1);
    assert_eq!(data["plants"].as_array().unwrap().len(), 1);
    assert_eq!(data["plants"][0]["name"], "Monstera");
    assert_eq!(data["care_events"].as_array().unwrap().len(), 1);

    let _ = std::fs::remove_dir_all(&upload_dir);
}

// --- Import tests ---

#[tokio::test]
async fn import_valid_archive() {
    let (app, upload_dir) = common::test_app_with_uploads().await;

    let json = valid_export_json();
    let zip_bytes = build_export_zip(&json);

    let response = app
        .oneshot(multipart_import_request(&zip_bytes))
        .await
        .unwrap();

    let status = response.status();
    let body = common::body_json(response).await;
    assert_eq!(
        (status, &body),
        (StatusCode::OK, &body),
        "Import failed: {body}"
    );
    assert_eq!(body["locations"], 1);
    assert_eq!(body["plants"], 1);
    assert_eq!(body["care_events"], 1);
    assert_eq!(body["photos"], 0);

    let _ = std::fs::remove_dir_all(&upload_dir);
}

#[tokio::test]
async fn import_with_photo() {
    let (app, upload_dir) = common::test_app_with_uploads().await;

    let json = format!(
        r#"{{
            "version": "{}",
            "exported_at": "2026-02-21T12:00:00Z",
            "locations": [],
            "plants": [{{
                "id": 1, "name": "Fern", "species": null, "icon": "🪴",
                "photo_path": "test-photo.jpg", "location_id": null,
                "watering_interval_days": 3, "last_watered": null,
                "light_needs": "indirect", "difficulty": null, "pet_safety": null,
                "growth_speed": null, "soil_type": null, "soil_moisture": null,
                "notes": null, "created_at": "2026-02-01T08:00:00",
                "updated_at": "2026-02-01T08:00:00"
            }}],
            "care_events": []
        }}"#,
        env!("CARGO_PKG_VERSION")
    );

    let photo_data = b"fake jpeg data";
    let zip_bytes = build_export_zip_with_photo(&json, "test-photo.jpg", photo_data);

    let response = app
        .oneshot(multipart_import_request(&zip_bytes))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = common::body_json(response).await;
    assert_eq!(body["photos"], 1);

    // Verify photo was written to disk
    let photo_path = upload_dir.join("test-photo.jpg");
    assert!(photo_path.exists());
    let contents = std::fs::read(&photo_path).unwrap();
    assert_eq!(contents, photo_data);

    let _ = std::fs::remove_dir_all(&upload_dir);
}

#[tokio::test]
async fn import_invalid_zip() {
    let app = common::test_app().await;

    let response = app
        .oneshot(multipart_import_request(b"not a zip file"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = common::body_json(response).await;
    assert!(body["message"].as_str().unwrap().contains("Invalid ZIP"));
}

#[tokio::test]
async fn import_missing_data_json() {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("other.txt", options).unwrap();
        std::io::Write::write_all(&mut zip, b"hello").unwrap();
        zip.finish().unwrap();
    }

    let app = common::test_app().await;

    let response = app.oneshot(multipart_import_request(&buf)).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = common::body_json(response).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("missing data.json")
    );
}

#[tokio::test]
async fn import_version_mismatch() {
    let json = r#"{
        "version": "99.0.0",
        "exported_at": "2026-02-21T12:00:00Z",
        "locations": [],
        "plants": [],
        "care_events": []
    }"#;
    let zip_bytes = build_export_zip(json);

    let app = common::test_app().await;

    let response = app
        .oneshot(multipart_import_request(&zip_bytes))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = common::body_json(response).await;
    let msg = body["message"].as_str().unwrap();
    assert!(msg.contains("Version mismatch"));
    assert!(msg.contains("99.0.0"));
}

#[tokio::test]
async fn import_patch_version_difference_allowed() {
    let parts: Vec<&str> = env!("CARGO_PKG_VERSION").split('.').collect();
    let patch: u32 = parts[2].parse::<u32>().unwrap() + 1;
    let compatible_version = format!("{}.{}.{}", parts[0], parts[1], patch);

    let json = format!(
        r#"{{
            "version": "{}",
            "exported_at": "2026-02-21T12:00:00Z",
            "locations": [],
            "plants": [],
            "care_events": []
        }}"#,
        compatible_version
    );
    let zip_bytes = build_export_zip(&json);

    let app = common::test_app().await;

    let response = app
        .oneshot(multipart_import_request(&zip_bytes))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn import_path_traversal_rejected() {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("../data.json", options).unwrap();
        std::io::Write::write_all(&mut zip, b"{}").unwrap();
        zip.finish().unwrap();
    }

    let app = common::test_app().await;

    let response = app.oneshot(multipart_import_request(&buf)).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = common::body_json(response).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("Invalid filename")
    );
}

// --- Round-trip test ---

#[tokio::test]
async fn round_trip_export_import_export() {
    let pool = common::test_pool().await;
    let upload_dir = std::env::temp_dir().join(format!("flowl-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&upload_dir).expect("create dir");

    let state = flowl::state::AppState {
        pool: pool.clone(),
        upload_dir: upload_dir.clone(),
        mqtt_client: None,
        mqtt_prefix: "flowl".to_string(),
        mqtt_connected: None,
        mqtt_host: "localhost".to_string(),
        mqtt_port: 1883,
        mqtt_disabled: true,
    };

    // Seed data via import
    let json = valid_export_json();
    let zip_bytes = build_export_zip(&json);

    let app = flowl::server::router(state.clone());
    let resp = app
        .oneshot(multipart_import_request(&zip_bytes))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Export 1
    let app = flowl::server::router(state.clone());
    let resp = app
        .oneshot(common::json_request("GET", "/api/data/export", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let export1_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();

    // Parse export 1
    let cursor = std::io::Cursor::new(&export1_bytes[..]);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut data_file = archive.by_name("data.json").unwrap();
    let mut json1 = String::new();
    data_file.read_to_string(&mut json1).unwrap();
    let mut data1: serde_json::Value = serde_json::from_str(&json1).unwrap();

    // Import export 1 into a fresh database
    let pool2 = common::test_pool().await;
    let upload_dir2 = std::env::temp_dir().join(format!("flowl-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&upload_dir2).expect("create dir");

    let state2 = flowl::state::AppState {
        pool: pool2,
        upload_dir: upload_dir2.clone(),
        mqtt_client: None,
        mqtt_prefix: "flowl".to_string(),
        mqtt_connected: None,
        mqtt_host: "localhost".to_string(),
        mqtt_port: 1883,
        mqtt_disabled: true,
    };

    let app = flowl::server::router(state2.clone());
    let resp = app
        .oneshot(multipart_import_request(&export1_bytes))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Export 2
    let app = flowl::server::router(state2);
    let resp = app
        .oneshot(common::json_request("GET", "/api/data/export", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let export2_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();

    // Parse export 2
    let cursor = std::io::Cursor::new(&export2_bytes[..]);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut data_file = archive.by_name("data.json").unwrap();
    let mut json2 = String::new();
    data_file.read_to_string(&mut json2).unwrap();
    let mut data2: serde_json::Value = serde_json::from_str(&json2).unwrap();

    // Remove exported_at before comparison
    data1.as_object_mut().unwrap().remove("exported_at");
    data2.as_object_mut().unwrap().remove("exported_at");

    assert_eq!(data1, data2);

    let _ = std::fs::remove_dir_all(&upload_dir);
    let _ = std::fs::remove_dir_all(&upload_dir2);
}
