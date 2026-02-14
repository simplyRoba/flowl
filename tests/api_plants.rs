mod common;

use axum::http::StatusCode;
use common::{body_json, json_request};
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
            Some(&format!(
                r#"{{"name":"Cactus","location_id":{loc_id}}}"#
            )),
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
