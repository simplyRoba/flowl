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
        .oneshot(json_request("GET", "/api/locations", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
async fn create_location() {
    let resp = app()
        .await
        .oneshot(json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Living Room"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp).await;
    assert_eq!(json["name"], "Living Room");
    assert!(json["id"].is_number());
}

#[tokio::test]
async fn create_duplicate() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Kitchen"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Kitchen"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn create_without_name() {
    let resp = app()
        .await
        .oneshot(json_request("POST", "/api/locations", Some(r"{}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn list_ordered_by_name() {
    let app = app().await;

    app.clone()
        .oneshot(json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Bedroom"}"#),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Attic"}"#),
        ))
        .await
        .unwrap();

    let resp = app
        .oneshot(json_request("GET", "/api/locations", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    let names: Vec<&str> = json.as_array().unwrap().iter().map(|l| l["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["Attic", "Bedroom"]);
}

#[tokio::test]
async fn update_location() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Old Name"}"#),
        ))
        .await
        .unwrap();
    let created = body_json(resp).await;
    let id = created["id"].as_i64().unwrap();

    let resp = app
        .oneshot(json_request(
            "PUT",
            &format!("/api/locations/{id}"),
            Some(r#"{"name":"New Name"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["name"], "New Name");
    assert_eq!(json["id"], id);
}

#[tokio::test]
async fn update_nonexistent() {
    let resp = app()
        .await
        .oneshot(json_request(
            "PUT",
            "/api/locations/999",
            Some(r#"{"name":"X"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_duplicate_name() {
    let app = app().await;

    app.clone()
        .oneshot(json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Kitchen"}"#),
        ))
        .await
        .unwrap();
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Bedroom"}"#),
        ))
        .await
        .unwrap();
    let created = body_json(resp).await;
    let id = created["id"].as_i64().unwrap();

    let resp = app
        .oneshot(json_request(
            "PUT",
            &format!("/api/locations/{id}"),
            Some(r#"{"name":"Kitchen"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn delete_location() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Garage"}"#),
        ))
        .await
        .unwrap();
    let created = body_json(resp).await;
    let id = created["id"].as_i64().unwrap();

    let resp = app
        .oneshot(json_request("DELETE", &format!("/api/locations/{id}"), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_nonexistent() {
    let resp = app()
        .await
        .oneshot(json_request("DELETE", "/api/locations/999", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_nullifies_plant_references() {
    let app = app().await;

    // Create location
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/locations",
            Some(r#"{"name":"Patio"}"#),
        ))
        .await
        .unwrap();
    let loc = body_json(resp).await;
    let loc_id = loc["id"].as_i64().unwrap();

    // Create plant with that location
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(&format!(
                r#"{{"name":"Fern","location_id":{loc_id}}}"#
            )),
        ))
        .await
        .unwrap();
    let plant = body_json(resp).await;
    let plant_id = plant["id"].as_i64().unwrap();
    assert_eq!(plant["location_id"], loc_id);

    // Delete location
    let resp = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/locations/{loc_id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify plant location_id is now null
    let resp = app
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}"),
            None,
        ))
        .await
        .unwrap();
    let plant = body_json(resp).await;
    assert!(plant["location_id"].is_null());
    assert!(plant["location_name"].is_null());
}
