mod common;

use axum::http::StatusCode;
use common::{body_json, json_request};
use tower::ServiceExt;

async fn app() -> axum::Router {
    common::test_app().await
}

async fn create_plant(app: &axum::Router) -> i64 {
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"TestPlant"}"#),
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    json["id"].as_i64().unwrap()
}

#[tokio::test]
async fn list_empty() {
    let app = app().await;
    let id = create_plant(&app).await;

    let resp = app
        .oneshot(json_request("GET", &format!("/api/plants/{id}/care"), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
async fn create_valid_event() {
    let app = app().await;
    let id = create_plant(&app).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"fertilized","notes":"Half strength"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp).await;
    assert_eq!(json["event_type"], "fertilized");
    assert_eq!(json["notes"], "Half strength");
    assert_eq!(json["plant_id"], id);
    assert_eq!(json["plant_name"], "TestPlant");
    assert!(json["id"].is_number());
    assert!(json["occurred_at"].is_string());
    assert!(json["created_at"].is_string());
}

#[tokio::test]
async fn create_with_explicit_occurred_at() {
    let app = app().await;
    let id = create_plant(&app).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"repotted","occurred_at":"2026-02-14T10:00:00"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp).await;
    assert_eq!(json["occurred_at"], "2026-02-14T10:00:00");
}

#[tokio::test]
async fn create_invalid_type() {
    let app = app().await;
    let id = create_plant(&app).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"unknown"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_missing_type() {
    let app = app().await;
    let id = create_plant(&app).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r"{}"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_nonexistent_plant() {
    let resp = app()
        .await
        .oneshot(json_request(
            "POST",
            "/api/plants/999/care",
            Some(r#"{"event_type":"watered"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_nonexistent_plant() {
    let resp = app()
        .await
        .oneshot(json_request("GET", "/api/plants/999/care", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_event() {
    let app = app().await;
    let plant_id = create_plant(&app).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"pruned"}"#),
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let event_id = json["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/plants/{plant_id}/care/{event_id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let resp = app
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}/care"),
            None,
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
async fn delete_nonexistent_event() {
    let app = app().await;
    let plant_id = create_plant(&app).await;

    let resp = app
        .oneshot(json_request(
            "DELETE",
            &format!("/api/plants/{plant_id}/care/999"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_ordered_by_occurred_at_desc() {
    let app = app().await;
    let id = create_plant(&app).await;

    // Create events with different occurred_at
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-10T08:00:00"}"#),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"fertilized","occurred_at":"2026-02-12T08:00:00"}"#),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"pruned","occurred_at":"2026-02-11T08:00:00"}"#),
        ))
        .await
        .unwrap();

    let resp = app
        .oneshot(json_request("GET", &format!("/api/plants/{id}/care"), None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json.as_array().unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0]["event_type"], "fertilized");
    assert_eq!(events[1]["event_type"], "pruned");
    assert_eq!(events[2]["event_type"], "watered");
}

// --- Global endpoint tests ---

#[tokio::test]
async fn global_empty() {
    let resp = app()
        .await
        .oneshot(json_request("GET", "/api/care", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["events"], serde_json::json!([]));
    assert_eq!(json["has_more"], false);
}

#[tokio::test]
async fn global_returns_events_across_plants() {
    let app = app().await;

    // Create two plants
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Plant A"}"#),
        ))
        .await
        .unwrap();
    let id_a = body_json(resp).await["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Plant B"}"#),
        ))
        .await
        .unwrap();
    let id_b = body_json(resp).await["id"].as_i64().unwrap();

    // Add events to each
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id_a}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-10T08:00:00"}"#),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id_b}/care"),
            Some(r#"{"event_type":"fertilized","occurred_at":"2026-02-11T08:00:00"}"#),
        ))
        .await
        .unwrap();

    let resp = app
        .oneshot(json_request("GET", "/api/care", None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    // Newest first
    assert_eq!(events[0]["plant_name"], "Plant B");
    assert_eq!(events[1]["plant_name"], "Plant A");
}

#[tokio::test]
async fn global_respects_limit() {
    let app = app().await;
    let id = create_plant(&app).await;

    for i in 0..5 {
        app.clone()
            .oneshot(json_request(
                "POST",
                &format!("/api/plants/{id}/care"),
                Some(&format!(
                    r#"{{"event_type":"watered","occurred_at":"2026-02-{:02}T08:00:00"}}"#,
                    10 + i
                )),
            ))
            .await
            .unwrap();
    }

    let resp = app
        .oneshot(json_request("GET", "/api/care?limit=2", None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert_eq!(json["events"].as_array().unwrap().len(), 2);
    assert_eq!(json["has_more"], true);
}

#[tokio::test]
async fn global_cursor_pagination() {
    let app = app().await;
    let id = create_plant(&app).await;

    for i in 0..4 {
        app.clone()
            .oneshot(json_request(
                "POST",
                &format!("/api/plants/{id}/care"),
                Some(&format!(
                    r#"{{"event_type":"watered","occurred_at":"2026-02-{:02}T08:00:00"}}"#,
                    10 + i
                )),
            ))
            .await
            .unwrap();
    }

    // Get first page
    let resp = app
        .clone()
        .oneshot(json_request("GET", "/api/care?limit=2", None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(json["has_more"], true);
    let last_id = events[1]["id"].as_i64().unwrap();

    // Get second page
    let resp = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/care?limit=2&before={last_id}"),
            None,
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(json["has_more"], false);
}

#[tokio::test]
async fn global_type_filter() {
    let app = app().await;
    let id = create_plant(&app).await;

    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"watered"}"#),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"fertilized"}"#),
        ))
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(json_request("GET", "/api/care?type=watered", None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event_type"], "watered");
}

#[tokio::test]
async fn global_invalid_type_filter() {
    let resp = app()
        .await
        .oneshot(json_request("GET", "/api/care?type=invalid", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// --- Water auto-logs care event ---

#[tokio::test]
async fn water_auto_logs_care_event() {
    let app = app().await;
    let id = create_plant(&app).await;

    // Water the plant
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/water"),
            None,
        ))
        .await
        .unwrap();

    // Check care events
    let resp = app
        .oneshot(json_request("GET", &format!("/api/plants/{id}/care"), None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json.as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event_type"], "watered");
    assert_eq!(events[0]["plant_id"], id);
}

// --- Cascade delete ---

#[tokio::test]
async fn delete_plant_cascades_care_events() {
    let app = app().await;
    let id = create_plant(&app).await;

    // Create care event
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"watered"}"#),
        ))
        .await
        .unwrap();

    // Delete plant
    let resp = app
        .clone()
        .oneshot(json_request("DELETE", &format!("/api/plants/{id}"), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Global endpoint should be empty
    let resp = app
        .oneshot(json_request("GET", "/api/care", None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert_eq!(json["events"], serde_json::json!([]));
}
