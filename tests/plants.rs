mod common;

use axum::http::StatusCode;
use common::{body_json, json_request};
use flowl::api::plants::{compute_watering_status, validate_care_info};
use tower::ServiceExt;

async fn app() -> axum::Router {
    common::test_app().await
}

#[tokio::test]
async fn list_empty() {
    let resp = app()
        .await
        .oneshot(json_request("GET", "/api/plants", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
async fn create_with_defaults() {
    let resp = app()
        .await
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Fern"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp).await;
    assert_eq!(json["name"], "Fern");
    assert_eq!(json["icon"], "🪴");
    assert_eq!(json["watering_interval_days"], 7);
    assert_eq!(json["light_needs"], "indirect");
    assert!(json["id"].is_number());
    assert!(json["created_at"].is_string());
    assert!(json["updated_at"].is_string());
}

#[tokio::test]
async fn create_without_name() {
    let resp = app()
        .await
        .oneshot(json_request("POST", "/api/plants", Some(r"{}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = body_json(resp).await;
    assert!(json["message"].is_string());
}

#[tokio::test]
async fn get_plant_with_location() {
    let app = app().await;

    // Create location
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Balcony"}"#),
        ))
        .await
        .unwrap();
    let loc = body_json(resp).await;
    let loc_id = loc["id"].as_i64().unwrap();

    // Create plant with location
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(&format!(r#"{{"name":"Cactus","location_id":{loc_id}}}"#)),
        ))
        .await
        .unwrap();
    let plant = body_json(resp).await;
    let plant_id = plant["id"].as_i64().unwrap();

    // Get plant
    let resp = app
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["name"], "Cactus");
    assert_eq!(json["location_name"], "Balcony");
    assert_eq!(json["location_id"], loc_id);
}

#[tokio::test]
async fn get_nonexistent() {
    let resp = app()
        .await
        .oneshot(json_request("GET", "/api/plants/999", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = body_json(resp).await;
    assert!(json["message"].is_string());
}

#[tokio::test]
async fn update_plant() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Monstera"}"#),
        ))
        .await
        .unwrap();
    let plant = body_json(resp).await;
    let id = plant["id"].as_i64().unwrap();
    let original_updated_at = plant["updated_at"].as_str().unwrap().to_string();

    // Small delay to ensure timestamp changes
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let resp = app
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{id}"),
            Some(r#"{"name":"Monstera Deliciosa"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["name"], "Monstera Deliciosa");
    assert_ne!(json["updated_at"].as_str().unwrap(), original_updated_at);
}

#[tokio::test]
async fn update_nonexistent() {
    let resp = app()
        .await
        .oneshot(json_request(
            "PUT",
            "/api/plants/999",
            Some(r#"{"name":"X"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_plant() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Orchid"}"#),
        ))
        .await
        .unwrap();
    let plant = body_json(resp).await;
    let id = plant["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(json_request("DELETE", &format!("/api/plants/{id}"), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let resp = app
        .oneshot(json_request("GET", &format!("/api/plants/{id}"), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_nonexistent() {
    let resp = app()
        .await
        .oneshot(json_request("DELETE", "/api/plants/999", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn invalid_json_returns_400() {
    let resp = app()
        .await
        .oneshot(json_request("POST", "/api/plants", Some("{invalid")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp).await;
    assert!(json["message"].is_string());
}

#[tokio::test]
async fn update_clears_location() {
    let app = app().await;

    // Create location
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Office"}"#),
        ))
        .await
        .unwrap();
    let loc = body_json(resp).await;
    let loc_id = loc["id"].as_i64().unwrap();

    // Create plant with location
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(&format!(r#"{{"name":"Ficus","location_id":{loc_id}}}"#)),
        ))
        .await
        .unwrap();
    let plant = body_json(resp).await;
    let plant_id = plant["id"].as_i64().unwrap();
    assert_eq!(plant["location_id"], loc_id);

    // Update plant with location_id: null
    let resp = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}"),
            Some(r#"{"location_id":null}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["location_id"].is_null());
    assert!(json["location_name"].is_null());
}

#[tokio::test]
async fn new_plant_has_due_status() {
    let resp = app()
        .await
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Aloe"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp).await;
    assert_eq!(json["watering_status"], "due");
    assert!(json["last_watered"].is_null());
    assert!(json["next_due"].is_null());
}

#[tokio::test]
async fn water_plant_sets_last_watered() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Rose"}"#),
        ))
        .await
        .unwrap();
    let plant = body_json(resp).await;
    let id = plant["id"].as_i64().unwrap();
    assert!(plant["last_watered"].is_null());

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/water"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["last_watered"].is_string());
    assert_eq!(json["watering_status"], "ok");
    assert!(json["next_due"].is_string());
}

#[tokio::test]
async fn water_nonexistent_plant() {
    let resp = app()
        .await
        .oneshot(json_request("POST", "/api/plants/999/water", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_plant_includes_watering_fields() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Basil"}"#),
        ))
        .await
        .unwrap();
    let plant = body_json(resp).await;
    let id = plant["id"].as_i64().unwrap();

    let resp = app
        .oneshot(json_request("GET", &format!("/api/plants/{id}"), None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert!(json.get("watering_status").is_some());
    assert!(json.get("last_watered").is_some());
    assert!(json.get("next_due").is_some());
}

#[test]
fn status_never_watered() {
    let (status, next_due) = compute_watering_status(None, 7);
    assert_eq!(status, "due");
    assert!(next_due.is_none());
}

#[test]
fn status_ok() {
    let today = chrono::Utc::now().date_naive();
    let yesterday = (today - chrono::Days::new(1)).to_string();
    let (status, next_due) = compute_watering_status(Some(&yesterday), 7);
    assert_eq!(status, "ok");
    assert!(next_due.is_some());
}

#[test]
fn status_due_today() {
    let today = chrono::Utc::now().date_naive();
    let watered = (today - chrono::Days::new(7)).to_string();
    let (status, next_due) = compute_watering_status(Some(&watered), 7);
    assert_eq!(status, "due");
    assert_eq!(next_due.as_deref(), Some(today.to_string().as_str()));
}

#[test]
fn status_overdue() {
    let today = chrono::Utc::now().date_naive();
    let watered = (today - chrono::Days::new(10)).to_string();
    let (status, next_due) = compute_watering_status(Some(&watered), 7);
    assert_eq!(status, "overdue");
    assert!(next_due.is_some());
}

// --- Care info validation tests ---

#[test]
fn care_info_valid_values() {
    assert!(
        validate_care_info(
            "difficulty",
            Some("easy"),
            &["easy", "moderate", "demanding"]
        )
        .is_ok()
    );
    assert!(
        validate_care_info(
            "difficulty",
            Some("moderate"),
            &["easy", "moderate", "demanding"]
        )
        .is_ok()
    );
    assert!(
        validate_care_info(
            "difficulty",
            Some("demanding"),
            &["easy", "moderate", "demanding"]
        )
        .is_ok()
    );
    assert!(validate_care_info("pet_safety", Some("safe"), &["safe", "caution", "toxic"]).is_ok());
    assert!(
        validate_care_info("growth_speed", Some("slow"), &["slow", "moderate", "fast"]).is_ok()
    );
    assert!(
        validate_care_info(
            "soil_type",
            Some("cactus-mix"),
            &["standard", "cactus-mix", "orchid-bark", "peat-moss"]
        )
        .is_ok()
    );
}

#[test]
fn care_info_null_allowed() {
    assert!(validate_care_info("difficulty", None, &["easy", "moderate", "demanding"]).is_ok());
}

#[test]
fn care_info_invalid_value() {
    let result = validate_care_info(
        "difficulty",
        Some("impossible"),
        &["easy", "moderate", "demanding"],
    );
    assert!(result.is_err());
}

// --- Care info integration tests ---

#[tokio::test]
async fn create_with_care_info() {
    let resp = app()
        .await
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Cactus","difficulty":"easy","pet_safety":"safe","soil_type":"cactus-mix","soil_moisture":"dry"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp).await;
    assert_eq!(json["difficulty"], "easy");
    assert_eq!(json["pet_safety"], "safe");
    assert!(json["growth_speed"].is_null());
    assert_eq!(json["soil_type"], "cactus-mix");
    assert_eq!(json["soil_moisture"], "dry");
}

#[tokio::test]
async fn create_defaults_care_info_null() {
    let resp = app()
        .await
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Fern"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp).await;
    assert!(json["difficulty"].is_null());
    assert!(json["pet_safety"].is_null());
    assert!(json["growth_speed"].is_null());
    assert!(json["soil_type"].is_null());
    assert!(json["soil_moisture"].is_null());
}

#[tokio::test]
async fn create_invalid_care_info() {
    let resp = app()
        .await
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Fern","difficulty":"impossible"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn update_set_care_info() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Orchid"}"#),
        ))
        .await
        .unwrap();
    let plant = body_json(resp).await;
    let id = plant["id"].as_i64().unwrap();
    assert!(plant["difficulty"].is_null());

    let resp = app
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{id}"),
            Some(r#"{"difficulty":"demanding","pet_safety":"toxic"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["difficulty"], "demanding");
    assert_eq!(json["pet_safety"], "toxic");
}

#[tokio::test]
async fn update_clear_care_info() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Rose","difficulty":"easy"}"#),
        ))
        .await
        .unwrap();
    let plant = body_json(resp).await;
    let id = plant["id"].as_i64().unwrap();
    assert_eq!(plant["difficulty"], "easy");

    let resp = app
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{id}"),
            Some(r#"{"difficulty":null}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["difficulty"].is_null());
}

#[tokio::test]
async fn update_set_and_clear_soil_moisture() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Basil"}"#),
        ))
        .await
        .unwrap();
    let plant = body_json(resp).await;
    let id = plant["id"].as_i64().unwrap();
    assert!(plant["soil_moisture"].is_null());

    // Set soil_moisture
    let resp = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{id}"),
            Some(r#"{"soil_moisture":"moist"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["soil_moisture"], "moist");

    // Clear soil_moisture
    let resp = app
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{id}"),
            Some(r#"{"soil_moisture":null}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["soil_moisture"].is_null());
}

#[tokio::test]
async fn update_invalid_care_info() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Lily"}"#),
        ))
        .await
        .unwrap();
    let plant = body_json(resp).await;
    let id = plant["id"].as_i64().unwrap();

    let resp = app
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{id}"),
            Some(r#"{"pet_safety":"unknown"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
